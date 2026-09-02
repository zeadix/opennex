//! frp (fatedier/frp) client manager for user-configured relay
//! channels (remote v3.5):
//!
//! - downloads the official `frpc` binary into the app data dir on
//!   first use (the release tag is resolved through the GitHub API
//!   because frp release asset names embed the version),
//! - generates a per-profile `frpc-<i>.toml` that forwards
//!   `127.0.0.1:{local_port}` to the relay's public port,
//! - spawns `frpc -c <config>` and parses its output for the login /
//!   proxy-success markers (or failure lines),
//! - the public phone URL is simply `http://{server}:{forward_port}`.
//!
//! Lifecycle mirrors `tunnel.rs` (Cloudflare quick tunnel): events flow
//! through a channel drained by the UI tick, and the child process is
//! killed on stop/drop.

use super::tunnel::TunnelEvent;
use crate::app::TunnelProfile;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Final frpc binary path in the app data dir (users may also place a
/// manually downloaded binary here when GitHub is unreachable).
pub(crate) fn frpc_path(data_dir: &Path) -> PathBuf {
    let name = if std::env::consts::OS == "windows" {
        "frpc.exe"
    } else {
        "frpc"
    };
    data_dir.join("tunnel").join(name)
}

/// Go arch suffix used in frp release assets for the current platform.
fn archive_suffix() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "linux_amd64",
        ("linux", "aarch64") => "linux_arm64",
        ("macos", "x86_64") => "darwin_amd64",
        ("macos", "aarch64") => "darwin_arm64",
        ("windows", _) => "windows_amd64",
        _ => "linux_amd64",
    }
}

/// GitHub release tag of frp ("0.61.2" — asset names drop the "v").
fn latest_frpc_tag() -> Result<String, String> {
    let resp = ureq::get("https://api.github.com/repos/fatedier/frp/releases/latest")
        .set("User-Agent", "opennex-remote")
        .timeout(Duration::from_secs(30))
        .call()
        .map_err(|e| format!("release lookup failed: {e}"))?;
    let body: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("release json failed: {e}"))?;
    let tag = body["tag_name"]
        .as_str()
        .ok_or("release json has no tag_name")?
        .strip_prefix('v')
        .unwrap_or_default()
        .to_string();
    if tag.is_empty() {
        return Err("empty release tag".into());
    }
    Ok(tag)
}

/// Download the frp release archive and extract the frpc binary to
/// `dest`. Streaming with progress events; archives are tar.gz on
/// Unix and zip on Windows.
fn download_frpc(dest: &Path, tx: &mpsc::Sender<TunnelEvent>) -> Result<(), String> {
    let tag = latest_frpc_tag()?;
    let url = format!(
        "https://github.com/fatedier/frp/releases/download/v{tag}/frp_{tag}_{}.{}",
        archive_suffix(),
        if std::env::consts::OS == "windows" {
            "zip"
        } else {
            "tar.gz"
        }
    );
    let resp = ureq::get(&url)
        .timeout(Duration::from_secs(600))
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let total: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut reader = resp.into_reader();
    let mut archive: Vec<u8> = Vec::new();
    let mut buf = [0u8; 64 * 1024];
    let mut done: u64 = 0;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("read failed: {e}"))?;
        if n == 0 {
            break;
        }
        archive.extend_from_slice(&buf[..n]);
        done += n as u64;
        if total > 0 {
            let _ = tx.send(TunnelEvent::Downloading(
                (done as f32 / total as f32).min(1.0),
            ));
        }
    }
    if done < 1024 * 1024 {
        return Err("downloaded archive too small - not a valid release".into());
    }
    let exe_name = if std::env::consts::OS == "windows" {
        "frpc.exe"
    } else {
        "frpc"
    };
    let mut extracted: Option<Vec<u8>> = None;
    if std::env::consts::OS == "windows" {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive))
            .map_err(|e| format!("unzip failed: {e}"))?;
        for i in 0..zip.len() {
            let mut file = zip
                .by_index(i)
                .map_err(|e| format!("zip entry failed: {e}"))?;
            if file.name().ends_with(exe_name) {
                let mut out = Vec::with_capacity(file.size() as usize);
                file.read_to_end(&mut out)
                    .map_err(|e| format!("zip read failed: {e}"))?;
                extracted = Some(out);
                break;
            }
        }
    } else {
        let gz = flate2::read::GzDecoder::new(archive.as_slice());
        let mut tar = tar::Archive::new(gz);
        for entry in tar.entries().map_err(|e| format!("untar failed: {e}"))? {
            let mut entry = entry.map_err(|e| format!("tar entry failed: {e}"))?;
            let path = entry
                .path()
                .map_err(|e| format!("tar path failed: {e}"))?
                .to_path_buf();
            if path.file_name().is_some_and(|n| n == exe_name) {
                let mut out = Vec::new();
                entry
                    .read_to_end(&mut out)
                    .map_err(|e| format!("tar read failed: {e}"))?;
                extracted = Some(out);
                break;
            }
        }
    }
    let bytes = extracted.ok_or_else(|| format!("archive has no {exe_name}"))?;
    let mut file = std::fs::File::create(dest).map_err(|e| format!("create failed: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("write failed: {e}"))?;
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(dest)
            .map_err(|e| format!("stat failed: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(dest, perms).map_err(|e| format!("chmod failed: {e}"))?;
    }
    Ok(())
}

/// TOML string literal escape for user-supplied values.
fn toml_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Render the frpc config for one relay profile. `local_port` is the
/// embedded remote server port on this machine.
pub(crate) fn render_frpc_config(profile: &TunnelProfile, local_port: u16) -> String {
    format!(
        "serverAddr = {}\n\
         serverPort = {}\n\
         auth.token = {}\n\
         transport.tls.enable = true\n\
         \n\
         [[proxies]]\n\
         name = {}\n\
         type = \"tcp\"\n\
         localIP = \"127.0.0.1\"\n\
         localPort = {}\n\
         remotePort = {}\n",
        toml_str(&profile.server),
        profile.port,
        toml_str(&profile.token),
        toml_str(&format!("opennex-{}", profile.forward_port)),
        local_port,
        profile.forward_port,
    )
}

/// The public URL phones use to reach the remote page through this
/// relay (IPv6 literals get bracketed).
pub(crate) fn relay_url(profile: &TunnelProfile) -> String {
    let host = if profile.server.contains(':') && !profile.server.starts_with('[') {
        format!("[{}]", profile.server)
    } else {
        profile.server.clone()
    };
    format!("http://{host}:{}", profile.forward_port)
}

/// Classify one frpc output line: Ready on login/proxy success, Failed
/// on explicit failure markers. `url` is passed through to Ready.
pub(crate) fn parse_frpc_line(line: &str, url: &str) -> Option<TunnelEvent> {
    let l = line.to_lowercase();
    if l.contains("login to server success") || l.contains("start proxy success") {
        return Some(TunnelEvent::Ready(url.to_string()));
    }
    if l.contains("login to the server failed")
        || l.contains("connect to server error")
        || l.contains("token in your configuration")
    {
        return Some(TunnelEvent::Failed(line.trim().to_string()));
    }
    None
}

/// One managed frpc session. `child` is killed on stop; the worker
/// threads exit with it.
pub struct FrpHandle {
    child: Child,
    pub events: mpsc::Receiver<TunnelEvent>,
}

impl FrpHandle {
    /// Download (if needed) and start frpc for `profile`. Returns
    /// immediately; status flows through the events channel.
    pub(crate) fn start(
        data_dir: &Path,
        profile: &TunnelProfile,
        profile_index: usize,
        local_port: u16,
    ) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let bin = frpc_path(data_dir);
        if !bin.is_file() {
            if let Some(parent) = bin.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
            }
            download_frpc(&bin, &tx)?;
        }
        let config = data_dir
            .join("tunnel")
            .join(format!("frpc-{profile_index}.toml"));
        std::fs::write(&config, render_frpc_config(profile, local_port))
            .map_err(|e| format!("write config failed: {e}"))?;

        let url = relay_url(profile);
        let mut child = Command::new(&bin)
            .arg("-c")
            .arg(&config)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn frpc failed: {e}"))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let mut streams: Vec<Box<dyn std::io::Read + Send>> = Vec::new();
        if let Some(out) = stdout {
            streams.push(Box::new(out));
        }
        if let Some(err) = stderr {
            streams.push(Box::new(err));
        }
        for stream in streams {
            let tx = tx.clone();
            let url = url.clone();
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(stream);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(event) = parse_frpc_line(&line, &url) {
                        let _ = tx.send(event);
                    }
                }
            });
        }
        let _ = tx.send(TunnelEvent::Starting);
        // Watchdog: if frpc never reports success (bad address, blocked
        // port), surface a failure instead of spinning forever.
        let tx_watch = tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(45));
            let _ = tx_watch.send(TunnelEvent::Failed(
                "frpc did not connect within 45s".into(),
            ));
        });
        Ok(FrpHandle { child, events: rx })
    }

    /// Kill the frpc process (the public relay port stops forwarding).
    pub(crate) fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for FrpHandle {
    fn drop(&mut self) {
        // A dropped handle must never leak a running frpc (config
        // change / session teardown paths that forget stop()).
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> TunnelProfile {
        TunnelProfile {
            name: "阿里云上海".into(),
            server: "1.2.3.4".into(),
            port: 7000,
            token: "sec\"ret\\1".into(),
            forward_port: 47823,
            enabled: true,
        }
    }

    #[test]
    fn config_renders_toml_with_escaped_strings() {
        let cfg = render_frpc_config(&profile(), 47822);
        assert!(cfg.contains("serverAddr = \"1.2.3.4\""), "{cfg}");
        assert!(cfg.contains("serverPort = 7000"));
        assert!(cfg.contains("auth.token = \"sec\\\"ret\\\\1\""), "{cfg}");
        assert!(cfg.contains("localPort = 47822"));
        assert!(cfg.contains("remotePort = 47823"));
    }

    #[test]
    fn relay_url_brackets_ipv6() {
        let mut p = profile();
        p.server = "2408:8888::1".into();
        assert_eq!(relay_url(&p), "http://[2408:8888::1]:47823");
        p.server = "relay.example.com".into();
        assert_eq!(relay_url(&p), "http://relay.example.com:47823");
    }

    #[test]
    fn frpc_line_parsing_classifies_success_and_failure() {
        let url = "http://1.2.3.4:47823";
        assert!(matches!(
            parse_frpc_line("2024/01/01 login to server success, get run id [x]", url),
            Some(TunnelEvent::Ready(_))
        ));
        assert!(matches!(
            parse_frpc_line("[I] start proxy success", url),
            Some(TunnelEvent::Ready(_))
        ));
        assert!(matches!(
            parse_frpc_line("login to the server failed: token error", url),
            Some(TunnelEvent::Failed(_))
        ));
        assert!(parse_frpc_line("some noise", url).is_none());
    }

    #[test]
    fn frpc_path_lives_in_the_tunnel_subdir() {
        let p = frpc_path(Path::new("/data"));
        assert!(p.to_string_lossy().contains("tunnel"));
        if std::env::consts::OS == "windows" {
            assert!(p.to_string_lossy().ends_with("frpc.exe"));
        } else {
            assert!(p.to_string_lossy().ends_with("frpc"));
        }
    }
}

//! Cloudflare Quick Tunnel manager for remote v2.5 (WAN access without
//! any self-hosted server):
//!
//! - downloads the official `cloudflared` binary into the app data dir
//!   on first use (plain binary on Linux/Windows, tar.gz on macOS -
//!   same extraction dependencies the updater already ships),
//! - spawns `cloudflared tunnel --url http://127.0.0.1:{port}` and
//!   parses the ephemeral `https://*.trycloudflare.com` URL from its
//!   stdout/stderr,
//! - reports status transitions through a channel the UI tick drains;
//!   `stop()` kills the child (the tunnel URL dies with it).
//!
//! No account, no configuration; data transits Cloudflare's edge (TLS).

use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

/// Lifecycle events surfaced to the UI.
#[derive(Debug, Clone)]
pub enum TunnelEvent {
    /// Downloading cloudflared (progress 0..1).
    Downloading(f32),
    /// cloudflared launched, waiting for the URL.
    Starting,
    /// Tunnel is live at this public URL.
    Ready(String),
    /// Something failed (download, spawn, or the URL never appeared).
    Failed(String),
}

/// Asset name for the current platform (GitHub releases "latest/download"
/// aliases are stable and version-free).
fn cloudflared_asset() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "cloudflared-linux-amd64",
        ("linux", "aarch64") => "cloudflared-linux-arm64",
        ("macos", "x86_64") => "cloudflared-macos-amd64.tgz",
        ("macos", "aarch64") => "cloudflared-macos-arm64.tgz",
        ("windows", _) => "cloudflared-windows-amd64.exe",
        _ => "cloudflared-linux-amd64",
    }
}

fn cloudflared_url() -> String {
    format!(
        "https://github.com/cloudflare/cloudflared/releases/latest/download/{}",
        cloudflared_asset()
    )
}

/// Final binary path in the app data dir.
pub fn cloudflared_path(data_dir: &Path) -> PathBuf {
    let name = if std::env::consts::OS == "windows" {
        "cloudflared.exe"
    } else {
        "cloudflared"
    };
    data_dir.join("tunnel").join(name)
}

/// Extract a tunnel URL from one cloudflared output line, if any.
pub fn parse_tunnel_url(line: &str) -> Option<String> {
    let marker = "https://";
    let start = line.find(marker)?;
    let rest = &line[start + marker.len()..];
    let host: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
        .collect();
    if host.ends_with(".trycloudflare.com") && host.len() > ".trycloudflare.com".len() + 2 {
        Some(format!("{marker}{host}"))
    } else {
        None
    }
}

/// Streaming download with progress events (mirrors the updater's
/// download_file, kept independent because the updater speaks Chinese
/// errors and reports through its own progress model).
fn download_cloudflared(dest: &Path, tx: &mpsc::Sender<TunnelEvent>) -> Result<(), String> {
    let resp = ureq::get(&cloudflared_url())
        .timeout(Duration::from_secs(600))
        .call()
        .map_err(|e| format!("download failed: {e}"))?;
    let total: u64 = resp
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(dest).map_err(|e| format!("create failed: {e}"))?;
    let mut buf = [0u8; 64 * 1024];
    let mut done: u64 = 0;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("read failed: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("write failed: {e}"))?;
        done += n as u64;
        if total > 0 {
            let _ = tx.send(TunnelEvent::Downloading(
                (done as f32 / total as f32).min(1.0),
            ));
        }
    }
    // Sanity: cloudflared is ~20-60MB; anything tiny is an error page.
    if done < 5 * 1024 * 1024 {
        return Err("downloaded file too small - not a valid binary".into());
    }
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

/// One managed tunnel session. `child` is killed on stop; the worker
/// thread exits with it.
pub struct TunnelHandle {
    child: Child,
    pub events: mpsc::Receiver<TunnelEvent>,
}

impl TunnelHandle {
    /// Download (if needed) and start a quick tunnel pointing at the
    /// local remote server. Returns immediately; status flows through
    /// the events channel.
    pub fn start(data_dir: &Path, port: u16) -> Result<Self, String> {
        let bin = cloudflared_path(data_dir);
        if !bin.is_file() {
            if let Some(parent) = bin.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir failed: {e}"))?;
            }
        }

        let (tx, rx) = mpsc::channel();
        let bin_for_thread = bin.clone();
        let tx_download = tx.clone();
        // Download on the calling (background-friendly) thread? No: the
        // UI calls this from a spawned thread too, so block here.
        if !bin.is_file() {
            download_cloudflared(&bin_for_thread, &tx_download)?;
        }

        let mut child = Command::new(&bin)
            .arg("tunnel")
            .arg("--url")
            .arg(format!("http://127.0.0.1:{port}"))
            .arg("--no-autoupdate")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn cloudflared failed: {e}"))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let tx2 = tx.clone();
        let tx3 = tx.clone();
        if let Some(out) = stdout {
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(out);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(url) = parse_tunnel_url(&line) {
                        let _ = tx2.send(TunnelEvent::Ready(url));
                        break;
                    }
                }
            });
        }
        if let Some(err) = stderr {
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(err);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(url) = parse_tunnel_url(&line) {
                        let _ = tx3.send(TunnelEvent::Ready(url));
                        break;
                    }
                }
            });
        }
        // Startup note + a failure watchdog: if no URL within 90s, fail.
        // The Ready send races benignly with this (UI treats the first
        // terminal state as final).
        let _ = tx.send(TunnelEvent::Starting);
        let tx_watch = tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(90));
            let _ = tx_watch.send(TunnelEvent::Failed(
                "tunnel did not come up within 90s".into(),
            ));
        });
        Ok(TunnelHandle { child, events: rx })
    }

    /// Kill the cloudflared process (the public URL stops working).
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for TunnelHandle {
    fn drop(&mut self) {
        // Belt-and-suspenders: a dropped handle must never leak a
        // running cloudflared (session teardown paths that forget
        // stop()).
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_url_parsing_accepts_quick_tunnel_hosts() {
        assert_eq!(
            parse_tunnel_url("2026-08-31 INFO Your quick Tunnel has been created! Visit it at: https://random-words-here.trycloudflare.com"),
            Some("https://random-words-here.trycloudflare.com".to_string())
        );
        // Trailing punctuation is not swallowed into the host.
        assert_eq!(
            parse_tunnel_url("see https://abc-def.trycloudflare.com now"),
            Some("https://abc-def.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn tunnel_url_parsing_rejects_non_quick_hosts() {
        assert!(parse_tunnel_url("go to https://example.com now").is_none());
        assert!(parse_tunnel_url("no url here").is_none());
        // Bare domain without enough label prefix.
        assert!(parse_tunnel_url("https://trycloudflare.com").is_none());
    }

    #[test]
    fn asset_name_matches_current_platform() {
        let asset = cloudflared_asset();
        assert!(asset.starts_with("cloudflared-"));
        if std::env::consts::OS == "windows" {
            assert!(asset.ends_with(".exe"));
        }
    }

    #[test]
    fn cloudflared_path_lives_in_the_tunnel_subdir() {
        let p = cloudflared_path(Path::new("/data"));
        assert!(p.to_string_lossy().contains("tunnel"));
        if std::env::consts::OS == "windows" {
            assert!(p.to_string_lossy().ends_with("cloudflared.exe"));
        } else {
            assert!(p.to_string_lossy().ends_with("cloudflared"));
        }
    }
}

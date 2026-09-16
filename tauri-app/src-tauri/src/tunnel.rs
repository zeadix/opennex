//! Cloudflare Quick Tunnel manager for WAN remote access — ported from
//! the egui build (remote v2.5): downloads the official `cloudflared`
//! binary into the app data dir on first use, spawns
//! `cloudflared tunnel --url http://127.0.0.1:{port}` and parses the
//! ephemeral `https://*.trycloudflare.com` URL from its output. No
//! account, no configuration; data transits Cloudflare's edge (TLS).

use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::Manager;

/// Status snapshot polled by the `tunnel_status` command.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    /// idle | downloading | starting | ready | failed
    pub state: String,
    /// Download progress 0..=1 (downloading state).
    pub progress: f32,
    /// Public URL (ready state).
    pub url: Option<String>,
    /// Failure detail (failed state).
    pub error: Option<String>,
}

impl TunnelStatus {
    fn new(state: &str) -> Self {
        Self { state: state.into(), progress: 0.0, url: None, error: None }
    }
}

struct Inner {
    status: Mutex<TunnelStatus>,
    child: Mutex<Option<Child>>,
}

static TUNNEL: Mutex<Option<Arc<Inner>>> = Mutex::new(None);

fn set_status(inner: &Inner, s: TunnelStatus) {
    *inner.status.lock().unwrap() = s;
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
fn cloudflared_path(data_dir: &Path) -> PathBuf {
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

/// Streaming download with progress updates into the shared status.
fn download_cloudflared(dest: &Path, inner: &Arc<Inner>) -> Result<(), String> {
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
        let n = reader.read(&mut buf).map_err(|e| format!("read failed: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| format!("write failed: {e}"))?;
        done += n as u64;
        if total > 0 {
            set_status(
                inner,
                TunnelStatus {
                    state: "downloading".into(),
                    progress: (done as f32 / total as f32).min(1.0),
                    url: None,
                    error: None,
                },
            );
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

/// Start (or reuse) the quick tunnel pointing at the local remote
/// server. Heavy work runs on a worker thread; `tunnel_status` polls.
#[tauri::command]
pub fn tunnel_start(app: tauri::AppHandle) -> Result<TunnelStatus, String> {
    let mut guard = TUNNEL.lock().unwrap();
    if let Some(inner) = guard.as_ref() {
        // Already active (or starting) — just report.
        return Ok(inner.status.lock().unwrap().clone());
    }
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("data dir: {e}"))?;
    let inner = Arc::new(Inner {
        status: Mutex::new(TunnelStatus::new("starting")),
        child: Mutex::new(None),
    });
    *guard = Some(inner.clone());
    drop(guard);

    let initial = inner.status.lock().unwrap().clone();
    std::thread::spawn(move || {
        let bin = cloudflared_path(&data_dir);
        if let Some(parent) = bin.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if !bin.is_file() {
            if let Err(e) = download_cloudflared(&bin, &inner) {
                set_status(
                    &inner,
                    TunnelStatus { state: "failed".into(), progress: 0.0, url: None, error: Some(e) },
                );
                return;
            }
        }
        let child = Command::new(&bin)
            .arg("tunnel")
            .arg("--url")
            .arg(format!("http://127.0.0.1:{}", crate::ws_port_value()))
            .arg("--no-autoupdate")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                set_status(
                    &inner,
                    TunnelStatus {
                        state: "failed".into(),
                        progress: 0.0,
                        url: None,
                        error: Some(format!("spawn cloudflared failed: {e}")),
                    },
                );
                return;
            }
        };
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        *inner.child.lock().unwrap() = Some(child);

        // Either output stream carries the URL; first hit wins.
        let hits = Arc::new(Mutex::new(false));
        let mut readers: Vec<Box<dyn std::io::Read + Send>> = Vec::new();
        if let Some(out) = stdout {
            readers.push(Box::new(out));
        }
        if let Some(err) = stderr {
            readers.push(Box::new(err));
        }
        for r in readers {
            let inner = inner.clone();
            let hits = hits.clone();
            std::thread::spawn(move || {
                let reader = std::io::BufReader::new(r);
                for line in reader.lines().map_while(Result::ok) {
                    if let Some(url) = parse_tunnel_url(&line) {
                        let mut hit = hits.lock().unwrap();
                        if !*hit {
                            *hit = true;
                            set_status(
                                &inner,
                                TunnelStatus {
                                    state: "ready".into(),
                                    progress: 1.0,
                                    url: Some(url),
                                    error: None,
                                },
                            );
                        }
                        break;
                    }
                }
            });
        }
        // Watchdog: if no URL within 90s, fail (the Ready write races
        // benignly; first terminal state wins).
        std::thread::sleep(Duration::from_secs(90));
        let st = inner.status.lock().unwrap().clone();
        if st.state != "ready" {
            set_status(
                &inner,
                TunnelStatus {
                    state: "failed".into(),
                    progress: 0.0,
                    url: None,
                    error: Some("tunnel did not come up within 90s".into()),
                },
            );
        }
    });

    Ok(initial)
}

/// Kill the cloudflared process (the public URL stops working).
#[tauri::command]
pub fn tunnel_stop() -> Result<(), String> {
    if let Some(inner) = TUNNEL.lock().unwrap().as_ref() {
        if let Some(mut child) = inner.child.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        set_status(inner, TunnelStatus::new("idle"));
    }
    Ok(())
}

/// Current tunnel status (polled ~1s by the frontend while active).
#[tauri::command]
pub fn tunnel_status() -> TunnelStatus {
    if let Some(inner) = TUNNEL.lock().unwrap().as_ref() {
        return inner.status.lock().unwrap().clone();
    }
    TunnelStatus::new("idle")
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
    }

    #[test]
    fn tunnel_url_parsing_rejects_non_quick_hosts() {
        assert!(parse_tunnel_url("go to https://example.com now").is_none());
        assert!(parse_tunnel_url("no url here").is_none());
    }
}

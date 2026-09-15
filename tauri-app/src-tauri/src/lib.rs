//! OpenNex Tauri shell — PTY sessions + local WebSocket byte pump.
//!
//! Architecture (VS Code model): the frontend owns the VT parser and
//! renderer (xterm.js + WebGL addon). The Rust side only spawns PTYs
//! and pumps RAW bytes both ways over a localhost WebSocket — no VT
//! parsing here, minimal latency, no per-cell serialization.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::sync::mpsc;
use portable_pty::native_pty_system;
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// One live PTY session. `reader` is taken by the WebSocket task on
/// first attach; `master` is kept for resizes; `kill` breaks the
/// blocking read loop on shutdown.
struct PtySession {
    writer: Mutex<Box<dyn std::io::Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    killer: Mutex<Box<dyn portable_pty::ChildKiller + Send + Sync>>,
    reader: Mutex<Option<Box<dyn std::io::Read + Send>>>,
    kill: Arc<Mutex<bool>>,
}

#[derive(Default)]
struct SessionMap {
    inner: Mutex<HashMap<String, Arc<PtySession>>>,
}

impl SessionMap {
    fn insert(&self, id: String, s: PtySession) {
        self.inner.lock().unwrap().insert(id, Arc::new(s));
    }
    fn get(&self, id: &str) -> Option<Arc<PtySession>> {
        self.inner.lock().unwrap().get(id).cloned()
    }
    fn remove(&self, id: &str) {
        self.inner.lock().unwrap().remove(id);
    }
}

struct AppState {
    sessions: Arc<SessionMap>,
}

static WS_PORT: OnceLock<u16> = OnceLock::new();

/// Global command history, captured from the INPUT path (bytes the user
/// sends are line-buffered; a carriage return commits the line). Newest
/// first, adjacent-dedup, capped.
const HISTORY_CAP: usize = 500;
static HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct LineBuf {
    bytes: Vec<u8>,
}
impl LineBuf {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }
    /// Feed raw input bytes; returns completed lines (utf8-lossy, trimmed).
    fn feed(&mut self, input: &[u8]) -> Vec<String> {
        let mut lines = Vec::new();
        for &b in input {
            match b {
                b'\r' | b'\n' => {
                    if !self.bytes.is_empty() {
                        let line = String::from_utf8_lossy(&self.bytes).trim().to_string();
                        self.bytes.clear();
                        if !line.is_empty() {
                            lines.push(line);
                        }
                    }
                }
                0x7f | 0x08 => {
                    self.bytes.pop();
                }
                0x03 | 0x04 => {
                    self.bytes.clear();
                }
                0x1b => self.bytes.clear(),
                b if b < 0x20 => {}
                _ => self.bytes.push(b),
            }
        }
        lines
    }
}

fn record_line(line: String) {
    let mut hist = HISTORY.lock().unwrap();
    if hist.first().map(|h| h == &line).unwrap_or(false) {
        return;
    }
    if let Some(pos) = hist.iter().position(|h| h == &line) {
        hist.remove(pos);
    }
    hist.insert(0, line);
    if hist.len() > HISTORY_CAP {
        hist.truncate(HISTORY_CAP);
    }
}

fn spawn_pty(
    session_id: String,
    sessions: Arc<SessionMap>,
    cols: u16,
    rows: u16,
    command: Option<Vec<String>>,
    shell: Option<String>,
) -> Result<(), String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("openpty: {e}"))?;
    let mut cmd = match &command {
        Some(args) if !args.is_empty() => CommandBuilder::new(&args[0]),
        _ => CommandBuilder::new(
            shell.clone().unwrap_or_else(|| {
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into())
            }),
        ),
    };
    if let Some(args) = &command {
        for a in args.iter().skip(1) {
            cmd.arg(a);
        }
    } else {
        cmd.arg("-l");
    }
    cmd.cwd(std::env::var("HOME").unwrap_or_else(|_| "/".into()));
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("spawn: {e}"))?;
    // Killer first (child is moved into the waiter thread below);
    // pump_reader's EOF sentinel already notifies the frontend on exit.
    let killer = child.clone_killer();
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("clone reader: {e}"))?;

    sessions.insert(
        session_id,
        PtySession {
            writer: Mutex::new(pair.master.take_writer().map_err(|e| format!("writer: {e}"))?),
            master: Mutex::new(pair.master),
            killer: Mutex::new(killer),
            reader: Mutex::new(Some(reader)),
            kill: Arc::new(Mutex::new(false)),
        },
    );
    // Waiter thread: reap the child on exit.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Blocking PTY -> channel pump, run inside spawn_blocking.
fn pump_reader(
    mut reader: Box<dyn std::io::Read + Send>,
    out_tx: mpsc::UnboundedSender<Vec<u8>>,
    kill: Arc<Mutex<bool>>,
) {
    let mut buf = [0u8; 65536];
    loop {
        if *kill.lock().unwrap() {
            break;
        }
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if out_tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    // Session ended: a sentinel lets the frontend print the exit state.
    let _ = out_tx.send(b"\x1b]777;session-exit\x07".to_vec());
}

async fn session_ws(mut socket: WebSocket, sessions: Arc<SessionMap>, sid: String) {
    let Some(session) = sessions.get(&sid) else {
        let _ = socket
            .send(Message::Text("{\"t\":\"error\",\"msg\":\"no session\"}".into()))
            .await;
        return;
    };
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let line_buf = Arc::new(Mutex::new(LineBuf::new()));

    // PTY -> WS pump.
    if let Some(reader) = session.reader.lock().unwrap().take() {
        let tx = out_tx.clone();
        let kill = session.kill.clone();
        tokio::task::spawn_blocking(move || pump_reader(reader, tx, kill));
    }

    // select: PTY output -> socket ; socket messages -> PTY.
    loop {
        tokio::select! {
            msg = out_rx.recv() => {
                match msg {
                    Some(bytes) => {
                        // Coalesce bursts: drain everything already queued
                        // (bounded at 256KB) into ONE websocket frame. This
                        // cuts frame counts dramatically on high-throughput
                        // output without adding any waiting delay.
                        let mut batch = bytes;
                        while batch.len() < 256 * 1024 {
                            match out_rx.try_recv() {
                                Ok(more) => batch.extend_from_slice(&more),
                                Err(_) => break,
                            }
                        }
                        if socket.send(Message::Binary(batch.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            maybe_msg = socket.recv() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        // Control messages: {"type":"resize","cols":N,"rows":N}
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                            if v["type"] == "resize" {
                                let cols = v["cols"].as_u64().unwrap_or(80) as u16;
                                let rows = v["rows"].as_u64().unwrap_or(24) as u16;
                                let _ = session.master.lock().unwrap().resize(PtySize {
                                    rows, cols, pixel_width: 0, pixel_height: 0,
                                });
                            }
                        }
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        for line in line_buf.lock().unwrap().feed(&bytes) {
                            record_line(line);
                        }
                        if session.writer.lock().unwrap().write_all(&bytes).is_err() {
                            break;
                        }
                        let _ = session.writer.lock().unwrap().flush();
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
    // Cleanup: kill child (already exited in the common case) and drop.
    *session.kill.lock().unwrap() = true;
    let _ = session.killer.lock().unwrap().kill();
    sessions.remove(&sid);
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<HashMap<String, String>>,
    State(sessions): State<Arc<SessionMap>>,
) -> axum::response::Response {
    match q.get("session") {
        Some(sid) => {
            let sid = sid.clone();
            ws.on_upgrade(move |socket| session_ws(socket, sessions, sid))
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

async fn spawn_ws_server(sessions: Arc<SessionMap>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind 127.0.0.1");
    let port = listener.local_addr().unwrap().port();
    let _ = WS_PORT.set(port);
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(sessions);
    axum::serve(listener, app).await.expect("ws server");
}

#[tauri::command]
fn start_terminal(
    state: tauri::State<AppState>,
    cols: u16,
    rows: u16,
    command: Option<Vec<String>>,
    shell: Option<String>,
) -> Result<serde_json::Value, String> {
    let session_id = uuid::Uuid::new_v4().simple().to_string();
    spawn_pty(
        session_id.clone(),
        state.sessions.clone(),
        cols,
        rows,
        command,
        shell,
    )?;
    let ws_port = WS_PORT.get().copied().unwrap_or(0);
    Ok(json!({ "session": session_id, "wsPort": ws_port }))
}

/// Shells available on this machine (from /etc/shells, deduped).
#[tauri::command]
fn list_shells() -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_to_string("/etc/shells")
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect();
    out.sort();
    out.dedup();
    if out.is_empty() {
        out.push("/bin/bash".into());
    }
    out
}

#[tauri::command]
fn ws_port() -> u16 {
    WS_PORT.get().copied().unwrap_or(0)
}

#[tauri::command]
fn get_history() -> Vec<String> {
    HISTORY.lock().unwrap().clone()
}

#[derive(serde::Deserialize)]
struct ChatMsg {
    role: String,
    content: String,
}

#[tauri::command]
async fn ai_chat(
    base_url: String,
    api_key: String,
    model: String,
    messages: Vec<ChatMsg>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "messages": messages
                .iter()
                .map(|m| json!({"role": m.role, "content": m.content}))
                .collect::<Vec<_>>(),
        });
        let mut req = ureq::post(&url)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(120));
        if !api_key.is_empty() {
            req = req.set("Authorization", &format!("Bearer {api_key}"));
        }
        let resp = req.send_json(body).map_err(|e| format!("request failed: {e}"))?;
        let v: serde_json::Value = resp.into_json().map_err(|e| format!("parse failed: {e}"))?;
        v["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "empty response".into())
    })
    .await
    .map_err(|e| format!("task failed: {e}"))?
}

pub fn run() {
    let sessions: Arc<SessionMap> = Arc::default();
    let state = AppState {
        sessions: sessions.clone(),
    };
    // Dedicated runtime thread for the WS server (Tauri manages its own).
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(spawn_ws_server(sessions));
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            start_terminal,
            ws_port,
            list_shells,
            get_history,
            ai_chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

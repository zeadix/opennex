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

fn spawn_pty(
    session_id: String,
    sessions: Arc<SessionMap>,
    cols: u16,
    rows: u16,
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
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
    let mut cmd = CommandBuilder::new(&shell);
    cmd.arg("-l");
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
    let mut buf = [0u8; 16384];
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
                        if socket.send(Message::Binary(bytes.into())).await.is_err() {
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
) -> Result<serde_json::Value, String> {
    let session_id = uuid::Uuid::new_v4().simple().to_string();
    spawn_pty(session_id.clone(), state.sessions.clone(), cols, rows)?;
    let ws_port = WS_PORT.get().copied().unwrap_or(0);
    Ok(json!({ "session": session_id, "wsPort": ws_port }))
}

#[tauri::command]
fn ws_port() -> u16 {
    WS_PORT.get().copied().unwrap_or(0)
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
        .invoke_handler(tauri::generate_handler![start_terminal, ws_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

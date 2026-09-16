//! OpenNex Tauri shell — PTY sessions + local WebSocket byte pump.
//!
//! Architecture (VS Code model): the frontend owns the VT parser and
//! renderer (xterm.js + WebGL addon). The Rust side only spawns PTYs
//! and pumps RAW bytes both ways over a localhost WebSocket — no VT
//! parsing here, minimal latency, no per-cell serialization.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use tokio::sync::mpsc;
use portable_pty::native_pty_system;
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::OnceLock as StdOnceLock;
static SYS: StdOnceLock<Mutex<sysinfo::System>> = StdOnceLock::new();

/// One live PTY session. The reader pump runs ONCE per session and
/// fans output out through `bcast`; `scrollback` replays to every new
/// attach. Sessions are DETACH-SAFE: a closed WebSocket (workspace
/// switch, layout change) keeps the shell running — only `close_session`
/// (tab close) ends it. `master` is kept for resizes; `kill` breaks the
/// blocking read loop on shutdown.
struct PtySession {
    writer: Mutex<Box<dyn std::io::Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    killer: Mutex<Box<dyn portable_pty::ChildKiller + Send + Sync>>,
    kill: Arc<Mutex<bool>>,
    /// Output fan-out; each attached WebSocket holds a Subscriber.
    bcast: tokio::sync::broadcast::Sender<Vec<u8>>,
    /// Capped tail of raw output bytes, replayed on (re)attach.
    scrollback: Arc<Mutex<Vec<u8>>>,
    /// Last activity (input OR output) in unix ms — feeds the workspace
    /// busy indicator (red = active within 10s, green = idle).
    last_activity_ms: Arc<AtomicU64>,
}

/// Scrollback kept per session for reattach replay (~512KB of raw VT
/// bytes is plenty to rebuild the visible screen).
const SCROLLBACK_CAP: usize = 512 * 1024;

#[derive(Default)]
struct SessionMap {
    inner: Mutex<HashMap<String, Arc<PtySession>>>,
    order: Mutex<Vec<(String, String)>>, // (id, display name), insertion order
}

impl SessionMap {
    fn insert_named(&self, id: String, name: String, s: PtySession) {
        self.order.lock().unwrap().push((id.clone(), name));
        self.inner.lock().unwrap().insert(id, Arc::new(s));
    }
    /// Live sessions (id + display name) in insertion order — feeds the
    /// remote page's session picker.
    fn names(&self) -> Vec<serde_json::Value> {
        self.order
            .lock()
            .unwrap()
            .iter()
            .map(|(id, name)| json!({ "id": id, "name": name }))
            .collect()
    }
    fn insert(&self, id: String, s: PtySession) {
        self.inner.lock().unwrap().insert(id, Arc::new(s));
    }
    fn get(&self, id: &str) -> Option<Arc<PtySession>> {
        self.inner.lock().unwrap().get(id).cloned()
    }
    fn remove(&self, id: &str) {
        self.inner.lock().unwrap().remove(id);
        self.order.lock().unwrap().retain(|(x, _)| x != id);
    }
}

struct AppState {
    sessions: Arc<SessionMap>,
}

static WS_PORT: OnceLock<u16> = OnceLock::new();

/// Local remote-server port for the tunnel module (0 until the server
/// is up — callers treat that as "not ready").
pub fn ws_port_value() -> u16 {
    WS_PORT.get().copied().unwrap_or(0)
}

pub mod tunnel;

/// Remote-control page assets, embedded at compile time (zero network
/// dependencies at runtime — mirrors the egui build's approach).
const REMOTE_HTML: &[u8] = include_bytes!("../remote/remote.html");
const REMOTE_XTERM_JS: &[u8] = include_bytes!("../remote/xterm.js");
const REMOTE_XTERM_CSS: &[u8] = include_bytes!("../remote/xterm.css");
const REMOTE_FIT_JS: &[u8] = include_bytes!("../remote/fit.js");

/// Best-effort LAN IPv4 (UDP connect trick — no packets actually sent).
fn lan_ipv4() -> Option<String> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("8.8.8.8:80").ok()?;
    Some(s.local_addr().ok()?.ip().to_string())
}

/// Global command history, captured from the INPUT path (bytes the user
/// sends are line-buffered; a carriage return commits the line). Newest
/// first, adjacent-dedup, capped.
const HISTORY_CAP_DEFAULT: usize = 500;
static HISTORY_CAP: Mutex<usize> = Mutex::new(HISTORY_CAP_DEFAULT);
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
    let cap = *HISTORY_CAP.lock().unwrap();
    if hist.len() > cap {
        hist.truncate(cap);
    }
}

fn spawn_pty(
    session_id: String,
    display_name: String,
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

    let last_activity_ms = Arc::new(AtomicU64::new(0));
    let (bcast, _) = tokio::sync::broadcast::channel::<Vec<u8>>(256);
    let scrollback: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::with_capacity(64 * 1024)));
    sessions.insert_named(
        session_id.clone(),
        display_name,
        PtySession {
            writer: Mutex::new(pair.master.take_writer().map_err(|e| format!("writer: {e}"))?),
            master: Mutex::new(pair.master),
            killer: Mutex::new(killer),
            kill: Arc::new(Mutex::new(false)),
            bcast,
            scrollback: scrollback.clone(),
            last_activity_ms,
        },
    );
    // Output pump: exactly ONE per session, started at spawn — attached
    // sockets come and go, the shell (and this pump) keeps running.
    let s = sessions.get(&session_id).expect("just inserted");
    std::thread::spawn(move || {
        pump_reader(reader, s.bcast.clone(), scrollback, s.kill.clone(), s.last_activity_ms.clone());
    });
    // Waiter thread: reap the child on exit.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Blocking PTY -> broadcast pump, run once per session on a worker
/// thread. Every chunk appends to the scrollback tail and fans out to
/// all currently-attached WebSockets.
fn pump_reader(
    mut reader: Box<dyn std::io::Read + Send>,
    bcast: tokio::sync::broadcast::Sender<Vec<u8>>,
    scrollback: Arc<Mutex<Vec<u8>>>,
    kill: Arc<Mutex<bool>>,
    activity: Arc<AtomicU64>,
) {
    let mut buf = [0u8; 65536];
    loop {
        if *kill.lock().unwrap() {
            break;
        }
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                activity.store(unix_ms(), Ordering::Relaxed);
                let chunk = buf[..n].to_vec();
                {
                    let mut sb = scrollback.lock().unwrap();
                    sb.extend_from_slice(&chunk);
                    let len = sb.len();
                    if len > SCROLLBACK_CAP {
                        sb.drain(..len - SCROLLBACK_CAP);
                    }
                }
                if bcast.send(chunk).is_err() {
                    // No attach right now — keep reading, the session
                    // stays alive and scrollback accumulates.
                }
            }
            Err(_) => break,
        }
    }
    // Session ended: a sentinel lets the frontend print the exit state.
    let _ = bcast.send(b"\x1b]777;session-exit\x07".to_vec());
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

    // (Re)attach: replay the scrollback tail, then bridge live output.
    // Subscribe WHILE holding the scrollback lock — the pump appends
    // under that same lock before broadcasting, so snapshot + live
    // stream have neither a gap nor an overlap.
    {
        let sb = session.scrollback.lock().unwrap();
        let mut rx = session.bcast.subscribe();
        let _ = out_tx.send(sb.clone());
        drop(sb);
        let tx = out_tx.clone();
        tokio::task::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(chunk) => {
                        if tx.send(chunk).is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });
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
                        session
                            .last_activity_ms
                            .store(unix_ms(), std::sync::atomic::Ordering::Relaxed);
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
    // Detach ONLY: the WebSocket going away (workspace switch, layout
    // drag, floating-window focus changes that remount panes) must NOT
    // end the session — the shell keeps running and its scrollback
    // accumulates for the next attach. Ending a session is explicit:
    // the frontend calls close_session when its tab is closed.
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

fn serve_bytes(bytes: &'static [u8], content_type: &'static str) -> Response {
    (
        [(axum::http::header::CONTENT_TYPE, content_type)],
        bytes.to_vec(),
    )
        .into_response()
}

async fn remote_sessions(State(sessions): State<Arc<SessionMap>>) -> Response {
    let names = sessions.names();
    Json(json!({ "sessions": names })).into_response()
}

async fn spawn_ws_server(sessions: Arc<SessionMap>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind 127.0.0.1");
    let port = listener.local_addr().unwrap().port();
    let _ = WS_PORT.set(port);
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/remote", get(|| async {
            Html(String::from_utf8_lossy(REMOTE_HTML).into_owned()).into_response()
        }))
        .route("/remote/xterm.js", get(|| async {
            serve_bytes(REMOTE_XTERM_JS, "application/javascript")
        }))
        .route("/remote/xterm.css", get(|| async {
            serve_bytes(REMOTE_XTERM_CSS, "text/css")
        }))
        .route("/remote/fit.js", get(|| async {
            serve_bytes(REMOTE_FIT_JS, "application/javascript")
        }))
        .route("/remote/sessions", get(remote_sessions))
        .with_state(sessions);
    axum::serve(listener, app).await.expect("ws server");
}

/// Block briefly until the WS server thread has bound its port (a
/// terminal created in the first milliseconds of app startup would
/// otherwise receive wsPort=0 and connect to nothing).
fn wait_ws_port() -> u16 {
    for _ in 0..100 {
        if let Some(p) = WS_PORT.get() {
            return *p;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    WS_PORT.get().copied().unwrap_or(0)
}

#[tauri::command]
fn start_terminal(
    state: tauri::State<AppState>,
    cols: u16,
    rows: u16,
    command: Option<Vec<String>>,
    shell: Option<String>,
    name: Option<String>,
    session_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let session_id = session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let display_name = name.unwrap_or_else(|| "bash".into());
    // Reattach path: the session already exists (workspace switch
    // remounted the pane) — keep the running shell, just hand back the
    // WS port; the socket replays scrollback into the fresh xterm.
    if state.sessions.get(&session_id).is_some() {
        let ws_port = wait_ws_port();
        return Ok(json!({ "session": session_id, "wsPort": ws_port, "name": display_name }));
    }
    spawn_pty(
        session_id.clone(),
        display_name.clone(),
        state.sessions.clone(),
        cols,
        rows,
        command,
        shell,
    )?;
    let ws_port = wait_ws_port();
    Ok(json!({ "session": session_id, "wsPort": ws_port, "name": display_name }))
}

/// Explicitly end a session (its terminal tab was closed). Detached
/// sessions are otherwise kept alive for reattach.
#[tauri::command]
fn close_session(state: tauri::State<AppState>, session_id: String) -> Result<(), String> {
    if let Some(s) = state.sessions.get(&session_id) {
        *s.kill.lock().unwrap() = true;
        let _ = s.killer.lock().unwrap().kill();
    }
    state.sessions.remove(&session_id);
    Ok(())
}

/// Executables reachable via PATH as bare names (sorted, deduped) —
/// feeds the auto-match suggestion list (port of egui's completion.rs).
#[tauri::command]
fn list_path_commands() -> Vec<String> {
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let Ok(ft) = entry.file_type() else { continue };
                if !ft.is_file() {
                    continue;
                }
                let raw = entry.file_name().to_string_lossy().into_owned();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let executable = entry
                        .metadata()
                        .map(|m| m.permissions().mode() & 0o111 != 0)
                        .unwrap_or(false);
                    if !executable {
                        continue;
                    }
                }
                if !raw.is_empty() {
                    names.insert(raw);
                }
            }
        }
    }
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    sorted
}

/// Remote-control endpoint info for the in-app QR view.
#[tauri::command]
fn remote_info() -> serde_json::Value {
    let port = WS_PORT.get().copied().unwrap_or(0);
    let ip = lan_ipv4().unwrap_or_else(|| "127.0.0.1".into());
    json!({ "url": format!("http://{ip}:{port}/remote"), "lanIp": ip, "port": port })
}

/// System stats for the monitor page (CPU% + memory), via sysinfo.
#[tauri::command]
async fn system_stats(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let sessions = state.sessions.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(sys_mutex) = SYS.get() else {
            return Err("sys not initialized".into());
        };
        let mut sys = sys_mutex.lock().unwrap();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        let cpu = sys.global_cpu_usage();
        let mem_used = sys.used_memory();
        let mem_total = sys.total_memory();
        let session_count = sessions.inner.lock().unwrap().len();
        Ok(json!({
            "cpuPct": cpu,
            "memUsedGb": mem_used as f64 / 1024.0 / 1024.0 / 1024.0,
            "memTotalGb": mem_total as f64 / 1024.0 / 1024.0 / 1024.0,
            "sessions": session_count,
        }))
    })
    .await
    .map_err(|e| format!("task failed: {e}"))?
}

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

fn unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Per-session last-activity timestamps for the workspace busy indicator.
#[tauri::command]
fn session_activities(state: tauri::State<AppState>) -> serde_json::Value {
    let sessions = state.sessions.inner.lock().unwrap();
    let mut out = serde_json::Map::new();
    for (id, s) in sessions.iter() {
        out.insert(
            id.clone(),
            json!(s.last_activity_ms.load(std::sync::atomic::Ordering::Relaxed)),
        );
    }
    json!(out)
}

#[tauri::command]
fn set_history_cap(cap: usize) {
    *HISTORY_CAP.lock().unwrap() = cap.clamp(10, 10_000);
    let mut hist = HISTORY.lock().unwrap();
    hist.truncate(*HISTORY_CAP.lock().unwrap());
}

#[tauri::command]
fn clear_history() {
    HISTORY.lock().unwrap().clear();
}

#[derive(serde::Deserialize)]
struct ChatMsg {
    role: String,
    content: String,
}

#[derive(serde::Deserialize)]
struct LatestManifest {
    version: String,
    #[serde(default)]
    changes: Vec<String>,
    #[serde(default)]
    changes_en: Vec<String>,
}

/// Update check: fetch the public manifest, compare versions, return the
/// newer release's bilingual notes (no download/install in v1).
#[tauri::command]
async fn check_update() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let resp = ureq::get("https://opennex.download.zeadix.com/latest.json")
            .timeout(std::time::Duration::from_secs(15))
            .call()
            .map_err(|e| format!("request failed: {e}"))?;
        let m: LatestManifest = resp.into_json().map_err(|e| format!("parse failed: {e}"))?;
        let current = env!("CARGO_PKG_VERSION");
        let newer = version_newer(&m.version, current);
        Ok(json!({
            "updateAvailable": newer,
            "latest": m.version,
            "current": current,
            "changes": m.changes,
            "changesEn": m.changes_en,
        }))
    })
    .await
    .map_err(|e| format!("task failed: {e}"))?
}

fn version_newer(remote: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .filter_map(|n| n.parse::<u64>().ok())
            .collect()
    };
    let (r, c) = (parse(remote), parse(current));
    r > c
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
    let _ = SYS.set(Mutex::new(sysinfo::System::new()));
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
            close_session,
            ws_port,
            list_shells,
            list_path_commands,
            remote_info,
            get_history,
            ai_chat,
            check_update,
            system_stats,
            session_activities,
            set_history_cap,
            clear_history,
            tunnel::tunnel_start,
            tunnel::tunnel_stop,
            tunnel::tunnel_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

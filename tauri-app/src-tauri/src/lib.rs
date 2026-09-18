//! OpenNex Tauri shell — PTY sessions + local WebSocket byte pump.
//!
//! Architecture (VS Code model): the frontend owns the VT parser and
//! renderer (xterm.js + WebGL addon). The Rust side only spawns PTYs
//! and pumps RAW bytes both ways over a localhost WebSocket — no VT
//! parsing here, minimal latency, no per-cell serialization.

mod activity;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{ConnectInfo, Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use portable_pty::native_pty_system;
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::OnceLock as StdOnceLock;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;
static SYS: StdOnceLock<Mutex<sysinfo::System>> = StdOnceLock::new();

/// One live PTY session. The reader pump runs ONCE per session and
/// fans output out through `bcast`; `scrollback` replays to every new
/// attach. Sessions are DETACH-SAFE: a closed WebSocket (workspace
/// switch, layout change) keeps the shell running — only `close_session`
/// (tab close) ends it. `master` is kept for resizes; `kill` breaks the
/// blocking read loop on shutdown.
struct PtySession {
    /// Immutable owner: reattaching must never move a session between workspaces.
    workspace_id: u64,
    writer: Mutex<Box<dyn std::io::Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    killer: Mutex<Box<dyn portable_pty::ChildKiller + Send + Sync>>,
    kill: Arc<Mutex<bool>>,
    /// Shell child pid — root of this terminal's process tree for the
    /// resource monitor.
    pid: u32,
    /// Output fan-out; each attached WebSocket holds a Subscriber.
    bcast: tokio::sync::broadcast::Sender<Vec<u8>>,
    /// Capped tail of raw output bytes, replayed on (re)attach.
    scrollback: Arc<Mutex<Vec<u8>>>,
    /// Zero until the first submitted input; startup output never arms
    /// activity. Once armed, command output refreshes the activity clock.
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

/// Random gate for the remote-control endpoints: the WS server must bind
/// 0.0.0.0 for LAN/WAN access, so every session-driving request must
/// carry this token (?k=...) from the QR'd URL.
static REMOTE_TOKEN: OnceLock<String> = OnceLock::new();
fn remote_token() -> &'static str {
    REMOTE_TOKEN.get().map(|s| s.as_str()).unwrap_or("")
}

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

/// Input history is deduplicated, ranked and capped within each workspace.
const HISTORY_CAP_DEFAULT: usize = 500;

struct HistEntry {
    id: u64,
    cmd: String,
    hits: u32,
}

struct WorkspaceHistory {
    entries: Vec<HistEntry>,
    cap: usize,
}

impl Default for WorkspaceHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            cap: HISTORY_CAP_DEFAULT,
        }
    }
}

#[derive(Default)]
struct HistoryStore {
    workspaces: HashMap<u64, WorkspaceHistory>,
    next_id: u64,
}

impl HistoryStore {
    fn record(&mut self, workspace_id: u64, line: String) {
        let hist = self.workspaces.entry(workspace_id).or_default();
        let entry = if let Some(pos) = hist.entries.iter().position(|e| e.cmd == line) {
            let mut entry = hist.entries.remove(pos);
            entry.hits = entry.hits.saturating_add(1);
            entry
        } else {
            self.next_id += 1;
            HistEntry {
                id: self.next_id,
                cmd: line,
                hits: 1,
            }
        };
        hist.entries.insert(0, entry);
        hist.entries.truncate(hist.cap);
    }

    fn get(&self, workspace_id: u64) -> &[HistEntry] {
        self.workspaces
            .get(&workspace_id)
            .map(|h| h.entries.as_slice())
            .unwrap_or(&[])
    }

    fn delete(&mut self, workspace_id: u64, id: u64) -> bool {
        let Some(hist) = self.workspaces.get_mut(&workspace_id) else {
            return false;
        };
        let before = hist.entries.len();
        hist.entries.retain(|e| e.id != id);
        hist.entries.len() != before
    }

    fn set_cap(&mut self, workspace_id: u64, cap: usize) {
        let hist = self.workspaces.entry(workspace_id).or_default();
        hist.cap = cap.clamp(10, 10_000);
        hist.entries.truncate(hist.cap);
    }

    fn clear(&mut self, workspace_id: u64) {
        if let Some(hist) = self.workspaces.get_mut(&workspace_id) {
            hist.entries.clear(); // Keep this workspace's configured capacity.
        }
    }
}

static HISTORY: OnceLock<Mutex<HistoryStore>> = OnceLock::new();
fn history() -> &'static Mutex<HistoryStore> {
    HISTORY.get_or_init(|| Mutex::new(HistoryStore::default()))
}

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

fn record_line(workspace_id: u64, line: String) {
    history().lock().unwrap().record(workspace_id, line);
}

fn spawn_pty(
    session_id: String,
    workspace_id: u64,
    display_name: String,
    sessions: Arc<SessionMap>,
    cols: u16,
    rows: u16,
    command: Option<Vec<String>>,
    shell: Option<String>,
    cwd: Option<String>,
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
    // Cross-platform default shell: $SHELL on Unix, PowerShell on Windows
    // ("/bin/bash" would fail to spawn there).
    #[cfg(windows)]
    const FALLBACK_SHELL: &str = "powershell.exe";
    #[cfg(not(windows))]
    const FALLBACK_SHELL: &str = "/bin/bash";
    let default_shell = shell
        .clone()
        .filter(|sh| !sh.is_empty())
        .or_else(|| std::env::var("SHELL").ok())
        .filter(|sh| !sh.is_empty())
        .unwrap_or_else(|| FALLBACK_SHELL.into());
    let mut cmd = match &command {
        Some(args) if !args.is_empty() => CommandBuilder::new(&args[0]),
        _ => CommandBuilder::new(default_shell),
    };
    if let Some(args) = &command {
        for a in args.iter().skip(1) {
            cmd.arg(a);
        }
    } else {
        // Login shell flag is a Unix convention.
        #[cfg(not(windows))]
        cmd.arg("-l");
    }
    // Restored per-terminal working directory (workspace path memory);
    // silently fall back to HOME when the dir no longer exists.
    if let Some(dir) = cwd.as_deref() {
        if std::path::Path::new(dir).is_dir() {
            cmd.cwd(dir);
        } else {
            cmd.cwd(std::env::var("HOME").unwrap_or_else(|_| "/".into()));
        }
    } else {
        cmd.cwd(std::env::var("HOME").unwrap_or_else(|_| "/".into()));
    }
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
    let child_pid = child.process_id().unwrap_or(0);
    sessions.insert_named(
        session_id.clone(),
        display_name,
        PtySession {
            workspace_id,
            writer: Mutex::new(
                pair.master
                    .take_writer()
                    .map_err(|e| format!("writer: {e}"))?,
            ),
            master: Mutex::new(pair.master),
            killer: Mutex::new(killer),
            kill: Arc::new(Mutex::new(false)),
            pid: child_pid,
            bcast,
            scrollback: scrollback.clone(),
            last_activity_ms,
        },
    );
    // Output pump: exactly ONE per session, started at spawn — attached
    // sockets come and go, the shell (and this pump) keeps running.
    let s = sessions.get(&session_id).expect("just inserted");
    std::thread::spawn(move || {
        pump_reader(
            reader,
            s.bcast.clone(),
            scrollback,
            s.kill.clone(),
            s.last_activity_ms.clone(),
        );
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
                activity::record_output(&activity, unix_ms());
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

#[cfg(test)]
mod startup_activity_tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn prompt_is_delivered_without_marking_startup_busy() {
        let prompt = b"user@host:~/project$ ";
        let activity = Arc::new(AtomicU64::new(0));
        let scrollback = Arc::new(Mutex::new(Vec::new()));
        let (sender, mut receiver) = tokio::sync::broadcast::channel(8);
        pump_reader(
            Box::new(std::io::Cursor::new(prompt.to_vec())),
            sender,
            scrollback.clone(),
            Arc::new(Mutex::new(false)),
            activity.clone(),
        );
        assert_eq!(activity.load(Ordering::Relaxed), 0);
        assert_eq!(*scrollback.lock().unwrap(), prompt);
        assert_eq!(receiver.try_recv().unwrap(), prompt);
    }
}

async fn session_ws(mut socket: WebSocket, sessions: Arc<SessionMap>, sid: String) {
    let Some(session) = sessions.get(&sid) else {
        let _ = socket
            .send(Message::Text(
                "{\"t\":\"error\",\"msg\":\"no session\"}".into(),
            ))
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
                        // Only a CONFIRMED line counts as activity for the
                        // busy indicator: typing alone (chars flowing into
                        // readline) is not "busy" until Enter commits it.
                        // 0x03 (^C) / 0x04 (^D) also commit/interrupt.
                        if bytes.contains(&b'\r')
                            || bytes.contains(&b'\n')
                            || bytes.contains(&0x03)
                            || bytes.contains(&0x04)
                        {
                            session
                                .last_activity_ms
                                .store(unix_ms(), std::sync::atomic::Ordering::Relaxed);
                        }
                        for line in line_buf.lock().unwrap().feed(&bytes) {
                            record_line(session.workspace_id, line);
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
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(sessions): State<Arc<SessionMap>>,
) -> axum::response::Response {
    // Loopback (the app itself) is trusted; remote clients need the token.
    let loopback = addr.ip().is_loopback();
    if !loopback && q.get("k").map(|x| x.as_str()) != Some(remote_token()) {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }
    match q.get("session") {
        Some(sid) => {
            let sid = sid.clone();
            ws.on_upgrade(move |socket| session_ws(socket, sessions, sid))
        }
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

/// Bare tunnel URL (/) → the remote page, preserving the access token.
async fn root_redirect(Query(q): Query<HashMap<String, String>>) -> Response {
    let k = q.get("k").map(|s| s.as_str()).unwrap_or("");
    axum::response::Redirect::to(&format!("/remote?k={}", k)).into_response()
}

fn serve_bytes(bytes: &'static [u8], content_type: &'static str) -> Response {
    (
        [(axum::http::header::CONTENT_TYPE, content_type)],
        bytes.to_vec(),
    )
        .into_response()
}

async fn remote_sessions(
    Query(q): Query<HashMap<String, String>>,
    State(sessions): State<Arc<SessionMap>>,
) -> Response {
    if q.get("k").map(|x| x.as_str()) != Some(remote_token()) {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }
    let names = sessions.names();
    Json(json!({ "sessions": names })).into_response()
}

async fn spawn_ws_server(sessions: Arc<SessionMap>) {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0")
        .await
        .expect("bind 127.0.0.1");
    let port = listener.local_addr().unwrap().port();
    let _ = WS_PORT.set(port);
    let app = Router::new()
        .route("/", get(root_redirect))
        .route("/ws", get(ws_handler))
        .route(
            "/remote",
            get(|| async {
                Html(String::from_utf8_lossy(REMOTE_HTML).into_owned()).into_response()
            }),
        )
        .route(
            "/remote/xterm.js",
            get(|| async { serve_bytes(REMOTE_XTERM_JS, "application/javascript") }),
        )
        .route(
            "/remote/xterm.css",
            get(|| async { serve_bytes(REMOTE_XTERM_CSS, "text/css") }),
        )
        .route(
            "/remote/fit.js",
            get(|| async { serve_bytes(REMOTE_FIT_JS, "application/javascript") }),
        )
        .route("/remote/sessions", get(remote_sessions))
        .with_state(sessions);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("ws server");
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
    cwd: Option<String>,
    workspace_id: u64,
) -> Result<serde_json::Value, String> {
    let session_id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string());
    let display_name = name.unwrap_or_else(|| "bash".into());
    // Reattach path: the session already exists (workspace switch
    // remounted the pane) — keep the running shell, just hand back the
    // WS port; the socket replays scrollback into the fresh xterm.
    // Ownership is verified so a stale config can never reattach a
    // terminal into (and record history under) the wrong workspace.
    if let Some(existing) = state.sessions.get(&session_id) {
        if existing.workspace_id != workspace_id {
            return Err(format!(
                "session {session_id} belongs to workspace {}, not {workspace_id}",
                existing.workspace_id
            ));
        }
        let ws_port = wait_ws_port();
        return Ok(json!({ "session": session_id, "wsPort": ws_port, "name": display_name }));
    }
    spawn_pty(
        session_id.clone(),
        workspace_id,
        display_name.clone(),
        state.sessions.clone(),
        cols,
        rows,
        command,
        shell,
        cwd,
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

/// The session shell's current working directory (/proc pid cwd) —
/// feeds the workspace path memory (per-terminal cwd restore).
#[tauri::command]
fn session_cwd(state: tauri::State<AppState>, session_id: String) -> Option<String> {
    let s = state.sessions.get(&session_id)?;
    let cwd = std::fs::read_link(format!("/proc/{}/cwd", s.pid)).ok()?;
    Some(cwd.to_string_lossy().into_owned())
}

/// Locally installed font families (fontconfig) — feeds the WYSIWYG
/// font pickers so users can pick ANY font on this machine.
#[tauri::command]
fn list_fonts() -> Vec<String> {
    let Ok(out) = std::process::Command::new("fc-list")
        .args([":", "family"])
        .output()
    else {
        return Vec::new();
    };
    let mut names: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .flat_map(|l| l.split(','))
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect();
    names.sort_by_key(|a| a.to_lowercase());
    names.dedup();
    names
}

/// Executables reachable via PATH as bare names (sorted, deduped) —
/// feeds the auto-match suggestion list (port of egui's completion.rs).
#[tauri::command]
fn list_path_commands() -> Vec<String> {
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
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
    json!({
        "url": format!("http://{ip}:{port}/remote?k={}", remote_token()),
        "lanIp": ip,
        "port": port
    })
}

/// PSS (proportional set size) from smaps_rollup: shared pages count
/// fractionally per process, so summing a process tree reports the
/// REAL total instead of multiplying shared libraries/WebViews (RSS
/// summed a WebKit multi-process app to absurd numbers). Falls back to
/// sysinfo RSS where the file is unavailable (non-Linux).
#[cfg(target_os = "linux")]
fn proc_mem_bytes(pid: u32) -> Option<u64> {
    let content = std::fs::read_to_string(format!("/proc/{pid}/smaps_rollup")).ok()?;
    let line = content
        .lines()
        .find(|l| l.starts_with("ProportionalSetSize:"))?;
    let kb: u64 = line["ProportionalSetSize:".len()..]
        .trim()
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(kb * 1024)
}

#[cfg(not(target_os = "linux"))]
fn proc_mem_bytes(_pid: u32) -> Option<u64> {
    None
}

/// CPU% + RSS aggregated over each root's whole process tree (egui
/// proc_stats parity). Shells are our children, so the app tree covers
/// every terminal plus the UI/webview.
fn tree_agg(
    sys: &sysinfo::System,
    roots: &[u32],
    children: &std::collections::HashMap<u32, Vec<u32>>,
) -> (f32, u64) {
    use std::collections::{HashSet, VecDeque};
    let mut cpu = 0f32;
    let mut mem = 0u64;
    let mut seen = HashSet::new();
    let mut q: VecDeque<u32> = roots.iter().copied().collect();
    while let Some(pid) = q.pop_front() {
        if !seen.insert(pid) {
            continue;
        }
        if let Some(p) = sys.process(sysinfo::Pid::from_u32(pid)) {
            cpu += p.cpu_usage();
            mem += proc_mem_bytes(pid).unwrap_or_else(|| p.memory());
        }
        if let Some(kids) = children.get(&pid) {
            for k in kids {
                q.push_back(*k);
            }
        }
    }
    (cpu, mem)
}

/// Three-level resource sampling for the 系统资源 panel: focused
/// terminal, active workspace, and the whole software (the app's own
/// process tree). `focused`/`workspace` carry SESSION SLOT ids (the
/// frontend's term-N numbers) which map to the session shell pids here
/// — raw pids from the frontend would be meaningless (and dangerous:
/// slot 1 is init). CPU% is normalized to the machine's core count so
/// a tree of busy processes reads as its share of the whole machine
/// instead of a sum that exceeds 100. The SYS snapshot persists
/// between polls (frontend polls every 2s) so the deltas are real.
#[tauri::command]
fn resource_stats(
    state: tauri::State<AppState>,
    focused: Vec<String>,
    workspace: Vec<String>,
) -> serde_json::Value {
    let Some(sys_mutex) = SYS.get() else {
        return json!({ "focused": null, "workspace": null, "app": null });
    };
    let roots_of = |slots: &[String]| -> Vec<u32> {
        slots
            .iter()
            .filter_map(|sid| state.sessions.get(sid))
            .map(|s| s.pid)
            .filter(|&p| p != 0)
            .collect()
    };
    let f_roots = roots_of(&focused);
    let w_roots = roots_of(&workspace);
    let mut sys = sys_mutex.lock().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let cores = sys.cpus().len().max(1) as f32;
    let mut children: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    for (pid, proc_) in sys.processes() {
        if let Some(ppid) = proc_.parent() {
            children
                .entry(ppid.as_u32())
                .or_default()
                .push(pid.as_u32());
        }
    }
    let tree = |roots: &[u32]| -> (f32, u64) {
        let (cpu, mem) = tree_agg(&sys, roots, &children);
        (cpu / cores, mem)
    };
    let f = tree(&f_roots);
    let w = tree(&w_roots);
    let app = tree(&[std::process::id()]);
    json!({
        "focused": { "cpu": f.0, "mem": f.1 },
        "workspace": { "cpu": w.0, "mem": w.1 },
        "app": { "cpu": app.0, "mem": app.1 },
    })
}

/// System stats for the monitor page (CPU% + memory), via sysinfo.
#[tauri::command]
async fn system_stats(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
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
fn get_history(workspace_id: u64) -> Vec<serde_json::Value> {
    history()
        .lock()
        .unwrap()
        .get(workspace_id)
        .iter()
        .map(|e| json!({ "id": e.id, "cmd": e.cmd, "hits": e.hits }))
        .collect()
}

/// Delete only a record belonging to the requested workspace.
#[tauri::command]
fn delete_history(workspace_id: u64, id: u64) -> bool {
    history().lock().unwrap().delete(workspace_id, id)
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
            json!(s
                .last_activity_ms
                .load(std::sync::atomic::Ordering::Relaxed)),
        );
    }
    json!(out)
}

#[tauri::command]
fn set_history_cap(workspace_id: u64, cap: usize) {
    history().lock().unwrap().set_cap(workspace_id, cap);
}

#[tauri::command]
fn clear_history(workspace_id: u64) {
    history().lock().unwrap().clear(workspace_id);
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
    /// Legacy free-form git log, kept for old manifests — per-line
    /// fallback when the structured `changes` array is absent.
    #[serde(default)]
    changelog: Option<String>,
    #[serde(default)]
    history: Vec<HistoryEntry>,
}

#[derive(serde::Deserialize)]
struct HistoryEntry {
    version: String,
    #[serde(default)]
    changes: Vec<String>,
    #[serde(default)]
    changes_en: Vec<String>,
    #[serde(default)]
    changelog: Option<String>,
}

/// Why the fetched manifest can never become an installable update: the
/// public channel serves the egui app — none of its packages run in
/// this Tauri shell, so it is surfaced as read-only info only.
const UPDATE_UNAVAILABLE_REASON: &str = "This update channel only publishes egui packages; there is no Tauri-compatible build available for automatic installation.";

/// Strict `x.y.z` version parse — no lenient partial parsing (the old
/// `filter_map` silently accepted garbage like "1.2.foo.4").
fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    fn component(s: &str) -> Option<u64> {
        if s.is_empty()
            || !s.bytes().all(|b| b.is_ascii_digit())
            || (s.len() > 1 && s.starts_with('0'))
        {
            return None;
        }
        s.parse().ok()
    }
    let mut parts = v.split('.');
    let major = component(parts.next()?)?;
    let minor = component(parts.next()?)?;
    let patch = component(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// Exact version identity for top-level / history lookups.
fn same_version(a: &str, b: &str) -> bool {
    a == b
}

/// Release notes (zh, en) with the legacy per-line changelog fallback.
/// Only real manifest content is ever returned — nothing is synthesized.
fn notes_from(
    changes: &[String],
    changes_en: &[String],
    changelog: Option<&str>,
) -> (Vec<String>, Vec<String>) {
    if changes.is_empty() {
        let lines: Vec<String> = changelog
            .map(|cl| {
                cl.lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if !lines.is_empty() {
            return (lines, changes_en.to_vec());
        }
    }
    (changes.to_vec(), changes_en.to_vec())
}

/// Notes for the RUNNING version: exact match against the manifest top
/// level or its `history` entries. No match -> empty (never the latest
/// release's notes).
fn current_notes(m: &LatestManifest, current: &str) -> (Vec<String>, Vec<String>) {
    if same_version(&m.version, current) {
        return notes_from(&m.changes, &m.changes_en, m.changelog.as_deref());
    }
    if let Some(h) = m.history.iter().find(|h| same_version(&h.version, current)) {
        return notes_from(&h.changes, &h.changes_en, h.changelog.as_deref());
    }
    (Vec::new(), Vec::new())
}

/// Pure payload builder for `check_update`. The manifest belongs to the
/// egui release channel, which ships NO Tauri packages — so it is never
/// reported as an installable update, only as release info.
fn build_update_payload(m: &LatestManifest, current: &str) -> Result<serde_json::Value, String> {
    if parse_version(&m.version).is_none() {
        return Err(format!(
            "update manifest rejected: invalid version {:?}",
            m.version
        ));
    }
    let (changes, changes_en) = notes_from(&m.changes, &m.changes_en, m.changelog.as_deref());
    let (current_changes, current_changes_en) = current_notes(m, current);
    Ok(json!({
        "updateAvailable": false,
        "canInstall": false,
        "channel": "egui",
        "latest": m.version,
        "current": current,
        "changes": changes,
        "changesEn": changes_en,
        "currentChanges": current_changes,
        "currentChangesEn": current_changes_en,
        "unavailableReason": UPDATE_UNAVAILABLE_REASON,
    }))
}

/// Real Cargo version of this Tauri shell (compile-time constant).
#[tauri::command]
fn get_app_info() -> serde_json::Value {
    json!({ "current": env!("CARGO_PKG_VERSION") })
}

/// Update check: fetch the public (egui-channel) manifest and surface it
/// as informational metadata. `canInstall` stays false — no download or
/// install path exists for these packages in the Tauri app.
#[tauri::command]
async fn check_update() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let resp = ureq::get("https://opennex.download.zeadix.com/latest.json")
            .timeout(std::time::Duration::from_secs(15))
            .call()
            .map_err(|e| format!("request failed: {e}"))?;
        let m: LatestManifest = resp.into_json().map_err(|e| format!("parse failed: {e}"))?;
        build_update_payload(&m, env!("CARGO_PKG_VERSION"))
    })
    .await
    .map_err(|e| format!("task failed: {e}"))?
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
        let resp = req
            .send_json(body)
            .map_err(|e| format!("request failed: {e}"))?;
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
    let _ = REMOTE_TOKEN.set(uuid::Uuid::new_v4().simple().to_string());
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
            session_cwd,
            ws_port,
            list_shells,
            list_path_commands,
            list_fonts,
            remote_info,
            get_history,
            delete_history,
            get_app_info,
            ai_chat,
            check_update,
            system_stats,
            resource_stats,
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

#[cfg(test)]
mod workspace_history_tests {
    use super::HistoryStore;

    #[test]
    fn records_hits_and_mutations_are_workspace_local() {
        let mut store = HistoryStore::default();
        store.record(1, "pwd".into());
        store.record(2, "pwd".into());
        store.record(1, "pwd".into());
        assert_eq!(store.get(1)[0].hits, 2);
        assert_eq!(store.get(2)[0].hits, 1);
        let id = store.get(1)[0].id;
        assert!(!store.delete(2, id));
        assert!(store.delete(1, id));
        assert_eq!(store.get(2).len(), 1);
        store.set_cap(1, 10);
        for i in 0..20 {
            store.record(1, format!("cmd {i}"));
        }
        assert_eq!(store.get(1).len(), 10);
        assert_eq!(store.get(2).len(), 1);
        store.clear(1);
        assert!(store.get(1).is_empty());
        assert_eq!(store.get(2).len(), 1);
    }
}

#[cfg(test)]
mod update_metadata_tests {
    use super::{build_update_payload, get_app_info, parse_version, same_version, LatestManifest};

    fn manifest(version: &str) -> LatestManifest {
        LatestManifest {
            version: version.into(),
            changes: vec!["新增: egui 新功能".into()],
            changes_en: vec!["Add: egui feature".into()],
            changelog: Some("feat: legacy line\nfix: another legacy line".into()),
            history: vec![
                history_entry("0.2.0", vec!["新增: 历史版说明"], vec!["Add: history note"]),
                history_entry("0.1.9", vec![], vec![]),
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn history_entry(
        version: &str,
        changes: Vec<&str>,
        changes_en: Vec<&str>,
    ) -> super::HistoryEntry {
        super::HistoryEntry {
            version: version.into(),
            changes: changes.into_iter().map(String::from).collect(),
            changes_en: changes_en.into_iter().map(String::from).collect(),
            changelog: None,
        }
    }

    #[test]
    fn manifest_version_must_be_strict_x_y_z() {
        assert_eq!(parse_version("0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_version("1.23.456"), Some((1, 23, 456)));
        assert_eq!(parse_version("0.0.1"), Some((0, 0, 1)));
        assert_eq!(parse_version("0.2"), None);
        assert_eq!(parse_version("0.2.0.1"), None);
        assert_eq!(parse_version("0.2.foo"), None);
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version(" 0.2.0"), None);
        assert_eq!(parse_version("0.2.0 "), None);
        assert_eq!(parse_version("+1.0.0"), None);
        assert_eq!(parse_version("-1.0.0"), None);
        assert_eq!(parse_version("０.２.０"), None);
        assert_eq!(parse_version("abc"), None);
        assert_eq!(parse_version("999999999999999999999999.0.0"), None);
    }

    #[test]
    fn version_match_is_exact_not_prefix() {
        assert!(same_version("0.2.0", "0.2.0"));
        assert!(!same_version("0.2.0", "v0.2.0"));
        assert!(!same_version(" 0.2.0 ", "0.2.0"));
        assert!(!same_version("0.2.0", "0.2"));
        assert!(!same_version("0.2.0", "0.2.1"));
        assert!(!same_version("0.2.0", "0.2.0-beta"));
    }

    #[test]
    fn payload_is_never_an_installable_tauri_update() {
        let payload = build_update_payload(&manifest("999.0.0"), "0.1.0").unwrap();
        assert_eq!(payload["updateAvailable"], false);
        assert_eq!(payload["canInstall"], false);
        assert_eq!(payload["channel"], "egui");
        assert_eq!(payload["latest"], "999.0.0");
        assert_eq!(payload["current"], "0.1.0");
        let reason = payload["unavailableReason"].as_str().unwrap();
        assert!(
            reason.contains("egui"),
            "reason must explain the egui channel: {reason}"
        );
    }

    #[test]
    fn invalid_manifest_version_is_rejected() {
        for bad in ["latest", "0.2", "1.2.3.4", "", "x.y.z"] {
            let err = build_update_payload(&manifest(bad), "0.1.0").unwrap_err();
            assert!(
                err.contains(bad),
                "error should name the bad version {bad:?}: {err}"
            );
        }
    }

    #[test]
    fn latest_notes_come_from_changes_array() {
        let payload = build_update_payload(&manifest("999.0.0"), "0.1.0").unwrap();
        assert_eq!(payload["changes"][0], "新增: egui 新功能");
        assert_eq!(payload["changesEn"][0], "Add: egui feature");
    }

    #[test]
    fn legacy_changelog_falls_back_per_line_with_en() {
        let mut m = manifest("999.0.0");
        m.changes.clear();
        m.changes_en = vec!["Fix: english hotfix".into()];
        let payload = build_update_payload(&m, "0.1.0").unwrap();
        let changes: Vec<String> = serde_json::from_value(payload["changes"].clone()).unwrap();
        assert_eq!(
            changes,
            vec!["feat: legacy line", "fix: another legacy line"]
        );
        // English notes from the manifest survive the changelog fallback.
        let en: Vec<String> = serde_json::from_value(payload["changesEn"].clone()).unwrap();
        assert_eq!(en, vec!["Fix: english hotfix"]);
    }

    #[test]
    fn missing_english_notes_yield_empty_array_not_chinese_copy() {
        let mut m = manifest("999.0.0");
        m.changes_en.clear();
        let payload = build_update_payload(&m, "0.1.0").unwrap();
        let en: Vec<String> = serde_json::from_value(payload["changesEn"].clone()).unwrap();
        assert!(en.is_empty(), "must not fabricate English from Chinese");
        assert_eq!(payload["changes"][0], "新增: egui 新功能");
    }

    #[test]
    fn empty_notes_stay_empty_no_synthetic_log() {
        let mut m = manifest("999.0.0");
        m.changes.clear();
        m.changes_en.clear();
        m.changelog = None;
        let payload = build_update_payload(&m, "0.1.0").unwrap();
        assert_eq!(payload["changes"].as_array().unwrap().len(), 0);
        assert_eq!(payload["changesEn"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn current_changes_match_top_level_exactly() {
        let m = manifest("0.1.0");
        let payload = build_update_payload(&m, "0.1.0").unwrap();
        assert_eq!(payload["currentChanges"][0], "新增: egui 新功能");
        assert_eq!(payload["currentChangesEn"][0], "Add: egui feature");
    }

    #[test]
    fn current_changes_match_history_entry() {
        let m = manifest("999.0.0");
        let payload = build_update_payload(&m, "0.2.0").unwrap();
        assert_eq!(payload["currentChanges"][0], "新增: 历史版说明");
        assert_eq!(payload["currentChangesEn"][0], "Add: history note");
    }

    #[test]
    fn unmatched_current_never_reuses_latest_notes() {
        let m = manifest("999.0.0");
        for current in ["0.5.0", "0.1.9", "10.0.0"] {
            let payload = build_update_payload(&m, current).unwrap();
            assert!(
                payload["currentChanges"].as_array().unwrap().is_empty(),
                "current {current} must not inherit latest notes"
            );
            assert!(
                payload["currentChangesEn"].as_array().unwrap().is_empty(),
                "current {current} must not inherit latest English notes"
            );
        }
    }

    #[test]
    fn history_entry_with_empty_notes_falls_back_to_its_own_changelog_only() {
        let mut m = manifest("999.0.0");
        // 0.1.9 has empty changes; give IT a changelog — top-level notes
        // must not leak into the current-version slot.
        m.history[1].changelog = Some("fix: 0.1.9 hotfix".into());
        let payload = build_update_payload(&m, "0.1.9").unwrap();
        let changes: Vec<String> =
            serde_json::from_value(payload["currentChanges"].clone()).unwrap();
        assert_eq!(changes, vec!["fix: 0.1.9 hotfix"]);
    }

    #[test]
    fn get_app_info_reports_cargo_version() {
        let info = get_app_info();
        assert_eq!(info["current"], env!("CARGO_PKG_VERSION"));
    }
}

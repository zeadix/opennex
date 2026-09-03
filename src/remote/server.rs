//! Embedded HTTP + WebSocket server for the phone remote control.
//!
//! Pure std (`TcpListener` + one thread per connection), no async. Plain
//! HTTP requests (page, state, fallback polling) are answered directly;
//! `GET /ws?token=` upgrades to a WebSocket whose reader parses client
//! JSON messages into the shared command queue and whose writer drains
//! a per-connection broadcast channel. Every request must carry the
//! session token; wrong/missing tokens get 403.

use super::ansi::FrameMsg;
use super::protocol::{parse_query, token_ok, RemoteCommand, RemoteSnapshot};
use super::ws::{
    accept_key, parse_client_msg, parse_handshake, read_frame, write_frame, write_text, ClientMsg,
    WsOpcode, WsOut,
};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// State shared between the UI thread (writer) and the server threads
/// (readers). `subscribers` are the live WS writers - the UI broadcasts
/// frame JSON to all of them and prunes dead ones on send failure.
pub struct RemoteShared {
    pub snapshot: Arc<RwLock<RemoteSnapshot>>,
    pub frames: Arc<RwLock<HashMap<String, FrameMsg>>>,
    pub commands: Arc<Mutex<VecDeque<RemoteCommand>>>,
    pub subscribers: Arc<Mutex<Vec<mpsc::Sender<WsOut>>>>,
    /// Set by stop(): connection threads poll it and exit so an old
    /// token cannot keep serving a stopped session.
    pub shutdown: Arc<AtomicBool>,
    pub token: String,
    pub port: u16,
}

/// A running server: holds the accept-loop thread and the shutdown flag.
pub struct RemoteServer {
    shutdown: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl RemoteServer {
    /// Bind and start serving. Fails (without spawning) when the port is
    /// taken - the caller surfaces a toast instead of silently disabling.
    pub fn start(port: u16, token: String) -> Result<(RemoteShared, Self), String> {
        // Dual-stack: `[::]` accepts IPv6 AND (via v4-mapped addresses)
        // IPv4 on every mainstream platform, enabling public-IPv6 direct
        // phone access. Fall back to plain `0.0.0.0` where v6 is
        // unavailable (IPv6 disabled, or Windows v6-only sockets).
        let listener = match TcpListener::bind((std::net::Ipv6Addr::UNSPECIFIED, port)) {
            Ok(l) => l,
            Err(_) => TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, port))
                .map_err(|e| format!("bind port {port} failed: {e}"))?,
        };
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("set_nonblocking failed: {e}"))?;
        let shared = RemoteShared {
            snapshot: Arc::new(RwLock::new(RemoteSnapshot::default())),
            frames: Arc::new(RwLock::new(HashMap::new())),
            commands: Arc::new(Mutex::new(VecDeque::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            shutdown: Arc::new(AtomicBool::new(false)),
            token,
            port,
        };
        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_shared = shared.clone_arcs();
        let thread_shutdown = shutdown.clone();
        let handle = std::thread::Builder::new()
            .name("opennex-remote".into())
            .spawn(move || accept_loop(listener, thread_shared, thread_shutdown))
            .map_err(|e| format!("spawn failed: {e}"))?;
        Ok((
            shared,
            RemoteServer {
                shutdown,
                handle: Some(handle),
            },
        ))
    }

    pub fn stop(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        // Connection threads observe the flag within ~250ms and exit;
        // the accept loop only handles NEW connections.
    }
}

impl RemoteShared {
    fn clone_arcs(&self) -> Self {
        RemoteShared {
            snapshot: self.snapshot.clone(),
            frames: self.frames.clone(),
            commands: self.commands.clone(),
            subscribers: self.subscribers.clone(),
            shutdown: self.shutdown.clone(),
            token: self.token.clone(),
            port: self.port,
        }
    }
}

fn accept_loop(listener: TcpListener, shared: RemoteShared, shutdown: Arc<AtomicBool>) {
    while !shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let shared = shared.clone_arcs();
                std::thread::spawn(move || {
                    let _ = handle_connection(stream, &shared);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                // Listener gone; stop serving.
                break;
            }
        }
    }
}

/// One request per HTTP connection (Connection: close) - the phone
/// reopens per poll; simplest robust model. WebSocket connections
/// upgrade and live as long as the page does.
fn handle_connection(mut stream: TcpStream, shared: &RemoteShared) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut head = Vec::new();
    let mut buf = [0u8; 2048];
    loop {
        if head.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if head.len() > 16 * 1024 {
            return write_response(&mut stream, 431, "text/plain", "request head too large");
        }
        match stream.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => head.extend_from_slice(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e),
        }
    }
    // The read loop may have pulled MORE than the head: when head+body
    // arrive in one TCP segment (browsers send small XHR bodies in a
    // single write), the body bytes sit after the first \r\n\r\n in
    // `head`. Split them off before parsing, or the later read_exact
    // would stall until the 5s timeout (POSTs used to fail with empty
    // bodies and unlock could never succeed).
    let head_end = head
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| i + 4)
        .unwrap_or(head.len());
    let buffered_body: Vec<u8> = head[head_end..].to_vec();
    let head = head[..head_end].to_vec();
    let head_text = String::from_utf8_lossy(&head).into_owned();
    let (request_line, header_block) = match head_text.split_once("\r\n") {
        Some(parts) => parts,
        None => return write_response(&mut stream, 400, "text/plain", "bad request"),
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");
    let is_upgrade = header_block
        .lines()
        .any(|l| l.to_lowercase().starts_with("upgrade: websocket"));

    let body = if header_block
        .lines()
        .any(|l| l.to_lowercase().starts_with("content-length:"))
    {
        let len: usize = header_block
            .lines()
            .find(|l| l.to_lowercase().starts_with("content-length:"))
            .and_then(|l| l.split_once(':').and_then(|(_, v)| v.trim().parse().ok()))
            .unwrap_or(0);
        let len = len.min(64 * 1024);
        // Take whatever already arrived inside `head`, read the rest.
        let take = buffered_body.len().min(len);
        let mut body = buffered_body[..take].to_vec();
        body.resize(len, 0);
        if len > take {
            let _ = stream.read_exact(&mut body[take..]);
        }
        body
    } else {
        Vec::new()
    };

    let (route, query) = match path.split_once('?') {
        Some((r, q)) => (r, q),
        None => (path, ""),
    };
    let query_pairs = parse_query(query);

    // Static page assets are served without a token: the browser loads
    // <script src="/xterm.js"> without appending the page's query string,
    // so requiring it here would 403 the very files the page needs to
    // boot. The assets themselves carry no secrets; the token gate still
    // guards the page, every API route and the WebSocket.
    if !matches!(route, "/xterm.js" | "/xterm.css") {
        let provided = query_pairs
            .iter()
            .find(|(k, _)| k == "token")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        if !token_ok(&provided, &shared.token) {
            return write_response(&mut stream, 403, "text/plain", "forbidden");
        }
    }

    // WebSocket upgrade: hand the connection to the ws pump (any bytes
    // after \r\n\r\n are still buffered inside `head`).
    if is_upgrade && route == "/ws" {
        stream.set_read_timeout(None)?;
        return handle_ws(stream, &head_text, shared);
    }

    match (method, route) {
        ("GET", "/") => {
            let html = super::REMOTE_PAGE;
            write_response(&mut stream, 200, "text/html; charset=utf-8", html)
        }
        ("GET", "/xterm.js") => {
            let js = include_str!("../../assets/vendor/xterm.min.js");
            write_response(
                &mut stream,
                200,
                "application/javascript; charset=utf-8",
                js,
            )
        }
        ("GET", "/xterm.css") => {
            let css = include_str!("../../assets/vendor/xterm.min.css");
            write_response(&mut stream, 200, "text/css; charset=utf-8", css)
        }
        ("GET", "/api/state") => {
            let snapshot = shared.snapshot.read().unwrap();
            let body = serde_json::to_string(&*snapshot).unwrap_or_default();
            write_response(&mut stream, 200, "application/json; charset=utf-8", &body)
        }
        ("GET", "/api/frame") => {
            let tab = query_pairs
                .iter()
                .find(|(k, _)| k == "tab")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let frames = shared.frames.read().unwrap();
            match frames.get(&tab) {
                Some(frame) => {
                    let body = frame.to_json();
                    write_response(&mut stream, 200, "application/json; charset=utf-8", &body)
                }
                None => write_response(&mut stream, 404, "text/plain", "no such terminal"),
            }
        }
        ("GET", "/api/scrollback") => {
            let tab = query_pairs
                .iter()
                .find(|(k, _)| k == "tab")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let (tx, rx) = mpsc::channel();
            shared
                .commands
                .lock()
                .unwrap()
                .push_back(RemoteCommand::RequestScrollback { tab, reply: tx });
            match rx.recv_timeout(Duration::from_secs(3)) {
                Ok(ansi) => {
                    let body = json_string(&ansi);
                    write_response(&mut stream, 200, "application/json; charset=utf-8", &body)
                }
                Err(_) => write_response(&mut stream, 404, "text/plain", "no scrollback"),
            }
        }
        ("GET", "/api/scroll_bottom") => {
            let tab = query_pairs
                .iter()
                .find(|(k, _)| k == "tab")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            shared
                .commands
                .lock()
                .unwrap()
                .push_back(RemoteCommand::ScrollBottom { tab });
            write_response(&mut stream, 200, "application/json", "{\"ok\":true}")
        }
        ("POST", "/api/focus") => {
            let body_text = String::from_utf8_lossy(&body);
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body_text);
            match parsed {
                Ok(v) if v.get("tab").and_then(|t| t.as_str()).is_some() => {
                    let tab = v["tab"].as_str().unwrap().to_string();
                    shared
                        .commands
                        .lock()
                        .unwrap()
                        .push_back(RemoteCommand::Focus { tab });
                    write_response(&mut stream, 200, "application/json", "{\"ok\":true}")
                }
                _ => write_response(&mut stream, 400, "text/plain", "missing tab"),
            }
        }
        ("POST", "/api/input") => {
            let body_text = String::from_utf8_lossy(&body);
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body_text);
            match parsed {
                Ok(v) => {
                    let tab = v.get("tab").and_then(|t| t.as_str()).unwrap_or("");
                    let data = v.get("data").and_then(|d| d.as_str()).unwrap_or("");
                    if !tab.is_empty() && !data.is_empty() {
                        shared
                            .commands
                            .lock()
                            .unwrap()
                            .push_back(RemoteCommand::Write {
                                tab: tab.to_string(),
                                data: data.to_string(),
                            });
                    }
                    write_response(&mut stream, 200, "application/json", "{\"ok\":true}")
                }
                _ => write_response(&mut stream, 400, "text/plain", "bad body"),
            }
        }
        ("POST", "/api/unlock") => {
            let body_text = String::from_utf8_lossy(&body);
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body_text);
            let panel = parsed
                .as_ref()
                .ok()
                .and_then(|v| v.get("panel"))
                .and_then(|p| p.as_u64())
                .unwrap_or(usize::MAX as u64) as usize;
            let password = parsed
                .ok()
                .and_then(|v| {
                    v.get("password")
                        .and_then(|p| p.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_default();
            let (tx, rx) = mpsc::channel();
            shared
                .commands
                .lock()
                .unwrap()
                .push_back(RemoteCommand::Unlock {
                    panel,
                    password,
                    reply: tx,
                });
            // The UI thread answers within ~2 frames; bounded wait.
            match rx.recv_timeout(Duration::from_secs(3)) {
                Ok(true) => write_response(&mut stream, 200, "application/json", "{\"ok\":true}"),
                _ => write_response(&mut stream, 403, "text/plain", "wrong password"),
            }
        }
        _ => write_response(&mut stream, 404, "text/plain", "not found"),
    }
}

/// Push the freshest serialized frame of `tab` to one connection - the
/// immediate-screen fix: broadcasts only carry CHANGES, so a freshly
/// connected/switching phone would otherwise stare at scrollback or
/// blank until the terminal next prints something.
fn push_current_frame(shared: &RemoteShared, tab: &str, tx: &mpsc::Sender<WsOut>) {
    let frames = shared.frames.read().unwrap();
    if let Some(frame) = frames.get(tab) {
        let _ = tx.send(WsOut::Text(super::ansi::frame_json(tab, frame)));
    }
}

/// Run one WebSocket connection: complete the handshake, then a writer
/// thread (broadcast drain + keepalive pings) while this thread reads
/// client frames until close/EOF.
fn handle_ws(mut stream: TcpStream, head_text: &str, shared: &RemoteShared) -> std::io::Result<()> {
    // The token was validated by handle_connection before we got here.
    let handshake = match parse_handshake(head_text) {
        Some(hs) if hs.path == "/ws" => hs,
        _ => return write_response(&mut stream, 400, "text/plain", "bad upgrade"),
    };
    let accept = accept_key(&handshake.key);
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
Sec-WebSocket-Accept: {accept}\r\n\r\n"
    );
    stream.write_all(response.as_bytes())?;

    // Writer pump: drains the broadcast channel; pings on 20s idle so
    // half-open connections eventually die.
    let (tx, rx) = mpsc::channel::<WsOut>();
    let mut writer_stream = stream.try_clone()?;
    let writer = std::thread::spawn(move || loop {
        match rx.recv_timeout(Duration::from_secs(20)) {
            Ok(WsOut::Text(msg)) => {
                if write_text(&mut writer_stream, &msg).is_err() {
                    break;
                }
            }
            Ok(WsOut::Ping) => {
                if write_frame(&mut writer_stream, WsOpcode::Ping, b"").is_err() {
                    break;
                }
            }
            Ok(WsOut::Pong(payload)) => {
                if write_frame(&mut writer_stream, WsOpcode::Pong, &payload).is_err() {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if write_frame(&mut writer_stream, WsOpcode::Ping, b"").is_err() {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    });

    // Reader loop (this thread). hello registers the writer as a
    // subscriber; every other message maps onto the command queue.
    // Read with a timeout so a stopped server sheds connections instead
    // of serving a dead token forever.
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    loop {
        if shared.shutdown.load(Ordering::SeqCst) {
            break;
        }
        let frame = match read_frame(&mut stream) {
            Some(frame) => frame,
            None => {
                // Distinguish idle (WouldBlock, retried) from a dead
                // socket: read_frame returns None on EOF too, so probe.
                let mut probe = [0u8; 1];
                match stream.peek(&mut probe) {
                    Ok(0) => break,    // EOF: phone closed
                    Ok(_) => continue, // partial frame, retry
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(_) => break,
                }
            }
        };
        {
            match frame.opcode {
                WsOpcode::Text => {
                    if let Some(msg) = parse_client_msg(&frame.payload) {
                        match msg {
                            ClientMsg::Hello => {
                                shared.subscribers.lock().unwrap().push(tx.clone());
                            }
                            ClientMsg::Focus { tab } => {
                                push_current_frame(shared, &tab, &tx);
                                shared
                                    .commands
                                    .lock()
                                    .unwrap()
                                    .push_back(RemoteCommand::Focus { tab });
                            }
                            ClientMsg::Input { tab, data } => {
                                shared
                                    .commands
                                    .lock()
                                    .unwrap()
                                    .push_back(RemoteCommand::Write { tab, data });
                            }
                            ClientMsg::ScrollBottom { tab } => {
                                shared
                                    .commands
                                    .lock()
                                    .unwrap()
                                    .push_back(RemoteCommand::ScrollBottom { tab });
                            }
                            ClientMsg::Mouse {
                                tab,
                                btn,
                                col,
                                row,
                                pressed,
                            } => {
                                shared
                                    .commands
                                    .lock()
                                    .unwrap()
                                    .push_back(RemoteCommand::Mouse {
                                        tab,
                                        btn,
                                        col,
                                        row,
                                        pressed,
                                    });
                            }
                            ClientMsg::Scrollback { tab } => {
                                // Serialize on the UI thread, deliver through
                                // this connection's writer; the visible screen
                                // rides along as a fresh frame right after.
                                let (rtx, rrx) = mpsc::channel();
                                shared.commands.lock().unwrap().push_back(
                                    RemoteCommand::RequestScrollback {
                                        tab: tab.clone(),
                                        reply: rtx,
                                    },
                                );
                                if let Ok(ansi) = rrx.recv_timeout(Duration::from_secs(3)) {
                                    let payload = format!(
                                        "{{\"t\":\"scrollback\",\"d\":{}}}",
                                        json_string(&ansi)
                                    );
                                    let _ = tx.send(WsOut::Text(payload));
                                    push_current_frame(shared, &tab, &tx);
                                }
                            }
                        }
                    }
                }
                WsOpcode::Ping => {
                    let _ = tx.send(WsOut::Pong(frame.payload));
                }
                // Pong = the phone ANSWERED our keepalive. Treating it as
                // Close (the v2 bug) killed every idle connection ~20s in.
                WsOpcode::Pong => continue,
                WsOpcode::Close => break,
                _ => {}
            }
        }
    }
    // Cleanup: send a final bye (best effort), then drop the sender so
    // the UI-side broadcast prunes this subscription on its next pass.
    let _ = tx.send(WsOut::Text("{\"t\":\"bye\"}".into()));
    drop(tx);
    let _ = writer.join();
    Ok(())
}

/// JSON-stringify a raw string payload.
fn json_string(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_binds_and_serves_index() {
        // Pick a random high port to avoid CI collisions.
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (shared, server) = RemoteServer::start(port, "test-token".into()).unwrap();
        // Plain std HTTP GET to the served page.
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .write_all(b"GET /?token=test-token HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        assert!(buf.starts_with("HTTP/1.1 200"), "{buf}");
        assert!(buf.contains("OpenNex Remote"));
        server.stop();
        let _ = shared;
    }

    #[test]
    fn static_assets_served_without_token() {
        // The page references /xterm.js and /xterm.css WITHOUT the token
        // query (browsers do not append it) - the token gate must skip
        // these two routes or the phone page cannot boot at all.
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (_shared, server) = RemoteServer::start(port, "t".into()).unwrap();
        for path in ["/xterm.js", "/xterm.css"] {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            let req = format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n");
            stream.write_all(req.as_bytes()).unwrap();
            let mut buf = String::new();
            stream.read_to_string(&mut buf).unwrap();
            assert!(buf.starts_with("HTTP/1.1 200"), "{path}: {buf:.80}");
        }
        server.stop();
    }

    #[test]
    fn post_body_arriving_with_head_is_parsed() {
        // Browsers send small XHR bodies in ONE TCP segment together with
        // the head. The body bytes land in the head buffer - the server
        // must split them off instead of stalling on read_exact (v2.5 bug:
        // every POST failed with an empty body and a 5s delay).
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (shared, server) = RemoteServer::start(port, "t".into()).unwrap();
        let body = r#"{"tab":"terminal-1"}"#;
        let request = format!(
            "POST /api/focus?token=t HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        assert!(buf.starts_with("HTTP/1.1 200"), "{buf}");
        // The command actually reached the queue.
        let queued = shared.commands.lock().unwrap().len();
        assert_eq!(queued, 1, "focus command must be queued");
        server.stop();
    }

    #[test]
    fn wrong_token_is_forbidden() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (_shared, server) = RemoteServer::start(port, "good-token".into()).unwrap();
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .write_all(b"GET /?token=bad-token HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        assert!(buf.starts_with("HTTP/1.1 403"), "{buf}");
        server.stop();
    }

    #[test]
    fn api_state_serves_the_shared_snapshot() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (shared, server) = RemoteServer::start(port, "t".into()).unwrap();
        {
            let mut snap = shared.snapshot.write().unwrap();
            snap.workspaces.push(super::super::protocol::WsInfo {
                name: "ws1".into(),
                locked: false,
                terminals: vec![],
                activity: 0,
            });
        }
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .write_all(b"GET /api/state?token=t HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        let body = buf.split("\r\n\r\n").nth(1).unwrap_or("");
        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(v["workspaces"][0]["name"], "ws1");
        server.stop();
    }

    #[test]
    fn ws_handshake_and_frame_push_roundtrip() {
        use base64::Engine as _;
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let (shared, server) = RemoteServer::start(port, "t".into()).unwrap();

        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        // Random-ish key; must be base64 of 16 bytes.
        let key = base64::engine::general_purpose::STANDARD.encode([0u8; 16]);
        let request = format!(
            "GET /ws?token=t HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\n\
Connection: Upgrade\r\nSec-WebSocket-Key: {key}\r\n\r\n"
        );
        client.write_all(request.as_bytes()).unwrap();
        // Read the 101 head.
        let mut head = vec![0u8; 1];
        let mut acc = Vec::new();
        while !acc.windows(4).any(|w| w == b"\r\n\r\n") {
            client.read_exact(&mut head).unwrap();
            acc.extend_from_slice(&head);
        }
        let head_text = String::from_utf8_lossy(&acc).into_owned();
        assert!(head_text.starts_with("HTTP/1.1 101"), "{head_text}");
        assert!(head_text.contains(&accept_key(&key)));

        // Send a masked "hello" text frame, then expect a UI-side
        // broadcast (simulated by pushing into subscribers) to arrive
        // as a text frame.
        let payload = br#"{"t":"hello"}"#;
        let mask = [0x11u8, 0x22, 0x33, 0x44];
        let mut frame = vec![0x81u8, 0x80 | payload.len() as u8];
        frame.extend_from_slice(&mask);
        for (i, b) in payload.iter().enumerate() {
            frame.push(b ^ mask[i % 4]);
        }
        client.write_all(&frame).unwrap();

        // Give the reader thread a moment to register the subscriber.
        std::thread::sleep(Duration::from_millis(150));
        {
            let subs = shared.subscribers.lock().unwrap();
            assert_eq!(subs.len(), 1);
            subs[0]
                .send(WsOut::Text("{\"t\":\"frame\",\"d\":\"x\"}".into()))
                .unwrap();
        }

        let got = read_frame(&mut client).unwrap();
        assert_eq!(got.opcode, WsOpcode::Text);
        assert!(String::from_utf8(got.payload).unwrap().contains("frame"));
        server.stop();
    }
}

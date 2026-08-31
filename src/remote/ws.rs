//! Minimal WebSocket server layer (RFC 6455) for the phone remote: the
//! handshake, frame encode/decode and the per-connection reader/writer
//! pumps. Text frames carry the JSON message protocol only - no binary,
//! no fragmentation on our side, and client frames MUST be masked.

use std::io::{Read, Write};
use std::net::TcpStream;

use base64::Engine as _;
use sha1::{Digest, Sha1};

/// The RFC 6455 magic GUID appended to the client key.
const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Compute the Sec-WebSocket-Accept value for a client key.
pub fn accept_key(client_key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(client_key.as_bytes());
    hasher.update(WS_GUID.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsOpcode {
    Continuation,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
}

impl WsOpcode {
    fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x0 => Some(Self::Continuation),
            0x1 => Some(Self::Text),
            0x2 => Some(Self::Binary),
            0x8 => Some(Self::Close),
            0x9 => Some(Self::Ping),
            0xA => Some(Self::Pong),
            _ => None,
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            Self::Continuation => 0x0,
            Self::Text => 0x1,
            Self::Binary => 0x2,
            Self::Close => 0x8,
            Self::Ping => 0x9,
            Self::Pong => 0xA,
        }
    }
}

/// One decoded client frame (always unmasked by the reader).
pub struct WsFrame {
    pub opcode: WsOpcode,
    pub payload: Vec<u8>,
}

/// Read one frame from `stream`. Returns None on EOF/protocol error.
/// Client-to-server frames MUST be masked per RFC 6455; we tolerate
/// unmasked ones anyway (some proxies strip masks).
pub fn read_frame(stream: &mut TcpStream) -> Option<WsFrame> {
    let mut head = [0u8; 2];
    stream.read_exact(&mut head).ok()?;
    let fin = head[0] & 0x80 != 0;
    if !fin {
        // We never fragment; treat as protocol error.
        return None;
    }
    let opcode = WsOpcode::from_u8(head[0] & 0x0F)?;
    let masked = head[1] & 0x80 != 0;
    let mut len = (head[1] & 0x7F) as u64;
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext).ok()?;
        len = u16::from_be_bytes(ext) as u64;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext).ok()?;
        len = u64::from_be_bytes(ext);
    }
    // Guard: a phone page sends tiny JSON messages only.
    if len > 4 * 1024 * 1024 {
        return None;
    }
    let mask = if masked {
        let mut m = [0u8; 4];
        stream.read_exact(&mut m).ok()?;
        Some(m)
    } else {
        None
    };
    let mut payload = vec![0u8; len as usize];
    stream.read_exact(&mut payload).ok()?;
    if let Some(m) = mask {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= m[i % 4];
        }
    }
    Some(WsFrame { opcode, payload })
}

/// Encode and send one server frame (never masked, per RFC).
pub fn write_frame(
    stream: &mut TcpStream,
    opcode: WsOpcode,
    payload: &[u8],
) -> std::io::Result<()> {
    let mut out = Vec::with_capacity(payload.len() + 10);
    out.push(0x80 | opcode.to_u8());
    let len = payload.len();
    if len < 126 {
        out.push(len as u8);
    } else if len <= u16::MAX as usize {
        out.push(126);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(127);
        out.extend_from_slice(&(len as u64).to_be_bytes());
    }
    out.extend_from_slice(payload);
    stream.write_all(&out)
}

/// Send one text frame.
pub fn write_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    write_frame(stream, WsOpcode::Text, text.as_bytes())
}

/// Parsed handshake of an HTTP upgrade request.
pub struct WsHandshake {
    pub key: String,
    pub path: String,
    pub query: String,
}

/// Parse a `GET /ws?... HTTP/1.1` upgrade request head (the raw bytes up
/// to and excluding the blank line).
pub fn parse_handshake(head: &str) -> Option<WsHandshake> {
    let mut lines = head.split("\r\n");
    let request = lines.next()?;
    let mut parts = request.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let target = parts.next()?;
    let (path, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (target.to_string(), String::new()),
    };
    let mut key = String::new();
    let mut upgrade = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_lowercase();
        let value = value.trim();
        if name == "sec-websocket-key" {
            key = value.to_string();
        } else if name == "upgrade" && value.to_lowercase().contains("websocket") {
            upgrade = true;
        }
    }
    if !upgrade || key.is_empty() {
        return None;
    }
    Some(WsHandshake { key, path, query })
}

/// Write the 101 Switching Protocols response.
pub fn write_handshake_response(stream: &mut TcpStream, key: &str) -> std::io::Result<()> {
    let accept = accept_key(key);
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
Sec-WebSocket-Accept: {accept}\r\n\r\n"
    );
    stream.write_all(response.as_bytes())
}

/// One outbound message for a connection's writer pump.
#[derive(Debug, Clone)]
pub enum WsOut {
    /// A JSON protocol message (frame/scrollback/bye).
    Text(String),
    /// Keepalive ping.
    Ping,
    /// Response to a client ping.
    Pong(Vec<u8>),
}

/// What one client message asked for (JSON text frames).
#[derive(Debug, Clone, PartialEq)]
pub enum ClientMsg {
    Hello,
    Focus { tab: String },
    Input { tab: String, data: String },
    Scrollback { tab: String },
}

/// Parse a client text-frame payload into a ClientMsg. Unknown shapes
/// return None and the connection just continues.
pub fn parse_client_msg(payload: &[u8]) -> Option<ClientMsg> {
    let text = std::str::from_utf8(payload).ok()?;
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    match v.get("t")?.as_str()? {
        "hello" => Some(ClientMsg::Hello),
        "focus" => Some(ClientMsg::Focus {
            tab: v.get("tab")?.as_str()?.to_string(),
        }),
        "input" => Some(ClientMsg::Input {
            tab: v.get("tab")?.as_str()?.to_string(),
            data: v.get("d")?.as_str()?.to_string(),
        }),
        "scrollback" => Some(ClientMsg::Scrollback {
            tab: v.get("tab")?.as_str()?.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_key_matches_rfc6455_example() {
        // RFC 6455 §1.3 example vector.
        assert_eq!(
            accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
    }

    #[test]
    fn handshake_parses_target_and_key() {
        let head = "GET /ws?token=abc HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\n\
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n";
        let hs = parse_handshake(head).unwrap();
        assert_eq!(hs.path, "/ws");
        assert_eq!(hs.query, "token=abc");
        assert_eq!(hs.key, "dGhlIHNhbXBsZSBub25jZQ==");
        // Missing upgrade header -> None.
        assert!(parse_handshake("GET /ws HTTP/1.1\r\nHost: x\r\n").is_none());
    }

    #[test]
    fn client_messages_parse_from_json() {
        assert_eq!(
            parse_client_msg(br#"{"t":"hello"}"#),
            Some(ClientMsg::Hello)
        );
        assert_eq!(
            parse_client_msg(br#"{"t":"input","tab":"terminal-1","d":"ls\r"}"#),
            Some(ClientMsg::Input {
                tab: "terminal-1".into(),
                data: "ls\r".into()
            })
        );
        assert_eq!(
            parse_client_msg(br#"{"t":"scrollback","tab":"terminal-1"}"#),
            Some(ClientMsg::Scrollback {
                tab: "terminal-1".into()
            })
        );
        assert!(parse_client_msg(br#"{"t":"nope"}"#).is_none());
        assert!(parse_client_msg(b"not json").is_none());
    }

    #[test]
    fn frame_roundtrip_small_and_extended() {
        // Round-trip through an in-memory socket pair.
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        write_text(&mut server, "hello").unwrap();
        let frame = read_frame(&mut client).unwrap();
        assert_eq!(frame.opcode, WsOpcode::Text);
        assert_eq!(String::from_utf8(frame.payload).unwrap(), "hello");
        // Extended length path (126).
        let big = "x".repeat(300);
        write_text(&mut server, &big).unwrap();
        let frame = read_frame(&mut client).unwrap();
        assert_eq!(frame.payload.len(), 300);
    }

    #[test]
    fn handshake_response_carries_accept() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let (mut server, _) = listener.accept().unwrap();
        write_handshake_response(&mut server, "dGhlIHNhbXBsZSBub25jZQ==").unwrap();
        // Close the server side so read_to_string sees EOF instead of
        // blocking forever.
        drop(server);
        let mut buf = String::new();
        client.read_to_string(&mut buf).unwrap();
        assert!(buf.starts_with("HTTP/1.1 101"), "{buf}");
        assert!(
            buf.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo="),
            "{buf}"
        );
    }
}

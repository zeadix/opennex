//! Remote phone control: protocol types, the terminal-grid serializer
//! and small pure helpers (LAN ip detection, QR url building).
//!
//! Architecture (no tokio — matches the project's std::thread style):
//! the UI thread refreshes a shared [`RemoteSnapshot`] every frame and
//! drains a command queue; an embedded HTTP server thread serves the
//! snapshot and pushes commands back. Phones poll plain HTTP endpoints —
//! the most WeChat-internal-browser-compatible transport.

use serde::Serialize;
use std::sync::mpsc;

/// One terminal in the snapshot (identity + geometry only; the screen
/// itself is fetched per-tab via `/api/frame`).
#[derive(Debug, Clone, Serialize)]
pub struct TermInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub cwd: String,
    pub cols: usize,
    pub rows: usize,
}

/// One workspace. A locked workspace hides its terminals entirely.
#[derive(Debug, Clone, Serialize)]
pub struct WsInfo {
    pub name: String,
    pub locked: bool,
    pub terminals: Vec<TermInfo>,
}

/// The full state the phone page renders its workspace/tab bars from.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RemoteSnapshot {
    pub workspaces: Vec<WsInfo>,
    pub focused: Option<String>,
}

/// A command the phone sent, executed by the UI thread on the next
/// frame (terminal writes and focus changes MUST happen there).
pub enum RemoteCommand {
    /// Focus a terminal (the UI thread resolves the workspace by
    /// searching the dock trees and selects the tab).
    Focus { tab: String },
    /// Feed bytes into a terminal (Enter, Ctrl-C etc. arrive pre-encoded).
    Write { tab: String, data: String },
    /// Try to unlock a workspace; the reply reports success.
    Unlock {
        panel: usize,
        password: String,
        reply: mpsc::Sender<bool>,
    },
    /// Serialize a terminal's scrollback on the UI thread; the reply
    /// carries the ANSI stream.
    RequestScrollback {
        tab: String,
        reply: mpsc::Sender<String>,
    },
}

/// Best-effort LAN ip: UDP "connect" to a public address never sends a
/// packet but makes the OS pick the outbound interface — zero deps.
pub fn lan_ip() -> Option<String> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    sock.local_addr().ok().map(|a| a.ip().to_string())
}

/// Is this a GLOBAL (publicly routable) IPv6 address? Excludes loopback,
/// unspecified, link-local (fe80::/10) and ULA (fc00::/7) ranges.
pub fn is_global_ipv6(ip: &std::net::Ipv6Addr) -> bool {
    let first = ip.segments()[0];
    !ip.is_loopback()
        && !ip.is_unspecified()
        && (first & 0xffc0) != 0xfe80 // link-local
        && (first & 0xfe00) != 0xfc00 // ULA
}

/// Best-effort public IPv6 for direct phone access over cellular: the
/// same UDP trick against a public IPv6 resolver. None whenever the
/// machine has no global IPv6 route (very common on LANs).
pub fn public_ipv6() -> Option<String> {
    let sock = std::net::UdpSocket::bind("[::]:0").ok()?;
    sock.connect("[2001:4860:4860::8888]:53").ok()?;
    let addr = sock.local_addr().ok()?;
    match addr.ip() {
        std::net::IpAddr::V6(v6) if is_global_ipv6(&v6) => Some(v6.to_string()),
        _ => None,
    }
}

/// The QR / copyable entry URL. IPv6 literals need bracketing or the
/// port separator becomes ambiguous (`http://2408::1:47822/`).
pub fn remote_url(ip: &str, port: u16, token: &str) -> String {
    if ip.contains(':') && !ip.starts_with('[') {
        format!("http://[{ip}]:{port}/?token={token}")
    } else {
        format!("http://{ip}:{port}/?token={token}")
    }
}

/// Constant-time-ish token check for the query string (avoids the easy
/// timing side channel of `==`; overkill for LAN but free).
pub fn token_ok(provided: &str, expected: &str) -> bool {
    let p = provided.as_bytes();
    let e = expected.as_bytes();
    if p.len() != e.len() {
        return false;
    }
    p.iter().zip(e).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0
}

/// Minimal query-string parsing: `key=value` pairs after `?`.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((k.to_string(), percent_decode(v)))
        })
        .collect()
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_check_is_exact_and_not_timing_obvious() {
        assert!(token_ok("abc123", "abc123"));
        assert!(!token_ok("abc123", "abc124"));
        assert!(!token_ok("abc12", "abc123"));
        assert!(!token_ok("", "abc123"));
    }

    #[test]
    fn query_parsing_decodes_percent() {
        let q = parse_query("token=a%20b&port=47822");
        assert_eq!(q[0], ("token".to_string(), "a b".to_string()));
        assert_eq!(q[1], ("port".to_string(), "47822".to_string()));
        assert!(parse_query("noequals").is_empty());
    }

    #[test]
    fn global_ipv6_filter_rejects_private_ranges() {
        use std::net::Ipv6Addr;
        assert!(is_global_ipv6(&Ipv6Addr::new(
            0x2408, 0x8888, 0, 0, 0, 0, 0, 1
        )));
        assert!(!is_global_ipv6(&Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)));
        assert!(!is_global_ipv6(&Ipv6Addr::new(0xfd00, 0, 0, 0, 0, 0, 0, 1)));
        assert!(!is_global_ipv6(&Ipv6Addr::new(0xfc42, 0, 0, 0, 0, 0, 0, 1)));
        assert!(!is_global_ipv6(&Ipv6Addr::LOCALHOST));
        assert!(!is_global_ipv6(&Ipv6Addr::UNSPECIFIED));
    }

    #[test]
    fn url_builds_with_token() {
        assert_eq!(
            remote_url("192.168.1.5", 47822, "tok"),
            "http://192.168.1.5:47822/?token=tok"
        );
        // IPv6 literals get bracketed (unbracketed is an invalid URL).
        assert_eq!(
            remote_url("2408:8888::1", 47822, "tok"),
            "http://[2408:8888::1]:47822/?token=tok"
        );
    }

    #[test]
    fn lan_ip_is_ipv4_local_when_available() {
        // Non-deterministic by nature; only assert the shape when present.
        if let Some(ip) = lan_ip() {
            assert!(
                ip.parse::<std::net::Ipv4Addr>().is_ok()
                    || ip.parse::<std::net::Ipv6Addr>().is_ok()
            );
        }
    }
}

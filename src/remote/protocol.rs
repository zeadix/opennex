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

/// One serialized screen frame. `lines` holds one HTML fragment per
/// grid row (runs of equal style merged into spans); the phone page
/// drops it straight into a preformatted container.
#[derive(Debug, Clone, Serialize)]
pub struct TermFrame {
    pub seq: u64,
    pub cols: usize,
    pub rows: usize,
    pub cur_x: usize,
    pub cur_y: usize,
    pub lines: Vec<String>,
}

impl TermFrame {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }
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
}

/// Best-effort LAN ip: UDP "connect" to a public address never sends a
/// packet but makes the OS pick the outbound interface — zero deps.
pub fn lan_ip() -> Option<String> {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    sock.local_addr().ok().map(|a| a.ip().to_string())
}

/// The QR / copyable entry URL.
pub fn remote_url(ip: &str, port: u16, token: &str) -> String {
    format!("http://{ip}:{port}/?token={token}")
}

/// Escape the four characters that would break an inline HTML fragment.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// Serialize the VISIBLE screen of a terminal grid into one frame.
///
/// Style runs are merged (adjacent cells with identical fg/bg/bold
/// collapse into a single span) to keep payloads small; spaces become
/// &nbsp; so the browser preserves trailing whitespace.
pub fn serialize_frame(
    grid: &alacritty_terminal::grid::Grid<alacritty_terminal::term::cell::Cell>,
    theme: &egui_term::TerminalTheme,
    seq: u64,
) -> TermFrame {
    use alacritty_terminal::grid::Dimensions;
    use alacritty_terminal::index::{Column, Line, Point};
    use alacritty_terminal::term::cell::{self, Flags};

    let cols = grid.columns();
    let rows = grid.screen_lines();
    let display_offset = grid.display_offset() as i32;
    let viewport_start = -display_offset; // Line of the visible top row

    let mut lines = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut html = String::with_capacity(cols * 16);
        let mut run_chars = String::new();
        let mut run_fg = String::new();
        let mut run_bg = String::new();
        let mut run_bold = false;
        let mut run_started = false;

        let flush = |html: &mut String,
                     chars: &mut String,
                     fg: &str,
                     bg: &str,
                     bold: bool,
                     started: &mut bool| {
            if !*started {
                return;
            }
            let style = if bold {
                format!("color:{};background:{};font-weight:bold", fg, bg)
            } else {
                format!("color:{};background:{}", fg, bg)
            };
            html.push_str(&format!("<span style=\"{style}\">{chars}</span>"));
            chars.clear();
            *started = false;
        };

        for col in 0..cols {
            let cell = &grid[Point::new(Line(viewport_start + row as i32), Column(col))];
            // Wide-char spacer cells render as a space inheriting the
            // previous run's background.
            let ch = if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                ' '
            } else {
                cell.c
            };
            let ch = if ch == ' ' { '\u{a0}' } else { ch };
            let fg = theme.get_color(cell.fg);
            let bg = theme.get_color(cell.bg);
            let bold = cell.flags.contains(cell::Flags::BOLD);
            let fg_hex = format!("#{:02x}{:02x}{:02x}", fg.r(), fg.g(), fg.b());
            let bg_hex = format!("#{:02x}{:02x}{:02x}", bg.r(), bg.g(), bg.b());
            if run_started && (fg_hex != run_fg || bg_hex != run_bg || bold != run_bold) {
                flush(
                    &mut html,
                    &mut run_chars,
                    &run_fg,
                    &run_bg,
                    run_bold,
                    &mut run_started,
                );
            }
            if !run_started {
                run_fg = fg_hex;
                run_bg = bg_hex;
                run_bold = bold;
                run_started = true;
            }
            // Push the FULL escaped entity for structural characters
            // (taking only the first char would truncate "&lt;" to "&").
            run_chars.push_str(&html_escape(&ch.to_string()));
        }
        flush(
            &mut html,
            &mut run_chars,
            &run_fg,
            &run_bg,
            run_bold,
            &mut run_started,
        );
        lines.push(html);
    }

    // Cursor: absolute grid point -> viewport coordinates (clamped).
    let cursor = &grid.cursor;
    let cur_line = cursor.point.line.0 - viewport_start;
    let cur_x = cursor.point.column.0.min(cols.saturating_sub(1));
    let cur_y = (cur_line.max(0) as usize).min(rows.saturating_sub(1));

    TermFrame {
        seq,
        cols,
        rows,
        cur_x,
        cur_y,
        lines,
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
    fn html_escape_covers_the_structural_four() {
        assert_eq!(html_escape("<a&b>\"c"), "&lt;a&amp;b&gt;&quot;c");
    }

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
    fn url_builds_with_token() {
        assert_eq!(
            remote_url("192.168.1.5", 47822, "tok"),
            "http://192.168.1.5:47822/?token=tok"
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

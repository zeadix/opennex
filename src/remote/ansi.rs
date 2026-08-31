//! ANSI escape-stream serializer for the phone remote (v2).
//!
//! Instead of HTML spans (v1), the grid is rendered into the SAME escape
//! sequences xterm.js natively consumes: cursor positioning, SGR style
//! runs and true-color codes. Every cell flag alacritty tracks (inverse,
//! bold, dim, italic, underline, strikeout, hidden) maps to its SGR
//! code, so full-screen TUIs (opencode/vim) render pixel-faithful.

use serde::Serialize;

use alacritty_terminal::grid::{Dimensions, Grid};
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::TermMode;

/// One serialized frame. `d` holds the raw ANSI stream; the phone page
/// writes it straight into xterm.js.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FrameMsg {
    pub seq: u64,
    pub cols: usize,
    pub rows: usize,
    pub cx: usize,
    pub cy: usize,
    /// Alternate screen active (TUI fullscreen): the page switches
    /// xterm's buffer so scrollback stays clean.
    pub alt: bool,
    /// DECCKM application cursor mode: xterm then emits SS3 arrow keys.
    pub app_cursor: bool,
    pub d: String,
}

impl FrameMsg {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".into())
    }
}

/// Style attributes of one cell, decoupled from alacritty types so the
/// SGR emitter is unit-testable without a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellStyle {
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub inverse: bool,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikeout: bool,
    pub hidden: bool,
}

impl CellStyle {
    pub fn plain() -> Self {
        Self {
            fg: (200, 200, 200),
            bg: (12, 13, 16),
            inverse: false,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            strikeout: false,
            hidden: false,
        }
    }
}

/// SGR sequence for a style (always starts with a reset so runs are
/// self-contained). INVERSE swaps fg/bg, mirroring the desktop
/// renderer's resolved_cell_colors; HIDDEN blanks the foreground.
pub fn sgr(style: &CellStyle) -> String {
    let (fg, bg) = if style.inverse {
        (style.bg, style.fg)
    } else {
        (style.fg, style.bg)
    };
    let fg = if style.hidden { style.bg } else { fg };
    let mut codes = vec!["0".to_string()];
    if style.bold {
        codes.push("1".into());
    }
    if style.dim {
        codes.push("2".into());
    }
    if style.italic {
        codes.push("3".into());
    }
    if style.underline {
        codes.push("4".into());
    }
    if style.inverse {
        codes.push("7".into());
    }
    if style.hidden {
        codes.push("8".into());
    }
    if style.strikeout {
        codes.push("9".into());
    }
    codes.push(format!("38;2;{};{};{}", fg.0, fg.1, fg.2));
    codes.push(format!("48;2;{};{};{}", bg.0, bg.1, bg.2));
    format!("\x1b[{}m", codes.join(";"))
}

/// Serialize one grid row into a style-run ANSI fragment (positioning
/// and erase are the caller's job).
fn row_ansi(cells: &[&Cell], theme: &egui_term::TerminalTheme) -> String {
    let mut out = String::with_capacity(cells.len() * 8);
    let mut run_style: Option<CellStyle> = None;
    let mut run = String::new();
    let flush = |out: &mut String, run: &mut String, style: &Option<CellStyle>| {
        if let Some(style) = style {
            out.push_str(&sgr(style));
            out.push_str(run);
            out.push_str("\x1b[0m");
        }
        run.clear();
    };
    for cell in cells {
        let color = |c: alacritty_terminal::vte::ansi::Color| {
            let c32 = theme.get_color(c);
            (c32.r(), c32.g(), c32.b())
        };
        let style = CellStyle {
            fg: color(cell.fg),
            bg: color(cell.bg),
            inverse: cell.flags.contains(Flags::INVERSE),
            bold: cell.flags.intersects(Flags::BOLD | Flags::BOLD_ITALIC),
            dim: cell.flags.intersects(Flags::DIM | Flags::DIM_BOLD),
            italic: cell.flags.intersects(Flags::ITALIC | Flags::BOLD_ITALIC),
            underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
            strikeout: cell.flags.contains(Flags::STRIKEOUT),
            hidden: cell.flags.contains(Flags::HIDDEN),
        };
        if run_style != Some(style) {
            flush(&mut out, &mut run, &run_style);
            run_style = Some(style);
        }
        run.push(cell.c);
    }
    flush(&mut out, &mut run, &run_style);
    out
}

/// Serialize the VISIBLE screen (viewport at the current display
/// offset) as a full-repaint ANSI stream.
pub fn serialize_frame(
    grid: &Grid<Cell>,
    theme: &egui_term::TerminalTheme,
    terminal_mode: &TermMode,
    seq: u64,
) -> FrameMsg {
    let cols = grid.columns();
    let rows = grid.screen_lines();
    let display_offset = grid.display_offset() as i32;
    let viewport_start = -display_offset;

    let mut data = String::with_capacity(cols * rows * 6);
    for row in 0..rows {
        // Position at column 1, then erase to EOL before the content so
        // shorter rows never keep stale characters.
        data.push_str(&format!("\x1b[{};1H\x1b[K", row + 1));
        let mut cells: Vec<&Cell> = Vec::with_capacity(cols);
        for col in 0..cols {
            cells.push(&grid[Point::new(Line(viewport_start + row as i32), Column(col))]);
        }
        data.push_str(&row_ansi(&cells, theme));
    }
    // Park the cursor at its viewport position (1-based).
    let cursor = &grid.cursor;
    let cur_line = cursor.point.line.0 - viewport_start;
    let cx = cursor.point.column.0.min(cols.saturating_sub(1)) + 1;
    let cy = (cur_line.max(0) as usize).min(rows.saturating_sub(1)) + 1;
    data.push_str(&format!("\x1b[{cy};{cx}H"));

    FrameMsg {
        seq,
        cols,
        rows,
        cx: cx.saturating_sub(1),
        cy: cy.saturating_sub(1),
        alt: terminal_mode.contains(TermMode::ALT_SCREEN),
        app_cursor: terminal_mode.contains(TermMode::APP_CURSOR),
        d: data,
    }
}

/// Scrollback history (excluding the visible screen) as an ANSI stream;
/// the phone seeds its xterm scrollback with it on connect. Only
/// meaningful on the primary screen - alt-screen TUIs have no history.
pub fn serialize_scrollback(
    grid: &Grid<Cell>,
    theme: &egui_term::TerminalTheme,
    max_lines: usize,
    byte_cap: usize,
) -> String {
    let cols = grid.columns();
    let history = (grid.total_lines() - grid.screen_lines()) as i32;
    let skip = (history - max_lines as i32).max(0);
    let mut out = String::new();
    for l in (-history..0).skip(skip as usize) {
        let mut cells: Vec<&Cell> = Vec::with_capacity(cols);
        for col in 0..cols {
            cells.push(&grid[Point::new(Line(l), Column(col))]);
        }
        out.push_str(&row_ansi(&cells, theme));
        out.push_str("\r\n");
        if out.len() >= byte_cap {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sgr_resets_then_lists_attributes() {
        let mut style = CellStyle::plain();
        style.bold = true;
        let s = sgr(&style);
        assert!(s.starts_with("\x1b[0;1;"), "{s}");
        assert!(s.contains("38;2;200;200;200"), "{s}");
        assert!(s.contains("48;2;12;13;16"), "{s}");
    }

    #[test]
    fn sgr_inverse_swaps_colors() {
        let mut style = CellStyle::plain();
        style.inverse = true;
        let s = sgr(&style);
        // fg is the old bg and vice versa; the "7" code is present.
        assert!(s.contains(";7;"), "{s}");
        assert!(s.contains("38;2;12;13;16"), "{s}");
        assert!(s.contains("48;2;200;200;200"), "{s}");
    }

    #[test]
    fn sgr_hidden_blanks_foreground_to_background() {
        let mut style = CellStyle::plain();
        style.hidden = true;
        let s = sgr(&style);
        assert!(s.contains(";8;"), "{s}");
        assert!(s.contains("38;2;12;13;16"), "{s}");
    }

    #[test]
    fn sgr_maps_every_supported_attribute() {
        let mut style = CellStyle::plain();
        style.bold = true;
        style.dim = true;
        style.italic = true;
        style.underline = true;
        style.strikeout = true;
        let s = sgr(&style);
        for code in ["0", "1", "2", "3", "4", "9"] {
            assert!(
                s.contains(&format!(";{code};")) || s.starts_with(&format!("\x1b[{code};")),
                "code {code} missing in {s}"
            );
        }
    }

    #[test]
    fn frame_positions_each_row_and_parks_cursor() {
        // Build a minimal grid through the public API: create with
        // template cells, then write through Index.
        let mut grid: Grid<Cell> = Grid::new(3, 4, 100);
        for row in 0..3 {
            for col in 0..4 {
                grid[Point::new(Line(row), Column(col))] = Cell {
                    c: if row == 0 { 'a' } else { ' ' },
                    ..Cell::default()
                };
            }
        }
        let theme = crate::theme::terminal_theme(&crate::theme::store::default_theme().unwrap());
        let frame = serialize_frame(&grid, &theme, &TermMode::empty(), 7);
        assert_eq!((frame.cols, frame.rows, frame.seq), (4, 3, 7));
        assert!(frame.d.contains("\x1b[1;1H"), "{}", frame.d);
        assert!(frame.d.contains("\x1b[2;1H"), "{}", frame.d);
        assert!(frame.d.contains("\x1b[3;1H"), "{}", frame.d);
        // Ends by parking the cursor (1-based park, 0-based fields).
        assert!(frame.d.ends_with("H"), "{}", frame.d);
        assert!(
            (frame.cy == 2 && frame.cx == 0) || (frame.cy == 0),
            "cy {} cx {}",
            frame.cy,
            frame.cx
        );
        assert!(!frame.alt);
        assert!(!frame.app_cursor);
    }

    #[test]
    fn frame_reports_alt_and_app_cursor_modes() {
        let grid: Grid<Cell> = Grid::new(2, 2, 100);
        let theme = crate::theme::terminal_theme(&crate::theme::store::default_theme().unwrap());
        let mode = TermMode::ALT_SCREEN | TermMode::APP_CURSOR;
        let frame = serialize_frame(&grid, &theme, &mode, 1);
        assert!(frame.alt);
        assert!(frame.app_cursor);
    }

    #[test]
    fn scrollback_serializes_history_rows() {
        let mut grid: Grid<Cell> = Grid::new(1, 4, 100);
        // Grow history by scrolling: write and scroll_display up.
        grid[Point::new(Line(0), Column(0))] = Cell {
            c: 'x',
            ..Cell::default()
        };
        grid.scroll_up(&(Line(0)..Line(1)), 1);
        let theme = crate::theme::terminal_theme(&crate::theme::store::default_theme().unwrap());
        let out = serialize_scrollback(&grid, &theme, 100, 64 * 1024);
        // Whatever the history content, the output is pure ANSI (SGR
        // reset pairs) and honors the byte cap.
        assert!(out.len() <= 64 * 1024 + 4096);
        assert!(out.contains("\r\n"));
    }
}

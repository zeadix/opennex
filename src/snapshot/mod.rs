pub mod state;

use state::{SnapshotCell, TerminalSnapshot};
use wezterm_term::Intensity;

/// Take a snapshot of the current terminal state from wezterm-term
pub fn take_snapshot(term: &mut wezterm_term::TerminalState, working_dir: &str, cols: usize, rows: usize) -> TerminalSnapshot {
    let palette = term.palette();
    let cursor = term.cursor_pos();
    let s = term.screen_mut();

    let mut snapshot_grid: Vec<Vec<SnapshotCell>> = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut grid_row: Vec<SnapshotCell> = Vec::with_capacity(cols);
        for col in 0..cols {
            if let Some(cell) = s.get_cell(col, row as i64) {
                let text = cell.str();
                let attrs = cell.attrs();
                let fg = palette.resolve_fg(attrs.foreground());
                let bg = palette.resolve_bg(attrs.background());
                let ch = text.chars().next().unwrap_or(' ');
                grid_row.push(SnapshotCell {
                    ch,
                    fg: [(fg.0 * 255.0) as u8, (fg.1 * 255.0) as u8, (fg.2 * 255.0) as u8],
                    bg: [(bg.0 * 255.0) as u8, (bg.1 * 255.0) as u8, (bg.2 * 255.0) as u8],
bold: attrs.intensity() == Intensity::Bold,
                italic: attrs.italic(),
                underline: attrs.underline() != wezterm_term::Underline::None,
                });
            } else {
                grid_row.push(SnapshotCell::default());
            }
        }
        snapshot_grid.push(grid_row);
    }

    TerminalSnapshot {
        grid: snapshot_grid,
        cursor: (cursor.x as u16, cursor.y as u16),
        scroll_offset: 0,
        working_directory: working_dir.to_string(),
        terminal_size: (cols as u16, rows as u16),
    }
}

/// Save snapshot to a file
pub fn save_snapshot(snapshot: &TerminalSnapshot, path: &std::path::Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(snapshot)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)?;
    Ok(())
}

/// Load snapshot from a file
pub fn load_snapshot(path: &std::path::Path) -> anyhow::Result<TerminalSnapshot> {
    let json = std::fs::read_to_string(path)?;
    let snapshot: TerminalSnapshot = serde_json::from_str(&json)?;
    Ok(snapshot)
}
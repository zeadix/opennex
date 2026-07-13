pub mod state;

use state::{SnapshotCell, TerminalSnapshot};

/// Take a snapshot of the current terminal state
pub fn take_snapshot(grid: &crate::terminal::Grid, working_dir: &str) -> TerminalSnapshot {
    let cols = grid.cols;
    let rows = grid.rows;
    let cursor = (grid.cursor_col as u16, grid.cursor_row as u16);

    let mut snapshot_grid: Vec<Vec<SnapshotCell>> = Vec::with_capacity(rows);
    for row in 0..rows {
        let mut grid_row: Vec<SnapshotCell> = Vec::with_capacity(cols);
        for col in 0..cols {
            let cell = &grid.cells[row][col];
            grid_row.push(SnapshotCell {
                ch: cell.ch,
                fg: cell.fg,
                bg: cell.bg,
                bold: cell.flags.bold,
                italic: cell.flags.italic,
                underline: cell.flags.underline,
            });
        }
        snapshot_grid.push(grid_row);
    }

    TerminalSnapshot {
        grid: snapshot_grid,
        cursor,
        scroll_offset: 0,
        working_directory: working_dir.to_string(),
        terminal_size: (cols as u16, rows as u16),
    }
}

/// Save snapshot to a file
pub fn save_snapshot(snapshot: &TerminalSnapshot, path: &std::path::Path) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(snapshot)?;
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    std::fs::write(path, json)?;
    Ok(())
}

/// Load snapshot from a file
pub fn load_snapshot(path: &std::path::Path) -> anyhow::Result<TerminalSnapshot> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}
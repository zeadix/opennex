pub mod state;

use egui_term::TerminalBackend;
use state::{SnapshotCell, TerminalSnapshot};

/// Take a snapshot of the current terminal state
pub fn take_snapshot(backend: &mut TerminalBackend, working_dir: &str) -> TerminalSnapshot {
    let content = backend.sync();
    let size = content.terminal_size;
    let cols = size.cell_width as usize;
    let rows = size.cell_height as usize;

    let mut grid: Vec<Vec<SnapshotCell>> = Vec::with_capacity(rows);
    for _row in 0..rows {
        let mut grid_row: Vec<SnapshotCell> = Vec::with_capacity(cols);
        for _col in 0..cols {
            grid_row.push(SnapshotCell::default());
        }
        grid.push(grid_row);
    }

    // Extract data from the terminal grid
    for indexed in content.grid.display_iter() {
        let row = indexed.point.line.0 as usize;
        let col = indexed.point.column.0 as usize;
        if row < rows && col < cols {
            let cell = &mut grid[row][col];
            cell.ch = indexed.c;
            cell.fg = extract_color(indexed.fg);
            cell.bg = extract_color(indexed.bg);
            cell.bold = indexed.cell.flags.contains(alacritty_terminal::term::cell::Flags::BOLD);
            cell.italic = indexed.cell.flags.contains(alacritty_terminal::term::cell::Flags::ITALIC);
            cell.underline = indexed.cell.flags.contains(alacritty_terminal::term::cell::Flags::UNDERLINE);
        }
    }

    // Get cursor position from the grid cursor
    let cursor_point = content.grid.cursor.clone();
    let cursor = (
        cursor_point.point.column.0 as u16,
        cursor_point.point.line.0 as u16,
    );

    TerminalSnapshot {
        grid,
        cursor,
        scroll_offset: 0,
        working_directory: working_dir.to_string(),
        terminal_size: (cols as u16, rows as u16),
    }
}

pub fn extract_color(color: alacritty_terminal::vte::ansi::Color) -> [u8; 3] {
    match color {
        alacritty_terminal::vte::ansi::Color::Named(name) => {
            match name {
                alacritty_terminal::vte::ansi::NamedColor::Black => [0, 0, 0],
                alacritty_terminal::vte::ansi::NamedColor::Red => [255, 0, 0],
                alacritty_terminal::vte::ansi::NamedColor::Green => [0, 255, 0],
                alacritty_terminal::vte::ansi::NamedColor::Yellow => [255, 255, 0],
                alacritty_terminal::vte::ansi::NamedColor::Blue => [0, 0, 255],
                alacritty_terminal::vte::ansi::NamedColor::Magenta => [255, 0, 255],
                alacritty_terminal::vte::ansi::NamedColor::Cyan => [0, 255, 255],
                alacritty_terminal::vte::ansi::NamedColor::White => [255, 255, 255],
                alacritty_terminal::vte::ansi::NamedColor::BrightBlack => [128, 128, 128],
                alacritty_terminal::vte::ansi::NamedColor::BrightRed => [255, 100, 100],
                alacritty_terminal::vte::ansi::NamedColor::BrightGreen => [100, 255, 100],
                alacritty_terminal::vte::ansi::NamedColor::BrightYellow => [255, 255, 100],
                alacritty_terminal::vte::ansi::NamedColor::BrightBlue => [100, 100, 255],
                alacritty_terminal::vte::ansi::NamedColor::BrightMagenta => [255, 100, 255],
                alacritty_terminal::vte::ansi::NamedColor::BrightCyan => [100, 255, 255],
                alacritty_terminal::vte::ansi::NamedColor::BrightWhite => [255, 255, 255],
                _ => [255, 255, 255],
            }
        }
        alacritty_terminal::vte::ansi::Color::Indexed(idx) => {
            let colors: [[u8; 3]; 16] = [
                [0, 0, 0], [128, 0, 0], [0, 128, 0], [128, 128, 0],
                [0, 0, 128], [128, 0, 128], [0, 128, 128], [192, 192, 192],
                [128, 128, 128], [255, 0, 0], [0, 255, 0], [255, 255, 0],
                [0, 0, 255], [255, 0, 255], [0, 255, 255], [255, 255, 255],
            ];
            colors[(idx as usize) % 16]
        }
        _ => [255, 255, 255],
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

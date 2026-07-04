use serde::{Deserialize, Serialize};

/// A single cell in the terminal grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotCell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for SnapshotCell {
    fn default() -> Self {
        SnapshotCell {
            ch: ' ',
            fg: [255, 255, 255],
            bg: [0, 0, 0],
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

/// Complete terminal snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSnapshot {
    /// Screen content as 2D grid of cells
    pub grid: Vec<Vec<SnapshotCell>>,
    /// Cursor position (col, row)
    pub cursor: (u16, u16),
    /// Scroll offset
    pub scroll_offset: i32,
    /// Current working directory
    pub working_directory: String,
    /// Terminal size (cols, rows)
    pub terminal_size: (u16, u16),
}

impl TerminalSnapshot {
    /// Create an empty snapshot with given dimensions
    pub fn empty(cols: u16, rows: u16) -> Self {
        let grid = vec![vec![SnapshotCell::default(); cols as usize]; rows as usize];
        TerminalSnapshot {
            grid,
            cursor: (0, 0),
            scroll_offset: 0,
            working_directory: String::new(),
            terminal_size: (cols, rows),
        }
    }
}

/// Process information for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
    pub args: Vec<String>,
    pub working_directory: String,
}

/// Full scene state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneState {
    pub panels: Vec<PanelState>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    pub name: String,
    pub dock_state: egui_dock::DockState<String>,
    pub terminals: HashMap<String, TerminalState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    pub name: String,
    pub font_size: f32,
    pub working_directory: String,
    pub snapshot: Option<TerminalSnapshot>,
    pub process_info: Option<ProcessInfo>,
}

impl Default for TerminalState {
    fn default() -> Self {
        TerminalState {
            name: String::new(),
            font_size: 14.0,
            working_directory: String::new(),
            snapshot: None,
            process_info: None,
        }
    }
}

use std::collections::HashMap;

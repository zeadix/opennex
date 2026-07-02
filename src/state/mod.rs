pub mod persistence;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub terminals: Vec<TerminalState>,
    pub active_terminal: Option<String>,
    pub layout: LayoutState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalState {
    pub id: String,
    pub name: String,
    pub working_directory: String,
    pub history: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub width: u16,
    pub height: u16,
    pub split_ratio: f32,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            terminals: Vec::new(),
            active_terminal: None,
            layout: LayoutState {
                width: 80,
                height: 24,
                split_ratio: 0.5,
            },
        }
    }
}

impl AppState {
    pub fn load() -> Result<Self> {
        persistence::load_state()
    }

    pub fn save(&self) -> Result<()> {
        persistence::save_state(self)
    }
}

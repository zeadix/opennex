use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub active_tab: usize,
    pub active_pane: Vec<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            active_tab: 0,
            active_pane: Vec::new(),
        }
    }
}

impl AppState {
    pub fn load() -> Result<Self> {
        let path = get_state_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let state: AppState = serde_json::from_str(&content)?;
            Ok(state)
        } else {
            Ok(AppState::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = get_state_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

fn get_state_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("open-zoo")
        .join("state.json")
}

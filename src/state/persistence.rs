use super::AppState;
use crate::config::paths;
use anyhow::Result;

pub fn load_state() -> Result<AppState> {
    let state_path = paths::get_state_path();
    if state_path.exists() {
        let content = std::fs::read_to_string(&state_path)?;
        let state: AppState = serde_json::from_str(&content)?;
        Ok(state)
    } else {
        Ok(AppState::default())
    }
}

pub fn save_state(state: &AppState) -> Result<()> {
    let state_path = paths::get_state_path();
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(state)?;
    std::fs::write(&state_path, content)?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    pub terminal_id: String,
    pub command: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandHistory {
    pub histories: Vec<HistoryEntry>,
}

impl Default for CommandHistory {
    fn default() -> Self {
        CommandHistory {
            histories: Vec::new(),
        }
    }
}

pub fn load_command_history() -> Result<CommandHistory> {
    let history_path = paths::get_history_path();
    if history_path.exists() {
        let content = std::fs::read_to_string(&history_path)?;
        let history: CommandHistory = serde_json::from_str(&content)?;
        Ok(history)
    } else {
        Ok(CommandHistory::default())
    }
}

pub fn save_command_history(history: &CommandHistory) -> Result<()> {
    let history_path = paths::get_history_path();
    if let Some(parent) = history_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(history)?;
    std::fs::write(&history_path, content)?;
    Ok(())
}

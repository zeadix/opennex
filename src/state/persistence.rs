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

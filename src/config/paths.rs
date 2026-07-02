use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("open-zoo")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.json")
}

pub fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("open-zoo")
}

pub fn get_state_path() -> PathBuf {
    get_data_dir().join("state.json")
}

pub fn get_history_path() -> PathBuf {
    get_data_dir().join("history.json")
}

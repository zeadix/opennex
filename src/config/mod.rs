pub mod paths;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub keybindings: KeybindingsConfig,
    pub appearance: AppearanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_shell: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    pub quit: String,
    pub new_terminal: String,
    pub close_terminal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String,
    pub font_size: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            general: GeneralConfig {
                default_shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()),
                startup_command: None,
            },
            keybindings: KeybindingsConfig {
                quit: "Ctrl+Q".to_string(),
                new_terminal: "Ctrl+N".to_string(),
                close_terminal: "Ctrl+W".to_string(),
            },
            appearance: AppearanceConfig {
                theme: "default".to_string(),
                font_size: 14,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = paths::get_config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = paths::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

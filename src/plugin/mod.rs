use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugins: HashMap<String, Plugin>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        PluginConfig {
            plugins: HashMap::new(),
        }
    }
}

impl PluginConfig {
    pub fn new() -> Self {
        PluginConfig::default()
    }

    pub fn add_plugin(&mut self, plugin: Plugin) {
        self.plugins.insert(plugin.id.clone(), plugin);
    }

    pub fn remove_plugin(&mut self, plugin_id: &str) -> Option<Plugin> {
        self.plugins.remove(plugin_id)
    }

    pub fn get_plugin(&self, plugin_id: &str) -> Option<&Plugin> {
        self.plugins.get(plugin_id)
    }

    pub fn enable_plugin(&mut self, plugin_id: &str) -> bool {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.enabled = true;
            true
        } else {
            false
        }
    }

    pub fn disable_plugin(&mut self, plugin_id: &str) -> bool {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.enabled = false;
            true
        } else {
            false
        }
    }

    pub fn list_enabled_plugins(&self) -> Vec<&Plugin> {
        self.plugins.values().filter(|p| p.enabled).collect()
    }

    pub fn save(&self) -> Result<()> {
        let config_path = crate::config::paths::get_config_dir().join("plugins.json");
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let config_path = crate::config::paths::get_config_dir().join("plugins.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: PluginConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(PluginConfig::default())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub command: String,
    pub description: String,
    pub handler: String,
}

pub trait PluginInterface {
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_commands(&self) -> Vec<PluginCommand>;
    fn execute_command(&self, command: &str, args: &[String]) -> Result<String>;
}

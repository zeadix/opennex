use open_zoo::terminal::TerminalManager;
use open_zoo::config::AppConfig;
use open_zoo::state::AppState;
use open_zoo::state::persistence::{CommandHistory, HistoryEntry};
use open_zoo::plugin::{PluginConfig, Plugin};
use open_zoo::template::TemplateConfig;

#[tokio::test]
async fn test_terminal_manager_create() {
    let mut manager = TerminalManager::new();
    let id = manager.create_terminal("test").await.unwrap();
    assert!(id.starts_with("terminal-"));

    let terminal = manager.get_terminal(&id).await.unwrap();
    assert_eq!(terminal.name, "test");
}

#[tokio::test]
async fn test_terminal_manager_list() {
    let mut manager = TerminalManager::new();
    manager.create_terminal("test1").await.unwrap();
    manager.create_terminal("test2").await.unwrap();

    let terminals = manager.list_terminals().await;
    assert_eq!(terminals.len(), 2);
}

#[test]
fn test_config_load() {
    let config = AppConfig::load().unwrap();
    assert!(!config.general.default_shell.is_empty());
}

#[test]
fn test_state_load() {
    let state = AppState::load().unwrap();
    assert!(state.terminals.is_empty());
}

#[test]
fn test_command_history_persistence() {
    let mut history = CommandHistory::default();
    history.histories.push(HistoryEntry {
        terminal_id: "terminal-1".to_string(),
        command: "ls -la".to_string(),
        timestamp: 1234567890,
    });

    let json = serde_json::to_string(&history).unwrap();
    assert!(json.contains("ls -la"));

    let restored: CommandHistory = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.histories.len(), 1);
    assert_eq!(restored.histories[0].command, "ls -la");
}

#[test]
fn test_plugin_config() {
    let mut config = PluginConfig::new();
    config.add_plugin(Plugin {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        enabled: true,
    });

    assert_eq!(config.plugins.len(), 1);
    assert!(config.get_plugin("test-plugin").is_some());

    let enabled = config.list_enabled_plugins();
    assert_eq!(enabled.len(), 1);

    config.disable_plugin("test-plugin");
    let enabled = config.list_enabled_plugins();
    assert_eq!(enabled.len(), 0);
}

#[test]
fn test_template_config() {
    let config = TemplateConfig::new();
    assert!(!config.templates.is_empty());

    let frontend = config.get_template("frontend-dev");
    assert!(frontend.is_some());
    let frontend = frontend.unwrap();
    assert_eq!(frontend.name, "前端开发");

    let backend = config.get_template("backend-dev");
    assert!(backend.is_some());

    let data_science = config.get_template("data-science");
    assert!(data_science.is_some());

    let web_templates = config.search_by_tag("web");
    assert_eq!(web_templates.len(), 1);
}

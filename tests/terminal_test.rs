use open_zoo::terminal::TerminalManager;
use open_zoo::config::AppConfig;
use open_zoo::state::AppState;

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

use open_zoo::terminal::TerminalManager;
use open_zoo::config::AppConfig;
use open_zoo::state::AppState;
use open_zoo::state::persistence::{LayoutState, CommandHistory, HistoryEntry};
use open_zoo::ui::layout_tree::{LayoutNode, SplitDirection};
use open_zoo::ui::tab_group::{TabGroup, Tab};
use open_zoo::ui::presets::{get_presets, get_preset_by_name};

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
fn test_layout_tree_split() {
    let mut layout = LayoutNode::default();
    let result = layout.split_pane("terminal-1", SplitDirection::Horizontal, "terminal-2", "终端 2");
    assert!(result);

    let pane_ids = layout.get_all_pane_ids();
    assert_eq!(pane_ids.len(), 2);
    assert!(pane_ids.contains(&"terminal-1".to_string()));
    assert!(pane_ids.contains(&"terminal-2".to_string()));
}

#[test]
fn test_layout_tree_remove() {
    let mut layout = LayoutNode::default();
    layout.split_pane("terminal-1", SplitDirection::Horizontal, "terminal-2", "终端 2");

    let result = layout.remove_pane("terminal-2");
    assert!(result);

    let pane_ids = layout.get_all_pane_ids();
    assert_eq!(pane_ids.len(), 1);
}

#[test]
fn test_tab_group_add_remove() {
    let mut group = TabGroup::new("test-group");
    group.add_tab(Tab {
        id: "tab-1".to_string(),
        title: "Tab 1".to_string(),
        terminal_id: "terminal-1".to_string(),
    });
    group.add_tab(Tab {
        id: "tab-2".to_string(),
        title: "Tab 2".to_string(),
        terminal_id: "terminal-2".to_string(),
    });

    assert_eq!(group.tabs.len(), 2);

    let removed = group.remove_tab("tab-1");
    assert!(removed.is_some());
    assert_eq!(group.tabs.len(), 1);
}

#[test]
fn test_tab_group_rename() {
    let mut group = TabGroup::new("test-group");
    group.add_tab(Tab {
        id: "tab-1".to_string(),
        title: "Old Name".to_string(),
        terminal_id: "terminal-1".to_string(),
    });

    let result = group.rename_tab("tab-1", "New Name");
    assert!(result);
    assert_eq!(group.tabs[0].title, "New Name");
}

#[test]
fn test_layout_state_persistence() {
    let mut state = LayoutState::default();
    state.layout.split_pane("terminal-1", SplitDirection::Horizontal, "terminal-2", "终端 2");

    // 测试序列化
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("terminal-1"));
    assert!(json.contains("terminal-2"));

    // 测试反序列化
    let restored: LayoutState = serde_json::from_str(&json).unwrap();
    let pane_ids = restored.layout.get_all_pane_ids();
    assert_eq!(pane_ids.len(), 2);
}

#[test]
fn test_command_history_persistence() {
    let mut history = CommandHistory::default();
    history.histories.push(HistoryEntry {
        terminal_id: "terminal-1".to_string(),
        command: "ls -la".to_string(),
        timestamp: 1234567890,
    });

    // 测试序列化
    let json = serde_json::to_string(&history).unwrap();
    assert!(json.contains("ls -la"));

    // 测试反序列化
    let restored: CommandHistory = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.histories.len(), 1);
    assert_eq!(restored.histories[0].command, "ls -la");
}

#[test]
fn test_layout_presets() {
    let presets = get_presets();
    assert!(!presets.is_empty());

    let single = get_preset_by_name("single");
    assert!(single.is_some());
    let single = single.unwrap();
    assert_eq!(single.name, "single");

    let grid = get_preset_by_name("grid-4");
    assert!(grid.is_some());
    let grid = grid.unwrap();
    let pane_ids = grid.layout.get_all_pane_ids();
    assert_eq!(pane_ids.len(), 4);
}

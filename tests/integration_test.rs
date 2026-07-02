use open_zoo::terminal::{Pane, Tab};
use open_zoo::app::App;

#[test]
fn test_pane_execute() {
    let mut pane = Pane::new("test");
    pane.execute("echo hello world");
    assert!(pane.content.contains("hello world"));
}

#[test]
fn test_pane_execute_help() {
    let mut pane = Pane::new("test");
    pane.execute("help");
    assert!(pane.content.contains("Available commands"));
}

#[test]
fn test_pane_execute_clear() {
    let mut pane = Pane::new("test");
    pane.execute("echo test");
    assert!(!pane.content.is_empty());
    pane.execute("clear");
    assert!(pane.content.is_empty());
}

#[test]
fn test_pane_execute_pwd() {
    let mut pane = Pane::new("test");
    pane.execute("pwd");
    let cwd = std::env::current_dir().unwrap().to_string_lossy().to_string();
    assert!(pane.content.contains(&cwd));
}

#[test]
fn test_pane_execute_calc() {
    let mut pane = Pane::new("test");
    pane.execute("calc 2 + 3");
    assert!(pane.content.contains("5"));
    pane.execute("calc 10 / 2");
    assert!(pane.content.contains("5"));
}

#[test]
fn test_tab_split() {
    let mut tab = Tab::new("test");
    assert_eq!(tab.panes.len(), 1);
    tab.split_horizontal();
    assert_eq!(tab.panes.len(), 2);
    assert_eq!(tab.active_pane, 1);
}

#[test]
fn test_tab_close_pane() {
    let mut tab = Tab::new("test");
    tab.split_horizontal();
    assert_eq!(tab.panes.len(), 2);
    tab.close_pane();
    assert_eq!(tab.panes.len(), 1);
}

#[test]
fn test_app_new_tab() {
    let mut app = App::new();
    assert_eq!(app.tabs.len(), 1);
    app.new_tab();
    assert_eq!(app.tabs.len(), 2);
}

#[test]
fn test_app_close_tab() {
    let mut app = App::new();
    app.new_tab();
    assert_eq!(app.tabs.len(), 2);
    app.close_tab();
    assert_eq!(app.tabs.len(), 1);
}

#[test]
fn test_app_split() {
    let mut app = App::new();
    app.split_horizontal();
    assert_eq!(app.tabs[0].panes.len(), 2);
    app.split_vertical();
    assert_eq!(app.tabs[0].panes.len(), 3);
}

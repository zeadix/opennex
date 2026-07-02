use super::KeyBinding;
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

pub fn get_default_bindings() -> HashMap<String, KeyBinding> {
    let mut bindings = HashMap::new();

    // 应用控制
    bindings.insert("quit".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('q')));
    bindings.insert("quit_alt".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('c')));

    // 终端切换
    for i in 1..=9 {
        bindings.insert(
            format!("switch_terminal_{}", i),
            (KeyModifiers::CONTROL, KeyCode::Char(std::char::from_digit(i, 10).unwrap())),
        );
    }

    // 终端操作
    bindings.insert("new_terminal".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('n')));
    bindings.insert("close_terminal".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('w')));

    // 分屏操作 (Phase 2)
    bindings.insert("split_horizontal".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('\\')));
    bindings.insert("split_vertical".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('|')));

    // 选项卡操作 (Phase 2)
    bindings.insert("new_tab".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('t')));
    bindings.insert("close_tab".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('w')));

    // 窗口操作
    bindings.insert("toggle_fullscreen".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('f')));
    bindings.insert("toggle_statusbar".to_string(), (KeyModifiers::CONTROL, KeyCode::Char('b')));

    bindings
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Texts {
    pub display_name: String,
    pub menu: MenuTexts,
    pub file_menu: FileMenuTexts,
    pub view_menu: ViewMenuTexts,
    pub settings: SettingsTexts,
    pub shortcut_labels: ShortcutLabelTexts,
    pub workspace: WorkspaceTexts,
    pub close_confirm: CloseConfirmTexts,
    pub password: PasswordTexts,
    pub lock_overlay: LockOverlayTexts,
    pub terminal: TerminalTexts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuTexts {
    pub file: String,
    pub view: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMenuTexts {
    pub save: String,
    pub load: String,
    pub save_as: String,
    pub exit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ViewMenuTexts {
    pub split_right: String,
    pub split_down: String,
    pub settings: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsTexts {
    pub title: String,
    pub tabs: SettingsTabsTexts,
    pub general: SettingsGeneralTexts,
    pub appearance: SettingsAppearanceTexts,
    pub shortcuts: SettingsShortcutsTexts,
    pub lock: SettingsLockTexts,
    pub buttons: SettingsButtonsTexts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsTabsTexts {
    pub general: String,
    pub appearance: String,
    pub shortcuts: String,
    pub lock: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsGeneralTexts {
    pub heading: String,
    pub scene_info: String,
    pub scene_path: String,
    pub templates_path: String,
    pub history_section: String,
    pub max_history: String,
    pub scrollback: String,
    pub clear_all_history: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsAppearanceTexts {
    pub heading: String,
    pub terminal_section: String,
    pub font_size: String,
    pub cell_spacing: String,
    pub font_family: String,
    pub bg_color: String,
    pub fg_color: String,
    pub command_menu_section: String,
    pub menu_bg_color: String,
    pub menu_fg_color: String,
    pub menu_font_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsShortcutsTexts {
    pub heading: String,
    pub hint: String,
    pub not_set: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsLockTexts {
    pub heading: String,
    pub password_section: String,
    pub set_password: String,
    pub change_password: String,
    pub clear_password: String,
    pub lock_section: String,
    pub lock_overlay_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsButtonsTexts {
    pub apply: String,
    pub close: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShortcutLabelTexts {
    pub new_terminal: String,
    pub close_terminal: String,
    pub workspace_up: String,
    pub workspace_down: String,
    pub panel_left: String,
    pub panel_right: String,
    pub lock_workspace: String,
    pub history_menu: String,
    pub history_prev: String,
    pub history_next: String,
    pub shortcuts_heading: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceTexts {
    pub heading: String,
    pub new: String,
    pub rename: String,
    pub save_as_template: String,
    pub close: String,
    pub rename_confirm: String,
    pub rename_cancel: String,
    pub drag_handle_hint: String,
    pub close_ws_hint: String,
    pub locked_hint: String,
    pub unlocked_hint: String,
    pub empty_hint: String,
    pub templates: String,
    pub templates_empty: String,
    pub name_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloseConfirmTexts {
    pub message_prefix: String,
    pub message_suffix: String,
    pub confirm: String,
    pub cancel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasswordTexts {
    pub enter: String,
    pub confirm: String,
    pub original: String,
    pub new: String,
    pub confirm_new: String,
    pub input_label: String,
    pub mismatch: String,
    pub r#match: String,
    pub empty_error: String,
    pub mismatch_error: String,
    pub wrong_error: String,
    pub wrong_password: String,
    pub confirm_button: String,
    pub cancel_button: String,
    pub set_title: String,
    pub change_title: String,
    pub clear_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockOverlayTexts {
    pub title: String,
    pub password_label: String,
    pub unlock_button: String,
    pub wrong_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalTexts {
    pub not_found: String,
    pub rename: String,
    pub clear_history: String,
    pub new_tab: String,
    pub add_tab: String,
    pub rename_hint: String,
    pub default_name_prefix: String,
}

impl Texts {
    pub fn zh_default() -> Self {
        Self {
            display_name: "中文".into(),
            menu: MenuTexts {
                file: "文件".into(),
                view: "视图".into(),
                language: "语言".into(),
            },
            file_menu: FileMenuTexts {
                save: "保存".into(),
                load: "加载".into(),
                save_as: "另存为...".into(),
                exit: "退出".into(),
            },
            view_menu: ViewMenuTexts {
                split_right: "向右分屏".into(),
                split_down: "向下分屏".into(),
                settings: "设置".into(),
            },
            settings: SettingsTexts {
                title: "设置".into(),
                tabs: SettingsTabsTexts {
                    general: "通用".into(),
                    appearance: "外观".into(),
                    shortcuts: "快捷键".into(),
                    lock: "锁定".into(),
                },
                general: SettingsGeneralTexts {
                    heading: "通用".into(),
                    scene_info: "场景和模板使用固定路径:".into(),
                    scene_path: "  场景: ./scene.json".into(),
                    templates_path: "  模板: ./templates/".into(),
                    history_section: "历史记录".into(),
                    max_history: "最大条数:".into(),
                    scrollback: "滚动回溯:".into(),
                    clear_all_history: "清空所有历史".into(),
                },
                appearance: SettingsAppearanceTexts {
                    heading: "外观".into(),
                    terminal_section: "终端外观".into(),
                    font_size: "字号:".into(),
                    cell_spacing: "字间距:".into(),
                    font_family: "字体:".into(),
                    bg_color: "背景色:".into(),
                    fg_color: "前景色:".into(),
                    command_menu_section: "指令菜单".into(),
                    menu_bg_color: "背景色:".into(),
                    menu_fg_color: "文字色:".into(),
                    menu_font_size: "字号:".into(),
                },
                shortcuts: SettingsShortcutsTexts {
                    heading: "快捷键".into(),
                    hint: "点击快捷键名称后按下新按键即可修改".into(),
                    not_set: "未设置".into(),
                },
                lock: SettingsLockTexts {
                    heading: "锁定".into(),
                    password_section: "密码配置:".into(),
                    set_password: "设置密码".into(),
                    change_password: "修改密码".into(),
                    clear_password: "清除密码".into(),
                    lock_section: "锁定:".into(),
                    lock_overlay_color: "遮罩色:".into(),
                },
                buttons: SettingsButtonsTexts {
                    apply: "应用".into(),
                    close: "关闭".into(),
                },
            },
            shortcut_labels: ShortcutLabelTexts {
                new_terminal: "新建终端".into(),
                close_terminal: "关闭终端".into(),
                workspace_up: "工作区上移".into(),
                workspace_down: "工作区下移".into(),
                panel_left: "面板左移".into(),
                panel_right: "面板右移".into(),
                lock_workspace: "锁定工作区".into(),
                history_menu: "历史菜单".into(),
                history_prev: "历史上一条".into(),
                history_next: "历史下一条".into(),
                shortcuts_heading: "快捷键".into(),
            },
            workspace: WorkspaceTexts {
                heading: "工作区".into(),
                new: "+ 新建工作区".into(),
                rename: "重命名".into(),
                save_as_template: "保存为模版".into(),
                close: "关闭".into(),
                rename_confirm: "确定".into(),
                rename_cancel: "取消".into(),
                drag_handle_hint: "拖动以调整 Workspace 顺序".into(),
                close_ws_hint: "关闭 Workspace".into(),
                locked_hint: "已锁定 Workspace".into(),
                unlocked_hint: "未锁定 Workspace".into(),
                empty_hint: "点击 '+ 新建工作区' 创建一个".into(),
                templates: "模板".into(),
                templates_empty: "(空)".into(),
                name_prefix: "工作区 ".into(),
            },
            close_confirm: CloseConfirmTexts {
                message_prefix: "确定要关闭工作区「".into(),
                message_suffix: "」吗？".into(),
                confirm: "确认".into(),
                cancel: "取消".into(),
            },
            password: PasswordTexts {
                enter: "输入密码:".into(),
                confirm: "确认密码:".into(),
                original: "原密码:".into(),
                new: "新密码:".into(),
                confirm_new: "确认新密码:".into(),
                input_label: "密码:".into(),
                mismatch: "不一致".into(),
                r#match: "一致".into(),
                empty_error: "密码不能为空".into(),
                mismatch_error: "两次输入的新密码不一致".into(),
                wrong_error: "原密码错误".into(),
                wrong_password: "密码错误".into(),
                confirm_button: "确认".into(),
                cancel_button: "取消".into(),
                set_title: "设置密码".into(),
                change_title: "修改密码".into(),
                clear_title: "清除密码".into(),
            },
            lock_overlay: LockOverlayTexts {
                title: "🔒 此工作区已锁定".into(),
                password_label: "密码:".into(),
                unlock_button: "解锁".into(),
                wrong_password: "密码错误".into(),
            },
            terminal: TerminalTexts {
                not_found: "未找到终端".into(),
                rename: "重命名".into(),
                clear_history: "清空指令历史".into(),
                new_tab: "+ 新建标签页".into(),
                add_tab: "+ 标签页".into(),
                rename_hint: "输入名称...".into(),
                default_name_prefix: "终端 ".into(),
            },
        }
    }

    pub fn load_from_yaml(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        match serde_yaml::from_str::<Self>(&content) {
            Ok(texts) => Some(texts),
            Err(e) => {
                eprintln!("Failed to parse language file {:?}: {}", path, e);
                None
            }
        }
    }
}

pub fn locales_dir() -> PathBuf {
    PathBuf::from("locales")
}

pub fn scan_available_languages() -> Vec<(String, String)> {
    let dir = locales_dir();
    let mut languages = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return vec![("zh".to_string(), "中文".to_string())],
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let code = match path.file_stem().and_then(|s| s.to_str()) {
            Some(c) => c.to_string(),
            None => continue,
        };
        if let Some(texts) = Texts::load_from_yaml(&path) {
            languages.push((code, texts.display_name));
        }
    }

    if languages.is_empty() {
        vec![("zh".to_string(), "中文".to_string())]
    } else {
        languages
    }
}

pub fn load_language(code: &str) -> Texts {
    let path = locales_dir().join(format!("{}.yaml", code));
    if path.exists() {
        if let Some(texts) = Texts::load_from_yaml(&path) {
            return texts;
        }
    }
    Texts::zh_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zh_default_contains_all_fields() {
        let texts = Texts::zh_default();
        assert!(!texts.display_name.is_empty());
        assert!(!texts.menu.file.is_empty());
        assert!(!texts.workspace.heading.is_empty());
        assert!(!texts.terminal.rename.is_empty());
    }

    #[test]
    fn yaml_roundtrip_preserves_display_name() {
        let texts = Texts::zh_default();
        let yaml = serde_yaml::to_string(&texts).unwrap();
        let parsed: Texts = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.display_name, texts.display_name);
        assert_eq!(parsed.menu.file, texts.menu.file);
        assert_eq!(parsed.workspace.heading, texts.workspace.heading);
    }

    #[test]
    fn load_language_falls_back_to_default_for_missing_code() {
        let texts = load_language("__nonexistent__");
        assert_eq!(texts.display_name, "中文");
    }
}

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
    pub theme: ThemeTexts,
    pub about: AboutTexts,
    pub update: UpdateTexts,
    pub theme_editor: ThemeEditorTexts,
    pub stats: StatsTexts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuTexts {
    pub workspace: String,
    pub view: String,
    pub language: String,
    pub theme: String,
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
    pub workspace_toggle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsTexts {
    pub title: String,
    pub tabs: SettingsTabsTexts,
    pub nav: SettingsNavTexts,
    pub general: SettingsGeneralTexts,
    pub appearance: SettingsAppearanceTexts,
    pub shortcuts: SettingsShortcutsTexts,
    pub lock: SettingsLockTexts,
    pub buttons: SettingsButtonsTexts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsTabsTexts {
    pub general: String,
    pub themes: String,
    pub shortcuts: String,
    pub lock: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsNavTexts {
    pub general: String,
    pub themes: String,
    pub shortcuts: String,
    pub lock: String,
    #[serde(default)]
    pub group_settings: String,
    #[serde(default)]
    pub group_theme: String,
    #[serde(default)]
    pub theme_select: String,
    #[serde(default)]
    pub theme_ui: String,
    #[serde(default)]
    pub theme_terminal: String,
    #[serde(default)]
    pub theme_ansi: String,
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
    #[serde(default)]
    pub auto_copy: String,
    pub auto_match: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsAppearanceTexts {
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub terminal_section: String,
    #[serde(default)]
    pub font_size: String,
    #[serde(default)]
    pub cell_spacing: String,
    #[serde(default)]
    pub font_family: String,
    #[serde(default)]
    pub bg_color: String,
    #[serde(default)]
    pub fg_color: String,
    #[serde(default)]
    pub terminal_theme: String,
    #[serde(default)]
    pub command_menu_section: String,
    #[serde(default)]
    pub menu_bg_color: String,
    #[serde(default)]
    pub menu_fg_color: String,
    #[serde(default)]
    pub menu_font_size: String,
    #[serde(default)]
    pub apply_theme_typography: String,
    #[serde(default)]
    pub import_theme: String,
    #[serde(default)]
    pub export_theme: String,
    #[serde(default)]
    pub import_success: String,
    #[serde(default)]
    pub export_success: String,
    #[serde(default)]
    pub invalid_theme: String,
    #[serde(default)]
    pub unsupported_version: String,
    #[serde(default)]
    pub save_failure: String,
    #[serde(default)]
    pub subtab_ui: String,
    #[serde(default)]
    pub subtab_terminal: String,
    #[serde(default)]
    pub subtab_ansi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsShortcutsTexts {
    pub heading: String,
    pub hint: String,
    pub not_set: String,
    pub reset_defaults: String,
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
    #[serde(default)]
    pub revert: String,
    #[serde(default)]
    pub applied: String,
    #[serde(default)]
    pub builtin: String,
    pub user_group: String,
    pub builtin_group: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub behavior_section: String,
    #[serde(default)]
    pub data_section: String,
    #[serde(default)]
    pub maintenance_section: String,
    #[serde(default)]
    pub font_section: String,
    #[serde(default)]
    pub color_section: String,
    #[serde(default)]
    pub preview_section: String,
    #[serde(default)]
    pub template_section: String,
    #[serde(default)]
    pub base_color_section: String,
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
    pub toggle_workspace_sidebar: String,
    pub zoom_in: String,
    pub zoom_out: String,
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
    pub terminal_title: String,
    pub terminal_message: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeTexts {
    pub light: String,
    pub dark: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AboutTexts {
    pub menu_label: String,
    pub title: String,
    pub version_label: String,
    pub description: String,
    pub homepage_label: String,
    pub source_label: String,
    pub license_label: String,
    pub credits_label: String,
    pub credits: String,
    pub close: String,
}

/// About-window update section strings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTexts {
    pub checking: String,
    pub downloading: String,
    pub verifying: String,
    pub ready: String,
    pub failed: String,
    pub available: String,
    pub up_to_date: String,
    pub check: String,
    pub update_now: String,
    pub restart: String,
    pub restart_title: String,
    pub restart_body: String,
    pub restart_confirm: String,
}

/// Theme editor color names.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ColorNameTexts {
    pub app_bg: String,
    pub sidebar: String,
    pub panel: String,
    pub input_bg: String,
    pub text: String,
    pub weak_text: String,
    pub accent: String,
    pub warning: String,
    pub danger: String,
    pub hover: String,
    pub active: String,
    pub selection_bg: String,
    pub selection_text: String,
    pub border: String,
    pub lock: String,
    pub window_shadow: String,
    pub tab_highlight: String,
    pub fg: String,
    pub bg: String,
    pub cursor: String,
    pub selection_term_bg: String,
    pub selection_term_text: String,
    pub link: String,
}

/// Theme editor popup strings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeEditorTexts {
    pub colors: ColorNameTexts,
    pub edit_title: String,
    pub name_label: String,
    pub confirm: String,
    pub cancel: String,
    pub system_ui: String,
    pub terminal: String,
    pub ui_font_label: String,
    pub ui_font_size: String,
    pub terminal_font_label: String,
    pub terminal_font_size: String,
    pub cell_spacing: String,
    pub terminal_padding: String,
}

/// Sidebar system-info strings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsTexts {
    pub copied_toast: String,
    pub focused: String,
    pub workspace: String,
    pub global: String,
    pub terminals: String,
    pub clear_history_title: String,
    pub clear_history_body: String,
}

impl Texts {
    pub fn zh_default() -> Self {
        Self {
            display_name: "中文".into(),
            menu: MenuTexts {
                workspace: "工作区".into(),
                view: "视图".into(),
                language: "语言".into(),
                theme: "主题".into(),
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
                workspace_toggle: "工作区".into(),
            },
            settings: SettingsTexts {
                title: "设置".into(),
                tabs: SettingsTabsTexts {
                    general: "通用".into(),
                    themes: "主题".into(),
                    shortcuts: "快捷键".into(),
                    lock: "锁定".into(),
                },
                nav: SettingsNavTexts {
                    general: "通用".into(),
                    themes: "主题".into(),
                    shortcuts: "快捷键".into(),
                    lock: "锁定".into(),
                    group_settings: "设置".into(),
                    group_theme: "主题".into(),
                    theme_select: "选择与管理".into(),
                    theme_ui: "UI 外观".into(),
                    theme_terminal: "终端".into(),
                    theme_ansi: "ANSI 调色板".into(),
                },
                general: SettingsGeneralTexts {
                    heading: "通用".into(),
                    scene_info: "场景和模板使用固定路径:".into(),
                    scene_path: "  场景: ./scene.json".into(),
                    templates_path: "  模板: ./templates/".into(),
                    history_section: "历史记录".into(),
                    max_history: "最大条数:".into(),
                    scrollback: "滚动回溯:".into(),
                    clear_all_history: "删除所有指令记录".into(),
                    auto_copy: "选中字符自动复制".into(),
                auto_match: "自动匹配指令".into(),
                },
                appearance: SettingsAppearanceTexts {
                    heading: "外观".into(),
                    terminal_section: "终端外观".into(),
                    font_size: "字号:".into(),
                    cell_spacing: "字间距:".into(),
                    font_family: "字体:".into(),
                    bg_color: "背景色:".into(),
                    fg_color: "前景色:".into(),
                    terminal_theme: "终端配色方案:".into(),
                    command_menu_section: "指令菜单".into(),
                    menu_bg_color: "背景色:".into(),
                    menu_fg_color: "文字色:".into(),
                    menu_font_size: "字号:".into(),
                    apply_theme_typography: "应用主题字体和字号".into(),
                    import_theme: "导入主题".into(),
                    export_theme: "导出主题".into(),
                    import_success: "主题已导入".into(),
                    export_success: "主题已导出".into(),
                    invalid_theme: "无效的主题文件".into(),
                    unsupported_version: "不支持的主题格式版本".into(),
                    save_failure: "保存主题失败".into(),
                    subtab_ui: "UI 外观".into(),
                    subtab_terminal: "终端".into(),
                    subtab_ansi: "ANSI 调色板".into(),
                },
                shortcuts: SettingsShortcutsTexts {
                    heading: "快捷键".into(),
                    hint: "点击快捷键名称后按下新按键即可修改".into(),
                    not_set: "未设置".into(),
                    reset_defaults: "恢复默认按键".into(),
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
                    revert: "还原".into(),
                    applied: "已应用".into(),
                    builtin: "内置".into(),
                    user_group: "自定义主题".into(),
                    builtin_group: "内置主题".into(),
                    user: "用户".into(),
                    behavior_section: "选择".into(),
                    data_section: "指令记录".into(),
                    maintenance_section: "维护".into(),
                    font_section: "字体".into(),
                    color_section: "颜色".into(),
                    preview_section: "预览".into(),
                    template_section: "配色模板".into(),
                    base_color_section: "基础颜色".into(),
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
                toggle_workspace_sidebar: "显示/隐藏工作区栏".into(),
                zoom_in: "整体放大".into(),
                zoom_out: "整体缩小".into(),
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
                terminal_title: "关闭终端".into(),
                terminal_message: "确定要关闭此终端吗？历史指令将被删除。".into(),
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
                title: "此工作区已锁定".into(),
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
            theme: ThemeTexts {
                light: "浅色".into(),
                dark: "深色".into(),
            },
            about: AboutTexts {
                menu_label: "关于".into(),
                title: "关于 OpenNex".into(),
                version_label: "版本".into(),
                description: "一款面向 AI 应用场景与命令行重度使用者的多窗口堆叠式终端管理器。\n\n支持自由排布终端窗口布局、标签堆叠管理大量会话，布局可保存与加载；内置无限会话命令记忆、全局界面缩放、自定义主题美化、工作区加密保护，自定义快捷键等能力，一站式管控复杂的终端运行环境。\n\n基于 Rust 构建，高性能引擎可稳定支撑6000+ 活动窗口并行运行，原生跨平台支持 Linux、Windows、macOS，并提供 20+ 国际化语言。\n如在使用中遇到 Bug 或有功能优化建议，欢迎反馈。".into(),
                homepage_label: "主页".into(),
                source_label: "开源".into(),
                license_label: "开源协议".into(),
                credits_label: "致谢".into(),
                credits: "基于 egui、egui_dock、alacritty_terminal、egui_term 等开源项目构建。".into(),
                close: "关闭".into(),
            },
            update: UpdateTexts {
                checking: "正在检查更新...".into(),
                downloading: "正在下载更新...".into(),
                verifying: "正在校验文件完整性...".into(),
                ready: "更新已准备就绪".into(),
                failed: "更新失败: {}".into(),
                available: "发现新版本 v{} — 准备下载".into(),
                up_to_date: "当前已是最新版本".into(),
                check: "检查更新".into(),
                update_now: "立即更新".into(),
                restart: "重启应用".into(),
                restart_title: "更新已准备就绪".into(),
                restart_body: "新版本已下载，是否立即重启应用？".into(),
                restart_confirm: "重启".into(),
            },
            theme_editor: ThemeEditorTexts {
                colors: ColorNameTexts {
                    app_bg: "主背景".into(),
                    sidebar: "侧栏".into(),
                    panel: "面板".into(),
                    input_bg: "输入框".into(),
                    text: "文字".into(),
                    weak_text: "弱化文字".into(),
                    accent: "强调".into(),
                    warning: "警告".into(),
                    danger: "危险".into(),
                    hover: "悬停".into(),
                    active: "激活".into(),
                    selection_bg: "选中背景".into(),
                    selection_text: "选中文字".into(),
                    border: "边框".into(),
                    lock: "锁定".into(),
                    window_shadow: "阴影".into(),
                    tab_highlight: "焦点标签".into(),
                    fg: "前景".into(),
                    bg: "背景".into(),
                    cursor: "光标".into(),
                    selection_term_bg: "选区背景".into(),
                    selection_term_text: "选区文字".into(),
                    link: "链接".into(),
                },
                edit_title: "编辑主题".into(),
                name_label: "名称:".into(),
                confirm: "保存".into(),
                cancel: "取消".into(),
                system_ui: "System UI".into(),
                terminal: "Terminal".into(),
                ui_font_label: "UI 字体: ".into(),
                ui_font_size: "UI 字号: ".into(),
                terminal_font_label: "终端字体: ".into(),
                terminal_font_size: "终端字号: ".into(),
                cell_spacing: "间距: ".into(),
                terminal_padding: "终端内边距: ".into(),
            },
            stats: StatsTexts {
                copied_toast: "已复制到剪切板".into(),
                focused: "当前终端".into(),
                workspace: "当前工作区".into(),
                global: "全局".into(),
                terminals: "终端".into(),
                clear_history_title: "删除所有指令记录".into(),
                clear_history_body: "确认删除所有终端的指令记录？此操作不可恢复。".into(),
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

fn embedded_locale(code: &str) -> Option<&'static str> {
    match code {
        "zh" => Some(include_str!("../locales/zh.yaml")),
        "en" => Some(include_str!("../locales/en.yaml")),
        "zh-TW" => Some(include_str!("../locales/zh-TW.yaml")),
        "de" => Some(include_str!("../locales/de.yaml")),
        "fr" => Some(include_str!("../locales/fr.yaml")),
        "ja" => Some(include_str!("../locales/ja.yaml")),
        "it" => Some(include_str!("../locales/it.yaml")),
        "ko" => Some(include_str!("../locales/ko.yaml")),
        "hi" => Some(include_str!("../locales/hi.yaml")),
        _ => None,
    }
}

fn known_locale_codes() -> Vec<&'static str> {
    vec!["zh", "en", "zh-TW", "de", "fr", "ja", "it", "ko", "hi"]
}

pub fn scan_available_languages() -> Vec<(String, String)> {
    let dir = locales_dir();
    let mut languages = Vec::new();
    let mut seen_codes = std::collections::HashSet::new();

    // 1. Try filesystem locales first
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                continue;
            }
            if let Some(code) = path.file_stem().and_then(|s| s.to_str()) {
                if seen_codes.insert(code.to_string()) {
                    if let Some(texts) = Texts::load_from_yaml(&path) {
                        languages.push((code.to_string(), texts.display_name));
                    }
                }
            }
        }
    }

    // 2. Add embedded locales not already found
    for code in known_locale_codes() {
        if seen_codes.insert(code.to_string()) {
            if let Some(yaml_str) = embedded_locale(code) {
                if let Ok(texts) = serde_yaml::from_str::<Texts>(yaml_str) {
                    languages.push((code.to_string(), texts.display_name));
                }
            }
        }
    }

    if languages.is_empty() {
        vec![("zh".to_string(), "中文".to_string())]
    } else {
        languages
    }
}

pub fn load_language(code: &str) -> Texts {
    // 1. Try filesystem
    let path = locales_dir().join(format!("{}.yaml", code));
    if path.exists() {
        if let Some(texts) = Texts::load_from_yaml(&path) {
            return texts;
        }
    }
    // 2. Try embedded
    if let Some(yaml_str) = embedded_locale(code) {
        if let Ok(texts) = serde_yaml::from_str::<Texts>(yaml_str) {
            return texts;
        }
    }
    // 3. Fallback to default
    Texts::zh_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zh_default_contains_all_fields() {
        let texts = Texts::zh_default();
        assert!(!texts.display_name.is_empty());
        assert!(!texts.menu.workspace.is_empty());
        assert!(!texts.workspace.heading.is_empty());
        assert!(!texts.terminal.rename.is_empty());
    }

    #[test]
    fn yaml_roundtrip_preserves_display_name() {
        let texts = Texts::zh_default();
        let yaml = serde_yaml::to_string(&texts).unwrap();
        let parsed: Texts = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.display_name, texts.display_name);
        assert_eq!(parsed.menu.workspace, texts.menu.workspace);
        assert_eq!(parsed.workspace.heading, texts.workspace.heading);
    }

    #[test]
    fn every_embedded_locale_parses_into_texts() {
        for code in known_locale_codes() {
            let y = embedded_locale(code).unwrap();
            assert!(
                serde_yaml::from_str::<Texts>(y).is_ok(),
                "embedded locale {code} failed to parse"
            );
        }
    }

    #[test]
    fn load_language_falls_back_to_default_for_missing_code() {
        let texts = load_language("__nonexistent__");
        assert_eq!(texts.display_name, "中文");
    }
}

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
    pub ssh: SshTexts,
    pub ai: AiTexts,
    pub monitor: MonitorTexts,
    pub remote: RemoteTexts,
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
    /// 视图 menu toggle for the floating AI assistant panel.
    #[serde(default)]
    pub ai_assistant: String,
    /// 视图 menu toggle for the floating monitor panel.
    #[serde(default)]
    pub monitor: String,
    /// 视图 menu toggle for the remote phone control panel.
    #[serde(default)]
    pub remote: String,
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
    pub smooth_rendering: String,
    pub smooth_level: String,
    /// PROD warning banner switch (SSH hosts marked as production).
    #[serde(default)]
    pub prod_banner: String,
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
    pub history_favorite: String,
    pub history_delete: String,
    pub next_terminal: String,
    pub next_panel: String,
    pub next_workspace: String,
    pub save_scene: String,
    pub terminal_interrupt: String,
    pub terminal_copy: String,
    pub terminal_paste: String,
    pub terminal_cut: String,
    pub toggle_workspace_sidebar: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub shortcuts_heading: String,
    /// Stop the running terminal agent (Ctrl+Shift+.).
    #[serde(default)]
    pub stop_agent: String,
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
    pub lock_btn: String,
    pub unlock_btn: String,
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
    pub need_setup: String,
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
    pub history_empty: String,
    pub fav_assemble: String,
    pub fav_new_folder: String,
    pub fav_rename_title: String,
    pub fav_new_title: String,
    pub fav_name_label: String,
    pub fav_delete_title: String,
    pub fav_delete_body: String,
    pub fav_expand_hint: String,
    pub fav_btn_assemble: String,
    pub fav_btn_add_cmd: String,
    pub fav_btn_rename: String,
    pub fav_btn_delete: String,
    pub fav_cmd_dialog_title: String,
    pub fav_cmd_dialog_label: String,
    pub fav_clear_folder: String,
    pub fav_menu_assemble: String,
    pub rename: String,
    pub clear_history: String,
    pub new_tab: String,
    pub add_tab: String,
    pub rename_hint: String,
    pub default_name_prefix: String,
    /// History-menu row action: add to global favorites.
    pub favorite: String,
    /// History-menu / favorites row action: delete the entry.
    pub delete: String,
    /// Footer action: clear the GLOBAL favorite commands.
    pub clear_favorites: String,
    /// Footer count, e.g. "42 history commands" (number appended by code).
    pub history_count: String,
    /// Folder-column footer count, e.g. "3 folders" (number prepended).
    pub fav_folder_count: String,
    /// Submenu-column footer count, e.g. "5 commands" (number prepended).
    pub fav_item_count: String,
    /// Confirmation dialog for clearing global favorites.
    pub clear_favorites_title: String,
    pub clear_favorites_body: String,
    /// Tab context-menu: join the broadcast-input group.
    #[serde(default)]
    pub broadcast_join: String,
    /// Tab context-menu: leave the broadcast-input group.
    #[serde(default)]
    pub broadcast_leave: String,
    /// Snippet fill-in dialog: title / hint ({} tokens shown per field).
    #[serde(default)]
    pub snippet_fill_title: String,
    #[serde(default)]
    pub snippet_fill_hint: String,
    /// Tab context-menu + dialog for the per-terminal startup command.
    #[serde(default)]
    pub startup_cmd: String,
    #[serde(default)]
    pub startup_cmd_title: String,
    #[serde(default)]
    pub startup_cmd_hint: String,
    /// Floating scrollback search bar hint text.
    #[serde(default)]
    pub term_search_hint: String,
}

/// Remote phone control (QR + embedded server) strings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteTexts {
    pub panel_title: String,
    pub start: String,
    pub stop: String,
    pub copy_url: String,
    pub copied: String,
    pub off_hint: String,
    pub on_hint: String,
    pub security_hint: String,
    pub bind_failed: String,
    pub settings_section: String,
    pub port_label: String,
    /// Address target selector labels.
    #[serde(default)]
    pub addr_lan: String,
    #[serde(default)]
    pub addr_ipv6: String,
    #[serde(default)]
    pub addr_tunnel: String,
    /// Quick tunnel controls / states.
    #[serde(default)]
    pub tunnel_start: String,
    #[serde(default)]
    pub tunnel_starting: String,
    #[serde(default)]
    pub tunnel_ready: String,
    #[serde(default)]
    pub tunnel_failed: String,
    #[serde(default)]
    pub tunnel_warning: String,
    #[serde(default)]
    pub no_addr: String,
    /// Relay-channel editor (frp profiles in settings) + panel hints.
    #[serde(default)]
    pub relay_section: String,
    #[serde(default)]
    pub relay_add: String,
    #[serde(default)]
    pub relay_name: String,
    #[serde(default)]
    pub relay_server: String,
    #[serde(default)]
    pub relay_port: String,
    #[serde(default)]
    pub relay_forward: String,
    #[serde(default)]
    pub relay_token: String,
    #[serde(default)]
    pub relay_enabled: String,
    #[serde(default)]
    pub relay_delete: String,
    #[serde(default)]
    pub relay_retry: String,
    #[serde(default)]
    pub relay_hint: String,
}

/// Floating monitor panel strings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorTexts {
    pub title: String,
    pub global: String,
    pub cpu: String,
    pub memory: String,
    pub empty_hint: String,
}

/// Floating AI assistant panel + its settings section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiTexts {
    pub panel_title: String,
    pub settings_section: String,
    pub enable: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub prompt_hint: String,
    pub send: String,
    pub explain_selection: String,
    pub insert_to_terminal: String,
    pub generating: String,
    pub empty_hint: String,
    pub error_label: String,
    pub not_enabled: String,
    pub clear: String,
    /// Terminal right-click AI menu (selection / screen actions).
    #[serde(default)]
    pub ctx_explain: String,
    #[serde(default)]
    pub ctx_fix: String,
    #[serde(default)]
    pub ctx_translate: String,
    #[serde(default)]
    pub ctx_explain_screen: String,
    /// Insert the answer into the terminal AND run it.
    #[serde(default)]
    pub insert_run: String,
    /// PROD execution guard dialog.
    #[serde(default)]
    pub exec_confirm_title: String,
    /// "{}" twice: host address, then the command preview.
    #[serde(default)]
    pub exec_confirm_body: String,
    #[serde(default)]
    pub save_snippet: String,
    #[serde(default)]
    pub saved_toast: String,
    /// Terminal agent section.
    #[serde(default)]
    pub agent_section: String,
    #[serde(default)]
    pub agent_goal_hint: String,
    #[serde(default)]
    pub agent_start: String,
    #[serde(default)]
    pub agent_stop: String,
    #[serde(default)]
    pub agent_close: String,
    #[serde(default)]
    pub agent_continue: String,
    #[serde(default)]
    pub agent_approval: String,
    #[serde(default)]
    pub agent_max_steps: String,
    #[serde(default)]
    pub agent_manual: String,
    #[serde(default)]
    pub agent_allowlist: String,
    #[serde(default)]
    pub agent_fullauto: String,
    #[serde(default)]
    pub agent_phase_thinking: String,
    #[serde(default)]
    pub agent_phase_waiting: String,
    #[serde(default)]
    pub agent_phase_executing: String,
    #[serde(default)]
    pub agent_phase_timed_out: String,
    #[serde(default)]
    pub agent_phase_need_input: String,
    #[serde(default)]
    pub agent_phase_done: String,
    #[serde(default)]
    pub agent_phase_failed: String,
    #[serde(default)]
    pub agent_confirm_title: String,
    #[serde(default)]
    pub agent_confirm_body: String,
    #[serde(default)]
    pub agent_confirm_run: String,
    #[serde(default)]
    pub agent_confirm_cancel: String,
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
    pub badge: String,
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
    /// "Confirm" for generic confirmation dialogs (distinct from the theme
    /// editor's save button, which reuses `confirm`).
    pub dialog_confirm: String,
    pub cancel: String,
    pub system_ui: String,
    pub terminal: String,
    pub ui_font_label: String,
    pub ui_font_size: String,
    pub terminal_font_label: String,
    pub terminal_font_size: String,
    pub cell_spacing: String,
    pub terminal_padding: String,
    pub accent_label: String,
    pub active_label: String,
    pub app_bg_label: String,
    pub apply_template: String,
    pub base_colors: String,
    pub bg_label: String,
    pub black: String,
    pub blue: String,
    pub border_label: String,
    pub bright: String,
    pub builtin_readonly: String,
    pub cell_spacing_label: String,
    pub color_selection_term_bg: String,
    pub color_selection_term_text: String,
    pub color_tab_highlight: String,
    pub color_window_shadow: String,
    pub copy: String,
    pub copy_dialog_hint: String,
    pub copy_dialog_title: String,
    pub current: String,
    pub cursor_label: String,
    pub cyan: String,
    pub danger_label: String,
    pub delete: String,
    pub delete_confirm: String,
    pub dim: String,
    pub discard: String,
    pub discard_and_switch: String,
    pub export: String,
    pub fg_label: String,
    pub green: String,
    pub heading: String,
    pub hover_label: String,
    pub import: String,
    pub input_bg_label: String,
    pub interaction_colors: String,
    pub keep: String,
    pub link_label: String,
    pub lock_label: String,
    pub magenta: String,
    pub new: String,
    pub new_dialog_hint: String,
    pub new_dialog_title: String,
    pub normal: String,
    pub palette_template_label: String,
    pub panel_label: String,
    pub red: String,
    pub rename: String,
    pub rename_dialog_title: String,
    pub save_and_switch: String,
    pub selection_bg_label: String,
    pub selection_text_label: String,
    pub sidebar_label: String,
    pub status_colors: String,
    pub switch_confirm: String,
    pub terminal_appearance: String,
    pub terminal_base_colors: String,
    pub terminal_font_label_short: String,
    pub terminal_font_size_label: String,
    pub terminal_padding_label: String,
    pub text_colors: String,
    pub text_label: String,
    pub ui_appearance: String,
    pub ui_font_label_short: String,
    pub ui_font_size_label: String,
    pub unsaved: String,
    pub warning_label: String,
    pub weak_text_label: String,
    pub white: String,
    pub yellow: String,
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

/// Sidebar SSH host book + connection dialogs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SshTexts {
    pub section: String,
    pub add: String,
    pub search_hint: String,
    pub empty_hint: String,
    pub connect: String,
    pub edit: String,
    pub duplicate: String,
    pub delete: String,
    pub delete_title: String,
    /// "{}" is replaced with the host name.
    pub delete_body: String,
    pub dialog_new_title: String,
    pub dialog_edit_title: String,
    pub label_name: String,
    pub label_group: String,
    pub label_host: String,
    pub label_port: String,
    pub label_user: String,
    pub label_auth: String,
    pub label_key_path: String,
    pub label_prod: String,
    pub auth_agent: String,
    pub auth_key: String,
    pub auth_password: String,
    pub error_required: String,
    pub browse: String,
    pub prod_banner: String,
    pub close_remote_message: String,
    pub ssh_unavailable: String,
    /// "{}" is replaced with the host name.
    pub host_missing_fallback: String,
    pub menu_entry: String,
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
                ai_assistant: "AI 助手".into(),
                monitor: "监控面板".into(),
                remote: "远程控制".into(),
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
                smooth_rendering: "边缘平滑".into(),
                smooth_level: "平滑等级".into(),
                prod_banner: "生产环境警示横幅 (PROD)".into(),
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
                history_favorite: "收藏指令".into(),
                history_delete: "删除指令".into(),
                next_terminal: "切换下一个终端".into(),
                next_panel: "切换下一个分屏".into(),
                next_workspace: "切换下一个工作区".into(),
                save_scene: "保存当前布局".into(),
                terminal_interrupt: "终端中断 (^C)".into(),
                terminal_copy: "终端复制".into(),
                terminal_paste: "终端粘贴".into(),
                terminal_cut: "终端剪切".into(),
                toggle_workspace_sidebar: "显示/隐藏工作区栏".into(),
                zoom_in: "整体放大".into(),
                zoom_out: "整体缩小".into(),
                shortcuts_heading: "快捷键".into(),
                stop_agent: "停止 AI 代理".into(),
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
                lock_btn: "锁定".into(),
                unlock_btn: "解锁".into(),
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
                need_setup: "请先设置密码后再锁定工作区".into(),
            },
            lock_overlay: LockOverlayTexts {
                title: "此工作区已锁定".into(),
                password_label: "密码:".into(),
                unlock_button: "解锁".into(),
                wrong_password: "密码错误".into(),
            },
            terminal: TerminalTexts {
                not_found: "未找到终端".into(),
                history_empty: "暂无指令".into(),
                fav_assemble: "组装指令".into(),
                fav_new_folder: "新建收藏夹".into(),
                fav_rename_title: "重命名收藏夹".into(),
                fav_new_title: "新建收藏夹".into(),
                fav_name_label: "名称:".into(),
                fav_delete_title: "删除收藏夹".into(),
                fav_delete_body: "将删除该收藏夹及其所有指令，此操作不可恢复。".into(),
                fav_expand_hint: "按 → 展开 / ← 收起".into(),
                fav_btn_assemble: "组合指令".into(),
                fav_btn_add_cmd: "新建指令".into(),
                fav_btn_rename: "修改".into(),
                fav_btn_delete: "删除".into(),
                fav_cmd_dialog_title: "新建指令".into(),
                fav_cmd_dialog_label: "指令:".into(),
                fav_clear_folder: "清空收藏夹".into(),
                fav_menu_assemble: "合并指令".into(),
                rename: "重命名".into(),
                clear_history: "清空指令历史".into(),
                new_tab: "+ 新建标签页".into(),
                add_tab: "+ 标签页".into(),
                rename_hint: "输入名称...".into(),
                default_name_prefix: "终端 ".into(),
                favorite: "收藏".into(),
                delete: "删除".into(),
                clear_favorites: "清除收藏".into(),
                history_count: "条历史指令".into(),
                fav_folder_count: "个指令收藏夹".into(),
                fav_item_count: "条收藏指令".into(),
                clear_favorites_title: "清除全部收藏指令".into(),
                clear_favorites_body: "确认清除全部收藏指令？此操作不可恢复。".into(),
                broadcast_join: "加入广播输入".into(),
                broadcast_leave: "退出广播输入".into(),
                snippet_fill_title: "填写片段参数".into(),
                snippet_fill_hint: "此片段包含 {参数} 占位符，填写后确认插入（不会自动执行）。".into(),
                startup_cmd: "启动命令...".into(),
                startup_cmd_title: "启动命令".into(),
                startup_cmd_hint: "每次创建或恢复此终端时自动执行；留空并确认即清除。".into(),
                term_search_hint: "搜索...".into(),
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
                badge: "新版本 v{}".into(),
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
                dialog_confirm: "确认".into(),
                cancel: "取消".into(),
                system_ui: "System UI".into(),
                terminal: "Terminal".into(),
                ui_font_label: "UI 字体: ".into(),
                ui_font_size: "UI 字号: ".into(),
                terminal_font_label: "终端字体: ".into(),
                terminal_font_size: "终端字号: ".into(),
                cell_spacing: "间距: ".into(),
                terminal_padding: "终端内边距: ".into(),
                accent_label: "强调:".into(),
                active_label: "激活:".into(),
                app_bg_label: "主背景:".into(),
                apply_template: "应用模板".into(),
                base_colors: "基础颜色".into(),
                bg_label: "背景色:".into(),
                black: "黑".into(),
                blue: "蓝".into(),
                border_label: "边框:".into(),
                bright: "明亮".into(),
                builtin_readonly: "内置主题只读，编辑将创建副本".into(),
                cell_spacing_label: "单元格间距:".into(),
                color_selection_term_bg: "选中背景:".into(),
                color_selection_term_text: "选中文字:".into(),
                color_tab_highlight: "标签高亮:".into(),
                color_window_shadow: "窗口阴影:".into(),
                copy: "复制".into(),
                copy_dialog_hint: "输入新主题名称:".into(),
                copy_dialog_title: "创建主题副本".into(),
                current: "当前主题:".into(),
                cursor_label: "光标:".into(),
                cyan: "青".into(),
                danger_label: "危险:".into(),
                delete: "删除".into(),
                delete_confirm: "确认删除此主题？".into(),
                dim: "暗淡".into(),
                discard: "放弃修改".into(),
                discard_and_switch: "放弃并切换".into(),
                export: "导出".into(),
                fg_label: "前景色:".into(),
                green: "绿".into(),
                heading: "主题".into(),
                hover_label: "悬停:".into(),
                import: "导入".into(),
                input_bg_label: "输入框:".into(),
                interaction_colors: "交互颜色".into(),
                keep: "保留".into(),
                link_label: "链接:".into(),
                lock_label: "锁定遮罩:".into(),
                magenta: "紫".into(),
                new: "新建".into(),
                new_dialog_hint: "输入新主题名称:".into(),
                new_dialog_title: "新建主题".into(),
                normal: "普通".into(),
                palette_template_label: "配色模板:".into(),
                panel_label: "面板:".into(),
                red: "红".into(),
                rename: "重命名".into(),
                rename_dialog_title: "重命名主题".into(),
                save_and_switch: "保存并切换".into(),
                selection_bg_label: "选中背景:".into(),
                selection_text_label: "选中文字:".into(),
                sidebar_label: "侧栏:".into(),
                status_colors: "状态颜色".into(),
                switch_confirm: "当前主题有未保存的修改".into(),
                terminal_appearance: "终端外观".into(),
                terminal_base_colors: "基础颜色".into(),
                terminal_font_label_short: "终端字体:".into(),
                terminal_font_size_label: "终端字号:".into(),
                terminal_padding_label: "终端内边距:".into(),
                text_colors: "文字颜色".into(),
                text_label: "普通文字:".into(),
                ui_appearance: "UI 外观".into(),
                ui_font_label_short: "UI 字体:".into(),
                ui_font_size_label: "UI 字号:".into(),
                unsaved: "● 未保存".into(),
                warning_label: "警告:".into(),
                weak_text_label: "弱化文字:".into(),
                white: "白".into(),
                yellow: "黄".into(),
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
            ssh: SshTexts {
                section: "SSH 主机".into(),
                add: "+ 添加主机".into(),
                search_hint: "搜索主机...".into(),
                empty_hint: "暂无主机，点击 + 添加".into(),
                connect: "连接".into(),
                edit: "编辑".into(),
                duplicate: "创建副本".into(),
                delete: "删除".into(),
                delete_title: "删除 SSH 主机".into(),
                delete_body: "确认删除主机「{}」？已打开的连接不受影响。".into(),
                dialog_new_title: "添加 SSH 主机".into(),
                dialog_edit_title: "编辑 SSH 主机".into(),
                label_name: "名称:".into(),
                label_group: "分组:".into(),
                label_host: "主机:".into(),
                label_port: "端口:".into(),
                label_user: "用户名:".into(),
                label_auth: "认证方式:".into(),
                label_key_path: "密钥路径:".into(),
                label_prod: "生产环境 (PROD)".into(),
                auth_agent: "ssh-agent".into(),
                auth_key: "密钥文件".into(),
                auth_password: "交互输入密码".into(),
                error_required: "名称和主机不能为空".into(),
                browse: "浏览...".into(),
                prod_banner: "PROD".into(),
                close_remote_message: "确定要关闭此远程连接吗？".into(),
                ssh_unavailable: "未找到 ssh 程序，请先安装 OpenSSH 客户端".into(),
                host_missing_fallback: "主机「{}」已删除，终端已回退为本地 shell".into(),
                menu_entry: "SSH 连接...".into(),
            },
            ai: AiTexts {
                panel_title: "AI 助手".into(),
                settings_section: "AI 助手".into(),
                enable: "启用 AI 助手（OpenAI 兼容接口 / 本地 Ollama）".into(),
                base_url: "API 地址:".into(),
                api_key: "API Key:".into(),
                model: "模型:".into(),
                prompt_hint: "输入问题或描述任务...".into(),
                send: "发送".into(),
                explain_selection: "解释终端输出".into(),
                insert_to_terminal: "插入到终端".into(),
                generating: "正在思考...".into(),
                empty_hint: "回答会显示在这里。可先选中终端输出再点击「解释终端输出」。".into(),
                error_label: "请求失败:".into(),
                not_enabled: "请先在 设置 → 通用 中启用 AI 助手并填写 API 地址与 Key。".into(),
                clear: "清空".into(),
                ctx_explain: "AI 解释选中".into(),
                ctx_fix: "AI 修复".into(),
                ctx_translate: "AI 翻译".into(),
                ctx_explain_screen: "AI 解释屏幕".into(),
                insert_run: "插入并执行".into(),
                exec_confirm_title: "生产环境执行确认".into(),
                exec_confirm_body: "即将在 PROD 主机 {} 上执行：\n{}".into(),
                save_snippet: "存为片段".into(),
                saved_toast: "已存入片段库".into(),
                agent_section: "AI 终端代理".into(),
                agent_goal_hint: "输入目标，例如：找出占用 8080 端口的进程并给出修复建议…".into(),
                agent_start: "启动代理".into(),
                agent_stop: "停止".into(),
                agent_close: "关闭".into(),
                agent_continue: "继续".into(),
                agent_approval: "审批模式:".into(),
                agent_max_steps: "最大步数:".into(),
                agent_manual: "每步确认".into(),
                agent_allowlist: "白名单自动".into(),
                agent_fullauto: "全自动".into(),
                agent_phase_thinking: "思考中…".into(),
                agent_phase_waiting: "等待审批…".into(),
                agent_phase_executing: "命令执行中…".into(),
                agent_phase_timed_out: "执行超时，等待手动处理".into(),
                agent_phase_need_input: "需要你的输入".into(),
                agent_phase_done: "目标已达成".into(),
                agent_phase_failed: "已停止".into(),
                agent_confirm_title: "代理命令审批".into(),
                agent_confirm_body: "AI 代理请求执行：\n{}".into(),
                agent_confirm_run: "执行".into(),
                agent_confirm_cancel: "拒绝".into(),
            },
            remote: RemoteTexts {
                panel_title: "手机远程控制".into(),
                start: "启动远程控制".into(),
                stop: "停止".into(),
                copy_url: "复制链接".into(),
                copied: "已复制链接".into(),
                off_hint: "开启后生成二维码，手机微信扫码即可查看并操作全部工作区终端（局域网内有效）。".into(),
                on_hint: "局域网内可用，微信扫码访问：".into(),
                security_hint: "注意：同一局域网内的设备均可访问此地址；关闭后二维码立即失效。Windows 首次开启可能弹出防火墙提示。".into(),
                bind_failed: "端口绑定失败".into(),
                settings_section: "手机远程控制".into(),
                port_label: "端口:".into(),
                addr_lan: "局域网".into(),
                addr_ipv6: "IPv6 直连".into(),
                addr_tunnel: "外网隧道".into(),
                tunnel_start: "开启外网隧道".into(),
                tunnel_starting: "正在准备外网隧道…".into(),
                tunnel_ready: "外网隧道已就绪".into(),
                tunnel_failed: "外网隧道失败".into(),
                tunnel_warning: "外网地址经 Cloudflare 中转（TLS 加密）；关闭远程控制即失效。".into(),
                no_addr: "未检测到可用地址".into(),
                relay_section: "中转通道（frp）".into(),
                relay_add: "添加中转通道".into(),
                relay_name: "名称".into(),
                relay_server: "服务器地址".into(),
                relay_port: "服务端口".into(),
                relay_forward: "公网端口".into(),
                relay_token: "令牌".into(),
                relay_enabled: "启用".into(),
                relay_delete: "删除".into(),
                relay_retry: "重试".into(),
                relay_hint: "选中上方通道即自动连接；首次使用将自动下载 frpc（约 10MB），也可手动放置到数据目录 tunnel/frpc。".into(),
            },
            monitor: MonitorTexts {
                title: "监控面板".into(),
                global: "全局".into(),
                cpu: "CPU".into(),
                memory: "内存".into(),
                empty_hint: "暂无采样数据，等待下一个采样周期…".into(),
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

/// Walk a serialized Texts JSON value and collect every leaf path
/// ("settings.general.max_history" style) - the structural counterpart
/// of `leaf_paths` for the Rust side.
#[cfg(test)]
fn collect_texts_fields(value: serde_json::Value, prefix: &str, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let path = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                collect_texts_fields(v, &path, out);
            }
        }
        _ => out.push(prefix.to_string()),
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

    /// Collect every leaf-key path ("settings.general.language") of a
    /// YAML document so locales can be compared structurally.
    fn leaf_paths(value: &serde_yaml::Value, prefix: &str, out: &mut Vec<String>) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                for (k, v) in map {
                    let key = k.as_str().unwrap_or_default().to_string();
                    let path = if prefix.is_empty() {
                        key
                    } else {
                        format!("{prefix}.{key}")
                    };
                    leaf_paths(v, &path, out);
                }
            }
            _ => out.push(prefix.to_string()),
        }
    }

    /// serde(default) fields silently degrade to empty strings when a
    /// locale file misses a key — this test makes any drift LOUD by
    /// requiring all embedded locales to expose exactly the same leaf
    /// keys as the English reference.
    #[test]
    fn all_locales_have_identical_key_sets_as_english() {
        let reference = serde_yaml::from_str::<serde_yaml::Value>(embedded_locale("en").unwrap())
            .expect("en.yaml parses");
        let mut expected = Vec::new();
        leaf_paths(&reference, "", &mut expected);
        assert!(
            expected.len() > 100,
            "reference extraction broke ({} keys)",
            expected.len()
        );
        expected.sort();

        for code in known_locale_codes() {
            if code == "en" {
                continue;
            }
            let doc: serde_yaml::Value =
                serde_yaml::from_str(embedded_locale(code).unwrap()).unwrap();
            let mut actual = Vec::new();
            leaf_paths(&doc, "", &mut actual);
            actual.sort();
            let missing: Vec<&String> = expected.iter().filter(|k| !actual.contains(k)).collect();
            let extra: Vec<&String> = actual.iter().filter(|k| !expected.contains(k)).collect();
            assert!(
                missing.is_empty() && extra.is_empty(),
                "locale {code} drift — missing: {missing:?} extra: {extra:?}"
            );
        }
    }

    /// The yaml-vs-yaml key test cannot catch keys that are missing from
    /// EVERY locale (they silently deserialize to empty strings via
    /// serde defaults - the v2.5 tunnel labels shipped blank this way).
    /// This guard compares the ENGLISH yaml against the Rust struct
    /// itself: every Texts leaf field must exist in en.yaml.
    #[test]
    fn english_yaml_covers_every_texts_field() {
        let expected = {
            let texts = Texts::zh_default();
            let mut fields = Vec::new();
            collect_texts_fields(serde_json::to_value(&texts).unwrap(), "", &mut fields);
            fields.sort();
            fields
        };
        assert!(expected.len() > 400, "field extraction broke");
        let doc: serde_yaml::Value = serde_yaml::from_str(embedded_locale("en").unwrap()).unwrap();
        let mut actual = Vec::new();
        leaf_paths(&doc, "", &mut actual);
        actual.sort();
        let missing: Vec<&String> = expected.iter().filter(|k| !actual.contains(k)).collect();
        assert!(
            missing.is_empty(),
            "en.yaml is missing {} Texts field(s): {missing:?}",
            missing.len()
        );
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

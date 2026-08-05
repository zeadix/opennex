use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::terminal::TerminalInstance;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 1.0;
const WORKSPACE_SIDEBAR_DEFAULT_WIDTH: f32 = 192.0;
const WORKSPACE_DRAG_HANDLE_WIDTH: f32 = 20.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    #[serde(default = "default_max_history")]
    max_history: usize,
    #[serde(default = "default_scrollback")]
    scrollback: usize,
    #[serde(default)]
    font_size: f32,
    #[serde(default = "default_font_family")]
    font_family: String,
    #[serde(default = "default_cell_spacing")]
    cell_spacing: f32,
    #[serde(default = "default_bg")]
    bg_color: [u8; 3],
    #[serde(default = "default_fg")]
    fg_color: [u8; 3],
    #[serde(default = "default_menu_bg")]
    menu_bg_color: [u8; 3],
    #[serde(default = "default_menu_fg")]
    menu_fg_color: [u8; 3],
    #[serde(default = "default_menu_font_size")]
    menu_font_size: f32,
    #[serde(default)]
    lock_password: String,
    #[serde(default = "default_lock_color")]
    lock_color: [u8; 3],
    #[serde(default)]
    settings_window: SettingsWindowState,
    #[serde(default = "default_key_binds")]
    key_binds: HashMap<String, ShortcutBinding>,
    #[serde(default = "default_language")]
    language: String,
}

fn default_language() -> String {
    "zh".into()
}

fn default_font_family() -> String {
    "monospace".into()
}
fn default_cell_spacing() -> f32 {
    1.0
}

fn default_max_history() -> usize {
    300
}
fn default_scrollback() -> usize {
    10000
}
fn default_bg() -> [u8; 3] {
    [0, 0, 0]
}
fn default_fg() -> [u8; 3] {
    [255, 255, 255]
}
fn default_menu_bg() -> [u8; 3] {
    [30, 30, 30]
}
fn default_menu_fg() -> [u8; 3] {
    [255, 255, 255]
}
fn default_menu_font_size() -> f32 {
    14.0
}
fn default_lock_color() -> [u8; 3] {
    [30, 30, 60]
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShortcutBinding {
    key: String,
    ctrl: bool,
    shift: bool,
    alt: bool,
}

fn default_key_binds() -> HashMap<String, ShortcutBinding> {
    let mut m = HashMap::new();
    m.insert(
        "new_terminal".into(),
        ShortcutBinding {
            key: "N".into(),
            ctrl: true,
            shift: true,
            alt: false,
        },
    );
    m.insert(
        "close_terminal".into(),
        ShortcutBinding {
            key: "W".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "workspace_up".into(),
        ShortcutBinding {
            key: "ArrowUp".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "workspace_down".into(),
        ShortcutBinding {
            key: "ArrowDown".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "panel_left".into(),
        ShortcutBinding {
            key: "ArrowLeft".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "panel_right".into(),
        ShortcutBinding {
            key: "ArrowRight".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "lock_workspace".into(),
        ShortcutBinding {
            key: "L".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "history_menu".into(),
        ShortcutBinding {
            key: "Alt".into(),
            ctrl: false,
            shift: false,
            alt: true,
        },
    );
    m.insert(
        "history_prev".into(),
        ShortcutBinding {
            key: "ArrowUp".into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "history_next".into(),
        ShortcutBinding {
            key: "ArrowDown".into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
    );
    m
}

fn binding_to_modifiers(b: &ShortcutBinding) -> egui::Modifiers {
    let mut m = egui::Modifiers::NONE;
    if b.ctrl {
        m |= egui::Modifiers::CTRL;
    }
    if b.shift {
        m |= egui::Modifiers::SHIFT;
    }
    if b.alt {
        m |= egui::Modifiers::ALT;
    }
    m
}

fn binding_to_key(b: &ShortcutBinding) -> Option<egui::Key> {
    if b.key == "Alt" {
        return None;
    }
    match b.key.as_str() {
        "N" => Some(egui::Key::N),
        "W" => Some(egui::Key::W),
        "L" => Some(egui::Key::L),
        "ArrowUp" => Some(egui::Key::ArrowUp),
        "ArrowDown" => Some(egui::Key::ArrowDown),
        "ArrowLeft" => Some(egui::Key::ArrowLeft),
        "ArrowRight" => Some(egui::Key::ArrowRight),
        "PageUp" => Some(egui::Key::PageUp),
        "PageDown" => Some(egui::Key::PageDown),
        "Tab" => Some(egui::Key::Tab),
        "Escape" => Some(egui::Key::Escape),
        "Enter" => Some(egui::Key::Enter),
        "Space" => Some(egui::Key::Space),
        _ => None,
    }
}

fn key_display_name(k: &str) -> &str {
    match k {
        "ArrowUp" => "↑",
        "ArrowDown" => "↓",
        "ArrowLeft" => "←",
        "ArrowRight" => "→",
        "PageUp" => "PageUp",
        "PageDown" => "PageDown",
        "N" => "N",
        "W" => "W",
        "Tab" => "Tab",
        "Escape" => "Esc",
        "Enter" => "Enter",
        "Space" => "Space",
        _ => k,
    }
}

fn shortcut_display(b: &ShortcutBinding) -> String {
    if b.key == "Alt" {
        return "Alt".into();
    }
    let mut s = String::new();
    if b.ctrl {
        s.push_str("Ctrl+");
    }
    if b.shift {
        s.push_str("Shift+");
    }
    if b.alt {
        s.push_str("Alt+");
    }
    s.push_str(key_display_name(&b.key));
    s
}

fn shortcut_hint_ids() -> [&'static str; 10] {
    [
        "new_terminal",
        "close_terminal",
        "workspace_up",
        "workspace_down",
        "panel_left",
        "panel_right",
        "lock_workspace",
        "history_menu",
        "history_prev",
        "history_next",
    ]
}

fn shortcut_label_for<'a>(texts: &'a crate::i18n::Texts, id: &str) -> &'a str {
    match id {
        "new_terminal" => &texts.shortcut_labels.new_terminal,
        "close_terminal" => &texts.shortcut_labels.close_terminal,
        "workspace_up" => &texts.shortcut_labels.workspace_up,
        "workspace_down" => &texts.shortcut_labels.workspace_down,
        "panel_left" => &texts.shortcut_labels.panel_left,
        "panel_right" => &texts.shortcut_labels.panel_right,
        "lock_workspace" => &texts.shortcut_labels.lock_workspace,
        "history_menu" => &texts.shortcut_labels.history_menu,
        "history_prev" => &texts.shortcut_labels.history_prev,
        "history_next" => &texts.shortcut_labels.history_next,
        _ => "",
    }
}

fn shortcut_hint_available_height(available_height: f32) -> f32 {
    available_height.max(0.0)
}

const WORKSPACE_ACTION_BUTTON_SIZE: egui::Vec2 = egui::vec2(24.0, 24.0);

fn workspace_lock_icon(is_locked: bool) -> &'static str {
    if is_locked {
        egui_phosphor::regular::LOCK
    } else {
        egui_phosphor::regular::LOCK_OPEN
    }
}

fn panel_order_after_move(src: usize, dst: usize, len: usize) -> Vec<usize> {
    if src >= len || dst >= len {
        return (0..len).collect();
    }
    let mut order: Vec<_> = (0..len).collect();
    let panel = order.remove(src);
    order.insert(dst, panel);
    order
}

fn remap_panel_index(index: usize, old_to_new: &[usize]) -> Option<usize> {
    old_to_new.get(index).copied()
}

fn apply_panel_rename(panel: &mut Panel, value: &str) {
    if !value.is_empty() {
        panel.name = value.to_string();
    }
}

fn terminal_should_have_focus(
    terminal_is_active: bool,
    workspace_is_renaming: bool,
    terminal_is_renaming: bool,
) -> bool {
    terminal_is_active && !workspace_is_renaming && !terminal_is_renaming
}

fn terminal_focus_lock_allowed(workspace_is_renaming: bool, terminal_is_renaming: bool) -> bool {
    !workspace_is_renaming && !terminal_is_renaming
}

fn drag_row_is_source(index: usize, source: Option<usize>) -> bool {
    source == Some(index)
}

fn drag_row_is_target(index: usize, target: Option<usize>) -> bool {
    target == Some(index)
}

fn drag_insertion_y(rect: egui::Rect, pointer_y: f32) -> f32 {
    if pointer_y <= rect.center().y {
        rect.top()
    } else {
        rect.bottom()
    }
}

fn drag_drop_destination(src: usize, target: usize, after_target: bool, len: usize) -> usize {
    if src >= len || target >= len || src == target {
        return target;
    }

    let target_after_removal = if src < target { target - 1 } else { target };
    (target_after_removal + usize::from(after_target)).min(len - 1)
}

fn terminal_tab_is_closeable(terminal_count: usize) -> bool {
    terminal_count > 1
}

fn cancel_workspace_rename(renaming_panel: &mut Option<usize>, escape_pressed: bool) -> bool {
    if escape_pressed && renaming_panel.is_some() {
        *renaming_panel = None;
        true
    } else {
        false
    }
}

fn check_shortcut(
    ctx: &egui::Context,
    binds: &HashMap<String, ShortcutBinding>,
    name: &str,
) -> bool {
    if let Some(b) = binds.get(name) {
        let mods = binding_to_modifiers(b);
        if let Some(key) = binding_to_key(b) {
            return ctx.input_mut(|i| i.consume_key(mods, key));
        }
    }
    false
}

fn history_menu_shortcut_released(
    ctx: &egui::Context,
    binds: &HashMap<String, ShortcutBinding>,
    state: &mut AltKeyState,
) -> bool {
    let Some(binding) = binds.get("history_menu") else {
        return false;
    };
    if binding.key != "Alt" || binding.ctrl || binding.shift || !binding.alt {
        return false;
    }

    let other_key_pressed = ctx.input(|input| {
        input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.alt
            )
        })
    });
    ctx.input(|input| update_alt_key_state(state, input.modifiers.alt, other_key_pressed))
}

fn update_alt_key_state(state: &mut AltKeyState, alt_down: bool, other_key_pressed: bool) -> bool {
    let mut released = false;
    if alt_down && !state.pressed {
        state.pressed = true;
        state.used_with_other_key = false;
    }
    if other_key_pressed {
        state.used_with_other_key = true;
    }
    if !alt_down && state.pressed {
        released = !state.used_with_other_key;
        state.pressed = false;
        state.used_with_other_key = false;
    }
    released
}

fn toggle_history_menu(nav: &mut Option<HistoryNav>, entries: Vec<String>) {
    if nav.is_some() {
        *nav = None;
    } else if !entries.is_empty() {
        *nav = Some(HistoryNav {
            entries,
            selected: 0,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsWindowState {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Default for SettingsWindowState {
    fn default() -> Self {
        SettingsWindowState {
            x: 200.0,
            y: 150.0,
            width: 500.0,
            height: 350.0,
        }
    }
}

fn settings_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join("settings.json")
}

fn load_settings() -> AppSettings {
    let path = settings_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return normalize_history_bindings(settings);
            }
        }
    }
    AppSettings::default()
}

fn normalize_history_bindings(mut settings: AppSettings) -> AppSettings {
    let defaults = default_key_binds();
    if !settings.key_binds.contains_key("history_menu") {
        settings.key_binds.insert(
            "history_menu".into(),
            defaults.get("history_menu").unwrap().clone(),
        );
    }
    if !settings.key_binds.contains_key("history_prev") {
        let binding = settings
            .key_binds
            .remove("history_up")
            .or_else(|| defaults.get("history_prev").cloned())
            .unwrap();
        settings.key_binds.insert("history_prev".into(), binding);
    }
    if !settings.key_binds.contains_key("history_next") {
        let binding = settings
            .key_binds
            .remove("history_down")
            .or_else(|| defaults.get("history_next").cloned())
            .unwrap();
        settings.key_binds.insert("history_next".into(), binding);
    }
    settings.key_binds.remove("history_up");
    settings.key_binds.remove("history_down");
    settings
}

fn save_settings(settings: &AppSettings) -> Result<(), anyhow::Error> {
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(settings_path(), content)?;
    Ok(())
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            max_history: default_max_history(),
            scrollback: default_scrollback(),
            font_size: 14.0,
            font_family: default_font_family(),
            cell_spacing: default_cell_spacing(),
            bg_color: default_bg(),
            fg_color: default_fg(),
            menu_bg_color: default_menu_bg(),
            menu_fg_color: default_menu_fg(),
            menu_font_size: default_menu_font_size(),
            lock_password: String::new(),
            lock_color: default_lock_color(),
            settings_window: SettingsWindowState::default(),
            key_binds: default_key_binds(),
            language: default_language(),
        }
    }
}

fn scan_system_fonts() -> Vec<(String, String)> {
    let mut result = Vec::new();
    let font_dirs = [
        "/usr/share/fonts",
        "/usr/local/share/fonts",
        "/home/kunpengwang/.local/share/fonts",
        "/home/kunpengwang/.fonts",
    ];
    let mut seen = std::collections::HashSet::new();
    for dir in &font_dirs {
        let path = std::path::Path::new(dir);
        if !path.exists() {
            continue;
        }
        for entry in walk_font_dir(path) {
            if !seen.contains(&entry.0) {
                seen.insert(entry.0.clone());
                result.push(entry);
            }
        }
    }
    result
}

fn walk_font_dir(dir: &std::path::Path) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(walk_font_dir(&path));
            } else if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext == "ttf" || ext == "otf" || ext == "ttc" {
                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".into());
                    result.push((name, path.to_string_lossy().to_string()));
                }
            }
        }
    }
    result
}

fn scene_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join("scene.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceState {
    panel_name: String,
    dock_state: DockState<String>,
    terminals: HashMap<String, TerminalStatePersist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TerminalStatePersist {
    name: String,
    font_size: f32,
    working_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneState {
    panels: Vec<ScenePanel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenePanel {
    name: String,
    dock_state: DockState<String>,
    terminals: HashMap<String, TerminalStatePersist>,
}

struct Panel {
    name: String,
    bound_file: Option<PathBuf>,
}

pub struct HistoryNav {
    pub entries: Vec<String>,
    pub selected: usize,
}

impl HistoryNav {
    fn move_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_next(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }
}

#[derive(Default)]
struct AltKeyState {
    pressed: bool,
    used_with_other_key: bool,
}

pub struct App {
    panels: Vec<Panel>,
    active_panel: usize,
    dock_states: HashMap<usize, DockState<String>>,
    terminals: HashMap<String, TerminalData>,
    tab_counter: u32,
    terminal_id_counter: u64,
    pending_new_terminal: Option<(usize, SurfaceIndex, NodeIndex)>,
    pending_close: Option<String>,
    pending_split_after: Option<String>,
    pending_split_vertical: bool,
    renaming_panel: Option<usize>,
    rename_buffer: String,
    renaming_terminal: Option<String>,
    terminal_rename_buffer: String,
    rename_frame_count: u32,
    pending_load_workspace: Option<PathBuf>,
    pending_load_from_template: Option<PathBuf>,
    pending_delete_template: Option<PathBuf>,
    pending_load_scene: bool,
    pending_save_scene_as: bool,
    pending_clear_history: bool,
    settings: AppSettings,
    show_settings: bool,
    settings_edit: AppSettings,
    settings_tab: usize,
    binding_recording: Option<String>,
    cached_template_files: Vec<(String, PathBuf)>,
    completion: crate::completion::CompletionEngine,
    history_db: crate::history_db::HistoryDb,
    focused_terminal: Option<String>,
    drag_src_panel: Option<usize>,
    drag_dst_panel: Option<usize>,
    locked_panels: std::collections::HashSet<usize>,
    lock_password_input: String,
    pw_old: String,
    pw_new1: String,
    pw_new2: String,
    pw_set1: String,
    pw_set2: String,
    pw_clear: String,
    panel_rects: Vec<egui::Rect>,
    pw_message: String,
    pw_popup: Option<&'static str>,
    close_confirm_panel: Option<usize>,
    system_fonts: Vec<String>,
    unlock_popup: Option<usize>,
    cwd_poll_frame: u8,
    alt_key: AltKeyState,
    terminal_focus_id: Option<egui::Id>,
    texts: crate::i18n::Texts,
    available_languages: Vec<(String, String)>,
}

struct TerminalData {
    instance: TerminalInstance,
    name: String,
    font_size: f32,
}

fn create_terminal(
    ctx: &egui::Context,
    working_dir: &str,
    id_counter: &mut u64,
) -> Option<TerminalInstance> {
    #[cfg(unix)]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    #[cfg(not(unix))]
    let shell = "cmd.exe".to_string();

    let cwd_str = if std::path::PathBuf::from(working_dir).exists() {
        working_dir.to_string()
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    *id_counter += 1;
    let id = *id_counter;

    TerminalInstance::create(ctx, id, &shell, &cwd_str, 80, 24)
}

fn build_panel_state(app: &mut App, panel_idx: usize) -> Option<WorkspaceState> {
    let panel = app.panels.get(panel_idx)?;
    let dock_state = app.dock_states.get(&panel_idx)?.clone();
    let mut terminals = HashMap::new();
    for (id, data) in &mut app.terminals {
        data.instance.poll_cwd();
        terminals.insert(
            id.clone(),
            TerminalStatePersist {
                name: data.name.clone(),
                font_size: data.font_size,
                working_directory: data.instance.cwd.clone(),
            },
        );
    }
    Some(WorkspaceState {
        panel_name: panel.name.clone(),
        dock_state,
        terminals,
    })
}

fn save_to_file<T: Serialize>(path: &PathBuf, data: &T) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn load_scene_file(path: &PathBuf) -> Option<SceneState> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn build_scene_state(app: &mut App) -> SceneState {
    let mut panels = Vec::new();
    for (panel_idx, panel) in app.panels.iter().enumerate() {
        let dock_state = app
            .dock_states
            .get(&panel_idx)
            .cloned()
            .unwrap_or_else(|| DockState::new(vec![]));
        let mut terminals = HashMap::new();
        for (id, data) in &mut app.terminals {
            data.instance.poll_cwd();
            terminals.insert(
                id.clone(),
                TerminalStatePersist {
                    name: data.name.clone(),
                    font_size: data.font_size,
                    working_directory: data.instance.cwd.clone(),
                },
            );
        }
        panels.push(ScenePanel {
            name: panel.name.clone(),
            dock_state,
            terminals,
        });
    }
    SceneState { panels }
}

fn save_scene(path: &PathBuf, app: &mut App) {
    let state = build_scene_state(app);
    if let Err(e) = save_to_file(path, &state) {
        log::error!("Failed to save scene: {}", e);
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let settings = load_settings();
        let ctx = &cc.egui_ctx.clone();

        // Scan system monospace fonts
        let system_fonts = scan_system_fonts();
        // Register all found fonts in egui
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        let mut registered_names: Vec<String> = Vec::new();
        for (name, path) in &system_fonts {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(
                    name.clone(),
                    std::sync::Arc::new(egui::FontData::from_owned(data)),
                );
                registered_names.push(name.clone());
            }
        }
        // Also try CJK font
        if let Ok(cjk_data) = std::fs::read("/usr/share/fonts/opentype/noto/NotoSansCJK-Medium.ttc")
        {
            fonts.font_data.insert(
                "noto-cjk".into(),
                std::sync::Arc::new(egui::FontData::from_owned(cjk_data).tweak(egui::FontTweak {
                    scale: 0.9,
                    ..Default::default()
                })),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "noto-cjk".into());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("noto-cjk".into());
        }
        // Register found fonts into Monospace family
        if let Some(mono_family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            for name in &registered_names {
                mono_family.push(name.clone());
            }
        }
        ctx.set_fonts(fonts);

        let font_names: Vec<String> = registered_names;
        let db_path = std::env::current_dir()
            .unwrap_or_default()
            .join("history.db");
        let language = settings.language.clone();
        let available_languages = crate::i18n::scan_available_languages();
        let mut app = App {
            panels: Vec::new(),
            active_panel: 0,
            dock_states: HashMap::new(),
            terminals: HashMap::new(),
            tab_counter: 0,
            terminal_id_counter: 0,
            pending_new_terminal: None,
            pending_close: None,
            pending_split_after: None,
            pending_split_vertical: false,
            renaming_panel: None,
            rename_buffer: String::new(),
            renaming_terminal: None,
            terminal_rename_buffer: String::new(),
            rename_frame_count: 0,
            pending_load_workspace: None,
            pending_load_from_template: None,
            pending_delete_template: None,
            pending_load_scene: false,
            pending_save_scene_as: false,
            pending_clear_history: false,
            settings,
            show_settings: false,
            settings_edit: AppSettings::default(),
            settings_tab: 0,
            binding_recording: None,
            cached_template_files: Vec::new(),
            completion: crate::completion::CompletionEngine::new(),
            history_db: crate::history_db::HistoryDb::new(&db_path, default_max_history()),
            focused_terminal: None,
            drag_src_panel: None,
            drag_dst_panel: None,
            locked_panels: std::collections::HashSet::new(),
            lock_password_input: String::new(),
            pw_old: String::new(),
            pw_new1: String::new(),
            pw_new2: String::new(),
            pw_set1: String::new(),
            pw_set2: String::new(),
            pw_clear: String::new(),
            panel_rects: Vec::new(),
            pw_message: String::new(),
            pw_popup: None,
            close_confirm_panel: None,
            system_fonts: font_names,
            unlock_popup: None,
            cwd_poll_frame: 0,
            alt_key: AltKeyState::default(),
            terminal_focus_id: None,
            texts: crate::i18n::load_language(&language),
            available_languages,
        };

        let scene_path = scene_path();
        if scene_path.exists() {
            if let Some(scene) = load_scene_file(&scene_path) {
                app.settings_edit = app.settings.clone();
                for panel in &scene.panels {
                    let idx = app.panels.len();
                    for (_id, tstate) in &panel.terminals {
                        let Some(instance) = create_terminal(
                            ctx,
                            &tstate.working_directory,
                            &mut app.terminal_id_counter,
                        ) else {
                            continue;
                        };
                        app.terminals.insert(
                            _id.clone(),
                            TerminalData {
                                instance,
                                name: tstate.name.clone(),
                                font_size: tstate.font_size,
                            },
                        );
                        if let Some(n) = _id
                            .strip_prefix("terminal-")
                            .and_then(|s| s.parse::<u32>().ok())
                        {
                            if n > app.tab_counter {
                                app.tab_counter = n;
                            }
                        }
                    }
                    app.panels.push(Panel {
                        name: panel.name.clone(),
                        bound_file: None,
                    });
                    app.dock_states.insert(idx, panel.dock_state.clone());
                }
                app.active_panel = 0;
                app.refresh_template_files();
                return app;
            }
        }

        app.add_initial_terminal(ctx);
        app.refresh_template_files();
        app
    }

    fn templates_dir(&self) -> PathBuf {
        std::env::current_dir()
            .unwrap_or_default()
            .join("templates")
    }

    fn refresh_template_files(&mut self) {
        let dir = self.templates_dir();
        let _ = std::fs::create_dir_all(&dir);
        self.cached_template_files = std::fs::read_dir(&dir)
            .into_iter()
            .flat_map(|rd| rd.into_iter())
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .map(|e| e.path())
            .filter_map(|path| {
                let stem = path.file_stem()?.to_string_lossy().to_string();
                Some((stem, path))
            })
            .collect();
    }

    fn save_as_template(&mut self, panel_idx: usize) {
        let Some(state) = build_panel_state(self, panel_idx) else {
            return;
        };
        let dir = self.templates_dir();
        let _ = std::fs::create_dir_all(&dir);
        let name = state.panel_name.replace(['/', '\\', ':'], "_");
        let path = dir.join(format!("{}.json", name));
        if let Err(e) = save_to_file(&path, &state) {
            log::error!("Failed to save template: {}", e);
        }
        self.refresh_template_files();
    }

    fn save_workspace(&mut self, path: PathBuf) {
        if self.panels.is_empty() {
            return;
        }
        let Some(state) = build_panel_state(self, self.active_panel) else {
            return;
        };
        if let Err(e) = save_to_file(&path, &state) {
            log::error!("Failed to save workspace: {}", e);
        }
    }

    fn load_workspace_file(&mut self, ctx: &egui::Context, path: PathBuf) {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read workspace: {}", e);
                return;
            }
        };
        let state: WorkspaceState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to parse workspace: {}", e);
                return;
            }
        };
        self.load_workspace_state(ctx, state, Some(path));
    }

    fn add_initial_terminal(&mut self, ctx: &egui::Context) {
        let name = format!("{}1", self.texts.workspace.name_prefix);
        let Some(tab_id) = self.create_terminal_inner(ctx) else {
            return;
        };
        self.dock_states.insert(0, DockState::new(vec![tab_id]));
        self.panels.push(Panel {
            name,
            bound_file: None,
        });
        self.active_panel = 0;
    }

    fn load_workspace_state(
        &mut self,
        ctx: &egui::Context,
        state: WorkspaceState,
        file: Option<PathBuf>,
    ) {
        let panel_idx = self.panels.len();
        for (id, tstate) in &state.terminals {
            if !self.terminals.contains_key(id) {
                let Some(instance) = create_terminal(
                    ctx,
                    &tstate.working_directory,
                    &mut self.terminal_id_counter,
                ) else {
                    continue;
                };
                self.terminals.insert(
                    id.clone(),
                    TerminalData {
                        instance,
                        name: tstate.name.clone(),
                        font_size: tstate.font_size,
                    },
                );
            }
        }
        self.panels.push(Panel {
            name: state.panel_name,
            bound_file: file,
        });
        self.dock_states.insert(panel_idx, state.dock_state);
    }

    fn is_renaming(&self) -> bool {
        self.renaming_panel.is_some() || self.renaming_terminal.is_some()
    }

    fn create_terminal_inner(&mut self, ctx: &egui::Context) -> Option<String> {
        self.tab_counter += 1;
        let id = format!("terminal-{}", self.tab_counter);
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let instance = create_terminal(ctx, &cwd, &mut self.terminal_id_counter)?;
        let random_suffix: String = uuid::Uuid::new_v4().as_bytes()[0..3]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        self.terminals.insert(
            id.clone(),
            TerminalData {
                instance,
                name: format!(
                    "{}{}",
                    self.texts.terminal.default_name_prefix, random_suffix
                ),
                font_size: DEFAULT_FONT_SIZE,
            },
        );
        if self.focused_terminal.is_none() {
            self.focused_terminal = Some(id.clone());
        }
        Some(id)
    }

    fn process_pending(&mut self, ctx: &egui::Context) {
        if let Some((panel_idx, surface_idx, node_idx)) = self.pending_new_terminal.take() {
            let _split_after = self.pending_split_after.take();
            let Some(tab_id) = self.create_terminal_inner(ctx) else {
                return;
            };
            if let Some(tree) = self.dock_states.get_mut(&panel_idx) {
                tree.set_focused_node_and_surface((surface_idx, node_idx));
                tree.push_to_focused_leaf(tab_id);
            }
        }
        if let Some(tab) = self.pending_close.take() {
            // Count terminals in the active workspace's dock tree
            let terminal_count = if let Some(tree) = self.dock_states.get(&self.active_panel) {
                let mut count = 0;
                for t in self.terminals.keys() {
                    if tree.find_tab(t).is_some() {
                        count += 1;
                    }
                }
                count
            } else {
                0
            };
            if terminal_count <= 1 {
                // Don't close the last terminal
            } else {
                self.terminals.remove(&tab);
                if self.focused_terminal.as_ref() == Some(&tab) {
                    self.focused_terminal = None;
                }
                for tree in self.dock_states.values_mut() {
                    if let Some(loc) = tree.find_tab(&tab) {
                        tree.remove_tab(loc);
                    }
                }
                self.panels.retain(|p| p.name != tab);
            }
        }
        if self.pending_load_scene {
            self.pending_load_scene = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Scene", &["json"])
                .set_file_name("scene.json")
                .pick_file()
            {
                if let Some(scene) = load_scene_file(&path) {
                    self.terminals.clear();
                    self.panels.clear();
                    self.dock_states.clear();
                    self.completion = crate::completion::CompletionEngine::new();
                    for panel in &scene.panels {
                        let idx = self.panels.len();
                        for (_id, tstate) in &panel.terminals {
                            if let Some(instance) = create_terminal(
                                ctx,
                                &tstate.working_directory,
                                &mut self.terminal_id_counter,
                            ) {
                                self.terminals.insert(
                                    _id.clone(),
                                    TerminalData {
                                        instance,
                                        name: tstate.name.clone(),
                                        font_size: tstate.font_size,
                                    },
                                );
                            }
                        }
                        self.panels.push(Panel {
                            name: panel.name.clone(),
                            bound_file: None,
                        });
                        self.dock_states.insert(idx, panel.dock_state.clone());
                    }
                    self.active_panel = 0;
                }
            }
        }
        if self.pending_save_scene_as {
            self.pending_save_scene_as = false;
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Scene", &["json"])
                .set_file_name("scene.json")
                .save_file()
            {
                save_scene(&path, self);
            }
        }
        if let Some(path) = self.pending_load_workspace.take() {
            self.load_workspace_file(ctx, path);
        }
        if let Some(path) = self.pending_load_from_template.take() {
            self.load_workspace_file(ctx, path);
        }
        if let Some(path) = self.pending_delete_template.take() {
            let _ = std::fs::remove_file(&path);
            self.refresh_template_files();
        }
        if self.pending_clear_history {
            self.pending_clear_history = false;
            self.history_db.clear_all();
        }
    }

    fn add_panel(&mut self, ctx: &egui::Context) {
        let n = self.panels.len() + 1;
        let name = format!("{}{}", self.texts.workspace.name_prefix, n);
        let Some(tab_id) = self.create_terminal_inner(ctx) else {
            return;
        };
        self.dock_states
            .insert(self.panels.len(), DockState::new(vec![tab_id]));
        self.panels.push(Panel {
            name,
            bound_file: None,
        });
    }

    fn close_workspace(&mut self, i: usize) {
        if self.panels.len() <= 1 {
            return;
        }
        let panel = self.panels.swap_remove(i);
        let _ = panel;
        self.dock_states.remove(&i);
        if self.active_panel >= self.panels.len() {
            self.active_panel = self.panels.len().saturating_sub(1);
        }
    }

    fn reorder_panel(&mut self, src: usize, dst: usize) {
        if src == dst || src >= self.panels.len() || dst >= self.panels.len() {
            return;
        }
        let old_to_new = {
            let new_order = panel_order_after_move(src, dst, self.panels.len());
            let mut mapping = vec![0; new_order.len()];
            for (new_idx, old_idx) in new_order.into_iter().enumerate() {
                mapping[old_idx] = new_idx;
            }
            mapping
        };

        let panel = self.panels.remove(src);
        self.panels.insert(dst, panel);
        let mut old_states: Vec<(usize, DockState<String>)> = self.dock_states.drain().collect();
        old_states.sort_by_key(|(k, _)| *k);
        let mut new_states = HashMap::new();
        for (old_k, state) in old_states {
            if old_k < old_to_new.len() {
                new_states.insert(old_to_new[old_k], state);
            }
        }
        self.dock_states = new_states;

        if let Some(active_panel) = remap_panel_index(self.active_panel, &old_to_new) {
            self.active_panel = active_panel;
        }
        self.locked_panels = self
            .locked_panels
            .iter()
            .filter_map(|index| remap_panel_index(*index, &old_to_new))
            .collect();
        self.close_confirm_panel = self
            .close_confirm_panel
            .and_then(|index| remap_panel_index(index, &old_to_new));
        self.renaming_panel = self
            .renaming_panel
            .and_then(|index| remap_panel_index(index, &old_to_new));
        self.unlock_popup = self
            .unlock_popup
            .and_then(|index| remap_panel_index(index, &old_to_new));
        self.save_scene();
    }

    fn switch_language(&mut self, code: &str) {
        self.texts = crate::i18n::load_language(code);
        self.settings.language = code.to_string();
        self.settings_edit.language = code.to_string();
        let _ = save_settings(&self.settings);
    }

    fn focus_adjacent_panel(&mut self, direction: i32) {
        use egui_dock::{Node, Surface};
        let Some(tree) = self.dock_states.get_mut(&self.active_panel) else {
            return;
        };
        let current = tree.focused_leaf();
        let main = SurfaceIndex(0);
        let mut leaves = Vec::new();
        if let Some(surface) = tree.get_surface(main) {
            let t = match surface {
                Surface::Main(t) => Some(t),
                Surface::Window(t, _) => Some(t),
                Surface::Empty => None,
            };
            if let Some(t) = t {
                for (i, node) in t.iter().enumerate() {
                    if let Node::Leaf { tabs, .. } = node {
                        if !tabs.is_empty() {
                            leaves.push((main, NodeIndex(i)));
                        }
                    }
                }
            }
        }
        if leaves.len() <= 1 {
            return;
        }
        let pos = current
            .and_then(|c| leaves.iter().position(|l| *l == c))
            .unwrap_or(0);
        let next = ((pos as i32 + direction).rem_euclid(leaves.len() as i32)) as usize;
        tree.set_focused_node_and_surface(leaves[next]);
    }

    fn save_scene(&mut self) {
        let path = scene_path();
        save_scene(&path, self);
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        if let Some(id) = self.terminal_focus_id {
            if terminal_focus_lock_allowed(
                self.renaming_panel.is_some(),
                self.renaming_terminal.is_some(),
            ) {
                ctx.memory_mut(|memory| {
                    memory.set_focus_lock_filter(id, egui_term::terminal_focus_event_filter())
                });
            } else {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
                self.terminal_focus_id = None;
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending(ctx);
        self.cwd_poll_frame = self.cwd_poll_frame.wrapping_add(1);
        if self.cwd_poll_frame >= 15 {
            self.cwd_poll_frame = 0;
            for data in self.terminals.values_mut() {
                data.instance.poll_cwd();
            }
        }
        let renaming = self.is_renaming();
        if renaming {
            self.rename_frame_count += 1;
        }

        let workspace_rename_escape = self.renaming_panel.is_some()
            && ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        cancel_workspace_rename(&mut self.renaming_panel, workspace_rename_escape);
        let workspace_renaming = self.renaming_panel.is_some();
        if workspace_renaming {
            if let Some(id) = self.terminal_focus_id.take() {
                ctx.memory_mut(|memory| memory.surrender_focus(id));
            }
        }

        // Keep focused_terminal in sync with dock active tab before handling shortcuts
        if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
            if let Some((_, tab)) = tree.find_active_focused() {
                self.focused_terminal = Some(tab.clone());
            }
        }

        let binds = self.settings.key_binds.clone();
        if self.binding_recording.is_some() || !ctx.input(|input| input.focused) {
            self.alt_key = AltKeyState::default();
        }
        let menu_requested = if workspace_renaming || self.binding_recording.is_some() {
            false
        } else if binds
            .get("history_menu")
            .is_some_and(|binding| binding.key == "Alt")
        {
            history_menu_shortcut_released(ctx, &binds, &mut self.alt_key)
        } else {
            check_shortcut(ctx, &binds, "history_menu")
        };
        if menu_requested && !self.locked_panels.contains(&self.active_panel) {
            if let Some(tab) = self.focused_terminal.clone() {
                let entries = self.history_db.get(&tab, self.settings.max_history);
                if let Some(td) = self.terminals.get_mut(&tab) {
                    toggle_history_menu(&mut td.instance.history_nav, entries);
                }
            }
        }

        let history_menu_active = self
            .focused_terminal
            .as_ref()
            .and_then(|tab| self.terminals.get(tab))
            .is_some_and(|td| td.instance.history_nav.is_some());

        let mut history_menu_handled = false;
        if !workspace_renaming && history_menu_active {
            let close =
                ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
            let confirm =
                ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
            let previous = !close && !confirm && check_shortcut(ctx, &binds, "history_prev");
            let next = !close && !confirm && check_shortcut(ctx, &binds, "history_next");

            if let Some(tab) = self.focused_terminal.clone() {
                history_menu_handled = previous || next || close || confirm;
                if previous || next {
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        if let Some(nav) = td.instance.history_nav.as_mut() {
                            if previous {
                                nav.move_previous();
                            }
                            if next {
                                nav.move_next();
                            }
                        }
                    }
                }
                if close {
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        td.instance.history_nav = None;
                    }
                }
                if confirm {
                    let selected = self.terminals.get_mut(&tab).and_then(|td| {
                        let nav = td.instance.history_nav.take()?;
                        let command = nav.entries.get(nav.selected)?.clone();
                        td.instance.write(command.as_bytes());
                        Some(command)
                    });
                    if let Some(command) = selected {
                        self.history_db.add(&tab, &command);
                    }
                }
            }
        }

        // Consume Tab key to prevent egui focus navigation, send to focused terminal
        if !workspace_renaming
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab))
        {
            if let Some(tab) = &self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    td.instance.write(&[0x09]);
                }
            }
        }

        // Enter: select history entry, or record command + send CR to terminal
        if !workspace_renaming
            && !history_menu_handled
            && !self.locked_panels.contains(&self.active_panel)
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
        {
            if let Some(tab) = &self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    if let Some(ref nav) = td.instance.history_nav {
                        let selected = nav.entries.get(nav.selected).cloned();
                        if let Some(cmd) = selected {
                            td.instance.write(cmd.as_bytes());
                        }
                        td.instance.history_nav = None;
                    } else {
                        let line = td.instance.get_current_line();
                        let prompt_end = line
                            .rfind("$ ")
                            .or_else(|| line.rfind("# "))
                            .map(|p| p + 2)
                            .unwrap_or(0);
                        let cmd = line[prompt_end..].trim().to_string();
                        if !cmd.is_empty() {
                            self.history_db.add(tab, &cmd);
                        }
                        td.instance.write(b"\r");
                    }
                }
            }
        }

        // Escape: close history menu
        if !workspace_renaming
            && !history_menu_handled
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            if let Some(tab) = &self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    if td.instance.history_nav.is_some() {
                        td.instance.history_nav = None;
                    } else {
                        td.instance.write(b"\x1b");
                    }
                }
            }
        }

        // Handle key binding recording in settings
        if let Some(recording) = self.binding_recording.clone() {
            if recording == "history_menu" && ctx.input(|i| i.modifiers.alt) {
                self.settings_edit.key_binds.insert(
                    recording,
                    ShortcutBinding {
                        key: "Alt".into(),
                        ctrl: false,
                        shift: false,
                        alt: true,
                    },
                );
                self.binding_recording = None;
            } else {
                let input = ctx.input(|i| i.clone());
                for event in &input.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if *key != egui::Key::Escape {
                            let key_name = format!("{:?}", key);
                            self.settings_edit.key_binds.insert(
                                recording.clone(),
                                ShortcutBinding {
                                    key: key_name,
                                    ctrl: modifiers.ctrl,
                                    shift: modifiers.shift,
                                    alt: modifiers.alt,
                                },
                            );
                        }
                        self.binding_recording = None;
                        break;
                    }
                }
            }
        }

        // Configurable shortcuts

        if !workspace_renaming && check_shortcut(ctx, &binds, "new_terminal") {
            if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                if let Some((surface, node)) = tree.focused_leaf() {
                    self.pending_new_terminal = Some((self.active_panel, surface, node));
                }
            }
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "close_terminal") {
            if let Some(tab) = &self.focused_terminal.clone() {
                self.pending_close = Some(tab.clone());
            }
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "workspace_up") {
            if self.active_panel > 0 {
                self.active_panel -= 1;
            }
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "workspace_down") {
            if self.active_panel + 1 < self.panels.len() {
                self.active_panel += 1;
            }
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "panel_left") {
            self.focus_adjacent_panel(-1);
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "panel_right") {
            self.focus_adjacent_panel(1);
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "lock_workspace") {
            if self.locked_panels.contains(&self.active_panel) {
                // Already locked: overlay already shows password input.
                self.lock_password_input.clear();
                self.pw_message.clear();
            } else {
                self.locked_panels.insert(self.active_panel);
                self.lock_password_input.clear();
                self.pw_message.clear();
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(self.texts.menu.file.clone(), |ui| {
                    if ui.button(self.texts.file_menu.save.clone()).clicked() {
                        self.save_scene();
                        ui.close_menu();
                    }
                    if ui.button(self.texts.file_menu.load.clone()).clicked() {
                        self.pending_load_scene = true;
                        ui.close_menu();
                    }
                    if ui.button(self.texts.file_menu.save_as.clone()).clicked() {
                        self.pending_save_scene_as = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(self.texts.file_menu.exit.clone()).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button(self.texts.menu.view.clone(), |ui| {
                    if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                        let active_tab = tree.find_active_focused().map(|(_, t)| t.clone());
                        if let Some(ref tab) = active_tab {
                            if ui
                                .button(self.texts.view_menu.split_right.clone())
                                .clicked()
                            {
                                self.pending_split_after = Some(tab.clone());
                                self.pending_split_vertical = false;
                                if let Some((surface, node, _)) = tree.find_tab(tab) {
                                    self.pending_new_terminal =
                                        Some((self.active_panel, surface, node));
                                }
                                ui.close_menu();
                            }
                            if ui.button(self.texts.view_menu.split_down.clone()).clicked() {
                                self.pending_split_after = Some(tab.clone());
                                self.pending_split_vertical = true;
                                if let Some((surface, node, _)) = tree.find_tab(tab) {
                                    self.pending_new_terminal =
                                        Some((self.active_panel, surface, node));
                                }
                                ui.close_menu();
                            }
                        }
                    }
                });
                ui.menu_button(self.texts.menu.language.clone(), |ui| {
                    let current_code = self.settings.language.clone();
                    let languages = self.available_languages.clone();
                    for (code, display_name) in &languages {
                        let label = if *code == current_code {
                            format!("✓ {}", display_name)
                        } else {
                            display_name.clone()
                        };
                        if ui.button(&label).clicked() {
                            self.switch_language(code);
                            ui.close_menu();
                        }
                    }
                });
                if ui.button(self.texts.view_menu.settings.clone()).clicked() {
                    self.show_settings = true;
                    self.settings_edit = self.settings.clone();
                }
            });
        });

        if self.show_settings {
            let mut open = self.show_settings;
            let ws = &self.settings_edit.settings_window;
            egui::Window::new(&self.texts.settings.title)
                .open(&mut open)
                .resizable(true)
                .default_pos([ws.x, ws.y])
                .default_size([ws.width, ws.height])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let tabs = [
                            &self.texts.settings.tabs.general,
                            &self.texts.settings.tabs.appearance,
                            &self.texts.settings.tabs.shortcuts,
                            &self.texts.settings.tabs.lock,
                        ];
                        for (i, label) in tabs.iter().enumerate() {
                            let selected = self.settings_tab == i;
                            if ui.selectable_label(selected, label.as_str()).clicked() {
                                self.settings_tab = i;
                            }
                        }
                    });
                    ui.separator();

                    match self.settings_tab {
                        0 => {
                            ui.label(&self.texts.settings.general.scene_info);
                            ui.label(&self.texts.settings.general.scene_path);
                            ui.label(&self.texts.settings.general.templates_path);
                            ui.separator();
                            ui.label(&self.texts.settings.general.history_section);
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.general.max_history);
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.max_history)
                                        .range(10..=10000),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.general.scrollback);
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.scrollback)
                                        .range(100..=50000),
                                );
                            });
                            if ui
                                .button(&self.texts.settings.general.clear_all_history)
                                .clicked()
                            {
                                self.pending_clear_history = true;
                            }
                        }
                        1 => {
                            ui.label(&self.texts.settings.appearance.terminal_section);
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.font_size);
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.font_size)
                                        .range(8.0..=32.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.cell_spacing);
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.settings_edit.cell_spacing,
                                        0.5..=2.0,
                                    )
                                    .text("x"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.font_family);
                                let current = &self.settings_edit.font_family;
                                egui::ComboBox::from_id_salt("font_family_select")
                                    .selected_text(if current.is_empty() {
                                        "monospace"
                                    } else {
                                        current
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.settings_edit.font_family,
                                            String::new(),
                                            "monospace (默认)",
                                        );
                                        for name in &self.system_fonts {
                                            ui.selectable_value(
                                                &mut self.settings_edit.font_family,
                                                name.clone(),
                                                name,
                                            );
                                        }
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.bg_color);
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.bg_color,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.fg_color);
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.fg_color,
                                );
                            });
                            ui.separator();
                            ui.label(&self.texts.settings.appearance.command_menu_section);
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.menu_bg_color);
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.menu_bg_color,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.menu_fg_color);
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.menu_fg_color,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.appearance.menu_font_size);
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.menu_font_size)
                                        .range(8.0..=32.0),
                                );
                            });
                            ui.separator();
                            ui.label(&self.texts.settings.lock.lock_section);
                            ui.horizontal(|ui| {
                                ui.label(&self.texts.settings.lock.lock_overlay_color);
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.lock_color,
                                );
                            });
                        }
                        2 => {
                            ui.label(&self.texts.settings.shortcuts.hint);
                            ui.separator();
                            for id in shortcut_hint_ids() {
                                let label = shortcut_label_for(&self.texts, id).to_string();
                                ui.horizontal(|ui| {
                                    ui.label(&label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let text =
                                                if self.binding_recording.as_deref() == Some(id) {
                                                    "按下按键...".to_string()
                                                } else if let Some(b) =
                                                    self.settings_edit.key_binds.get(id)
                                                {
                                                    shortcut_display(b)
                                                } else {
                                                    self.texts.settings.shortcuts.not_set.clone()
                                                };
                                            if ui.button(text).clicked() {
                                                self.binding_recording = Some(id.to_string());
                                            }
                                        },
                                    );
                                });
                            }
                        }
                        3 => {
                            ui.label(&self.texts.settings.lock.password_section);
                            ui.separator();
                            if self.settings.lock_password.is_empty() {
                                if ui.button(&self.texts.settings.lock.set_password).clicked() {
                                    self.pw_popup = Some("set");
                                    self.pw_set1.clear();
                                    self.pw_set2.clear();
                                    self.pw_message.clear();
                                }
                            } else {
                                if ui
                                    .button(&self.texts.settings.lock.change_password)
                                    .clicked()
                                {
                                    self.pw_popup = Some("change");
                                    self.pw_old.clear();
                                    self.pw_new1.clear();
                                    self.pw_new2.clear();
                                    self.pw_message.clear();
                                }
                                ui.add_space(10.0);
                                if ui
                                    .button(&self.texts.settings.lock.clear_password)
                                    .clicked()
                                {
                                    self.pw_popup = Some("clear");
                                    self.pw_clear.clear();
                                    self.pw_message.clear();
                                }
                            }
                        }
                        _ => {}
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(&self.texts.settings.buttons.apply).clicked() {
                            self.settings = self.settings_edit.clone();
                            self.history_db.set_max_entries(self.settings.max_history);
                            let _ = save_settings(&self.settings);
                            // Apply font size to all terminals
                            let new_size = self.settings.font_size;
                            for td in self.terminals.values_mut() {
                                td.font_size = new_size;
                            }
                            // Apply font family: rebuild FontDefinitions with selected font first
                            let ctx2 = ctx.clone();
                            let ff = self.settings.font_family.clone();
                            let sys_fonts = self.system_fonts.clone();
                            std::thread::spawn(move || {
                                let mut fonts = egui::FontDefinitions::default();
                                egui_phosphor::add_to_fonts(
                                    &mut fonts,
                                    egui_phosphor::Variant::Regular,
                                );
                                // Register all system fonts
                                for name in &sys_fonts {
                                    let paths = scan_system_fonts();
                                    if let Some((_, path)) = paths.iter().find(|(n, _)| n == name) {
                                        if let Ok(data) = std::fs::read(path) {
                                            fonts.font_data.insert(
                                                name.clone(),
                                                std::sync::Arc::new(egui::FontData::from_owned(
                                                    data,
                                                )),
                                            );
                                        }
                                    }
                                }
                                // CJK font
                                if let Ok(cjk_data) = std::fs::read(
                                    "/usr/share/fonts/opentype/noto/NotoSansCJK-Medium.ttc",
                                ) {
                                    fonts.font_data.insert(
                                        "noto-cjk".into(),
                                        std::sync::Arc::new(
                                            egui::FontData::from_owned(cjk_data).tweak(
                                                egui::FontTweak {
                                                    scale: 0.9,
                                                    ..Default::default()
                                                },
                                            ),
                                        ),
                                    );
                                }
                                // Put selected font first in Monospace
                                if let Some(mono) =
                                    fonts.families.get_mut(&egui::FontFamily::Monospace)
                                {
                                    if !ff.is_empty() && fonts.font_data.contains_key(&ff) {
                                        mono.insert(0, ff.clone());
                                    }
                                    mono.push("noto-cjk".into());
                                }
                                if let Some(prop) =
                                    fonts.families.get_mut(&egui::FontFamily::Proportional)
                                {
                                    prop.insert(0, "noto-cjk".into());
                                }
                                ctx2.set_fonts(fonts);
                            });
                        }
                        if ui.button(&self.texts.workspace.close).clicked() {
                            self.settings_edit = self.settings.clone();
                            self.show_settings = false;
                        }
                    });
                });
            if !open {
                self.show_settings = false;
                self.settings.settings_window = self.settings_edit.settings_window.clone();
                let _ = save_settings(&self.settings);
            }
        }

        // Close workspace confirmation
        if let Some(panel_idx) = self.close_confirm_panel {
            let mut open = true;
            let panel_name = self
                .panels
                .get(panel_idx)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            egui::Window::new(&self.texts.close_confirm.confirm)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "{}{}{}",
                        self.texts.close_confirm.message_prefix,
                        panel_name,
                        self.texts.close_confirm.message_suffix
                    ));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(&self.texts.close_confirm.confirm).clicked() {
                            self.close_workspace(panel_idx);
                            self.close_confirm_panel = None;
                        }
                        if ui.button(&self.texts.close_confirm.cancel).clicked() {
                            self.close_confirm_panel = None;
                        }
                    });
                });
            if !open {
                self.close_confirm_panel = None;
            }
        }

        // Password popup windows
        if let Some(popup) = self.pw_popup {
            let mut open = true;
            let title = match popup {
                "set" => self.texts.password.set_title.as_str(),
                "change" => self.texts.password.change_title.as_str(),
                "clear" => self.texts.password.clear_title.as_str(),
                _ => "",
            };
            egui::Window::new(title)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| match popup {
                    "set" => {
                        ui.horizontal_centered(|ui| {
                            ui.label(&self.texts.password.enter);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_set1)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_set1"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(&self.texts.password.confirm);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_set2)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_set2"),
                            );
                            if !self.pw_set2.is_empty() && self.pw_set1 != self.pw_set2 {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.mismatch)
                                        .color(egui::Color32::RED),
                                );
                            } else if !self.pw_set2.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.r#match)
                                        .color(egui::Color32::GREEN),
                                );
                            }
                        });
                        if !self.pw_message.is_empty() {
                            ui.label(
                                egui::RichText::new(&self.pw_message).color(egui::Color32::RED),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button(&self.texts.password.confirm_button).clicked() {
                                if self.pw_set1.is_empty() {
                                    self.pw_message = self.texts.password.empty_error.clone();
                                } else if self.pw_set1 != self.pw_set2 {
                                    self.pw_message = self.texts.password.mismatch_error.clone();
                                } else {
                                    self.settings_edit.lock_password = self.pw_set1.clone();
                                    self.settings.lock_password = self.pw_set1.clone();
                                    let _ = save_settings(&self.settings);
                                    self.pw_set1.clear();
                                    self.pw_set2.clear();
                                    self.pw_message.clear();
                                    self.pw_popup = None;
                                }
                            }
                            if ui.button(&self.texts.password.cancel_button).clicked() {
                                self.pw_set1.clear();
                                self.pw_set2.clear();
                                self.pw_message.clear();
                                self.pw_popup = None;
                            }
                        });
                    }
                    "change" => {
                        ui.horizontal(|ui| {
                            ui.label(&self.texts.password.original);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_old)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_old"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(&self.texts.password.new);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_new1)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_new1"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(&self.texts.password.confirm_new);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_new2)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_new2"),
                            );
                            if !self.pw_new2.is_empty() && self.pw_new1 != self.pw_new2 {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.mismatch)
                                        .color(egui::Color32::RED),
                                );
                            } else if !self.pw_new2.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.r#match)
                                        .color(egui::Color32::GREEN),
                                );
                            }
                        });
                        if !self.pw_message.is_empty() {
                            ui.label(
                                egui::RichText::new(&self.pw_message).color(egui::Color32::RED),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button(&self.texts.password.confirm_button).clicked() {
                                if self.pw_old != self.settings.lock_password {
                                    self.pw_message = self.texts.password.wrong_error.clone();
                                } else if self.pw_new1.is_empty() {
                                    self.pw_message = self.texts.password.empty_error.clone();
                                } else if self.pw_new1 != self.pw_new2 {
                                    self.pw_message = self.texts.password.mismatch_error.clone();
                                } else {
                                    self.settings_edit.lock_password = self.pw_new1.clone();
                                    self.settings.lock_password = self.pw_new1.clone();
                                    let _ = save_settings(&self.settings);
                                    self.pw_old.clear();
                                    self.pw_new1.clear();
                                    self.pw_new2.clear();
                                    self.pw_message.clear();
                                    self.pw_popup = None;
                                }
                            }
                            if ui.button(&self.texts.password.cancel_button).clicked() {
                                self.pw_old.clear();
                                self.pw_new1.clear();
                                self.pw_new2.clear();
                                self.pw_message.clear();
                                self.pw_popup = None;
                            }
                        });
                    }
                    "clear" => {
                        ui.horizontal(|ui| {
                            ui.label(&self.texts.password.input_label);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_clear)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_clear"),
                            );
                        });
                        if !self.pw_message.is_empty() {
                            ui.label(
                                egui::RichText::new(&self.pw_message).color(egui::Color32::RED),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button(&self.texts.password.confirm_button).clicked() {
                                if self.pw_clear != self.settings.lock_password {
                                    self.pw_message = self.texts.password.wrong_password.clone();
                                } else {
                                    self.settings_edit.lock_password.clear();
                                    self.settings.lock_password.clear();
                                    self.locked_panels.clear();
                                    let _ = save_settings(&self.settings);
                                    self.pw_clear.clear();
                                    self.pw_message.clear();
                                    self.pw_popup = None;
                                }
                            }
                            if ui.button(&self.texts.password.cancel_button).clicked() {
                                self.pw_clear.clear();
                                self.pw_message.clear();
                                self.pw_popup = None;
                            }
                        });
                    }
                    _ => {}
                });
            if !open {
                self.pw_popup = None;
                self.pw_set1.clear();
                self.pw_set2.clear();
                self.pw_old.clear();
                self.pw_new1.clear();
                self.pw_new2.clear();
                self.pw_clear.clear();
                self.pw_message.clear();
            }
        }

        egui::SidePanel::left("navigation")
            .default_width(WORKSPACE_SIDEBAR_DEFAULT_WIDTH)
            .show(ctx, |ui| {
                ui.heading(&self.texts.workspace.heading);
                ui.separator();
                let mut to_select = None;
                let panel_count = self.panels.len();
                let mut reorder = None;
                self.panel_rects.clear();
                self.panel_rects.resize(panel_count, egui::Rect::NOTHING);

                // Detect drag state from pointer
                let pointer_down = ui.input(|i| i.pointer.primary_down());
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                let pointer_released = ui.input(|i| i.pointer.primary_released());

                for i in 0..panel_count {
                    let is_active = i == self.active_panel;
                    if self.renaming_panel == Some(i) {
                        let mut confirm = false;
                        let mut cancel = false;
                        let response = ui
                            .horizontal(|ui| {
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.rename_buffer)
                                        .font(egui::FontId::monospace(14.0))
                                        .desired_width((ui.available_width() - 112.0).max(80.0))
                                        .id_source("workspace_rename"),
                                );
                                response.request_focus();
                                confirm = ui.button(&self.texts.workspace.rename_confirm).clicked();
                                cancel = ui.button(&self.texts.workspace.rename_cancel).clicked();
                                response
                            })
                            .response;
                        self.panel_rects[i] = response.rect;
                        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let escape = ui
                            .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
                        if confirm || enter {
                            apply_panel_rename(&mut self.panels[i], &self.rename_buffer);
                            self.renaming_panel = None;
                        } else if cancel || escape {
                            cancel_workspace_rename(&mut self.renaming_panel, true);
                        }
                    } else {
                        let row = ui.horizontal(|ui| {
                            let panel_name = self.panels[i].name.clone();
                            let handle_size = egui::vec2(
                                WORKSPACE_DRAG_HANDLE_WIDTH,
                                ui.spacing().interact_size.y,
                            );
                            let (handle_rect, handle) =
                                ui.allocate_exact_size(handle_size, egui::Sense::drag());
                            let handle_color = if handle.hovered() {
                                ui.visuals().text_color()
                            } else {
                                ui.visuals().weak_text_color()
                            };
                            ui.painter().text(
                                handle_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                egui_phosphor::regular::DOTS_SIX_VERTICAL,
                                egui::FontId::proportional(14.0),
                                handle_color,
                            );
                            let handle =
                                handle.on_hover_text(&self.texts.workspace.drag_handle_hint);
                            if handle.drag_started() {
                                self.drag_src_panel = Some(i);
                                self.drag_dst_panel = None;
                            }
                            let response = ui.selectable_label(is_active, &panel_name);
                            self.panel_rects[i] = response.rect;

                            // Click to select
                            if response.double_clicked() && !renaming {
                                self.renaming_panel = Some(i);
                                self.rename_buffer = panel_name;
                                self.rename_frame_count = 0;
                                to_select = None;
                            } else if response.clicked() && !renaming {
                                to_select = Some(i);
                            }

                            // Context menu
                            response.context_menu(|ui| {
                                if ui.button(&self.texts.workspace.rename).clicked() {
                                    self.renaming_panel = Some(i);
                                    self.rename_buffer = self.panels[i].name.clone();
                                    self.rename_frame_count = 0;
                                    ui.close_menu();
                                }
                                if ui.button(&self.texts.workspace.save_as_template).clicked() {
                                    self.save_as_template(i);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button(&self.texts.settings.buttons.close).clicked() {
                                    self.close_confirm_panel = Some(i);
                                    ui.close_menu();
                                }
                            });

                            // Lock and close buttons
                            if self.panels.len() > 1 || self.locked_panels.contains(&i) {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if self.panels.len() > 1 {
                                            if ui
                                                .add_sized(
                                                    WORKSPACE_ACTION_BUTTON_SIZE,
                                                    egui::Button::new(egui_phosphor::regular::X),
                                                )
                                                .on_hover_text(&self.texts.workspace.close_ws_hint)
                                                .clicked()
                                            {
                                                self.close_confirm_panel = Some(i);
                                                return;
                                            }
                                        }
                                        let is_locked = self.locked_panels.contains(&i);
                                        if ui
                                            .add_sized(
                                                WORKSPACE_ACTION_BUTTON_SIZE,
                                                egui::Button::new(workspace_lock_icon(is_locked)),
                                            )
                                            .on_hover_text(if is_locked {
                                                &self.texts.workspace.locked_hint
                                            } else {
                                                &self.texts.workspace.unlocked_hint
                                            })
                                            .clicked()
                                        {
                                            if is_locked {
                                                // Switch to the locked panel; the overlay
                                                // has its own password input.
                                                self.active_panel = i;
                                                self.lock_password_input.clear();
                                                self.pw_message.clear();
                                            } else {
                                                self.locked_panels.insert(i);
                                                self.lock_password_input.clear();
                                                self.pw_message.clear();
                                            }
                                        }
                                    },
                                );
                            }
                        });
                        self.panel_rects[i] = row.response.rect;
                    }
                }

                // Handle drag target detection after all rects are known
                if let Some(src) = self.drag_src_panel {
                    if pointer_down || pointer_released {
                        self.drag_dst_panel = None;
                        if let Some(pos) = pointer_pos {
                            for j in (0..panel_count).rev() {
                                if j == src {
                                    continue;
                                }
                                if j < self.panel_rects.len() && self.panel_rects[j].contains(pos) {
                                    self.drag_dst_panel = Some(j);
                                    break;
                                }
                            }
                        }
                    }

                    if pointer_down || pointer_released {
                        ui.ctx().request_repaint();
                        let painter = ui.painter();
                        let source_fill = ui.visuals().faint_bg_color.linear_multiply(0.65);
                        let target_stroke =
                            egui::Stroke::new(1.5, ui.visuals().selection.stroke.color);

                        if drag_row_is_source(src, self.drag_src_panel) {
                            let source_rect = self.panel_rects[src];
                            if source_rect.is_positive() {
                                painter.rect_filled(source_rect, 3.0, source_fill);
                            }
                        }

                        if let Some(dst) = self.drag_dst_panel {
                            if drag_row_is_target(dst, self.drag_dst_panel) {
                                let target_rect = self.panel_rects[dst];
                                if target_rect.is_positive() {
                                    painter.rect_stroke(
                                        target_rect.expand(1.0),
                                        3.0,
                                        target_stroke,
                                        egui::StrokeKind::Outside,
                                    );
                                    if let Some(pos) = pointer_pos {
                                        let insertion_y = drag_insertion_y(target_rect, pos.y);
                                        painter.line_segment(
                                            [
                                                egui::pos2(target_rect.left(), insertion_y),
                                                egui::pos2(target_rect.right(), insertion_y),
                                            ],
                                            egui::Stroke::new(2.0, target_stroke.color),
                                        );
                                    }
                                }
                            }
                        }
                    }

                    if pointer_released {
                        reorder = self.drag_dst_panel.take().map(|target| {
                            let after_target = pointer_pos.is_some_and(|pos| {
                                self.panel_rects
                                    .get(target)
                                    .is_some_and(|rect| pos.y > rect.center().y)
                            });
                            (
                                src,
                                drag_drop_destination(src, target, after_target, panel_count),
                            )
                        });
                        self.drag_src_panel = None;
                    }
                } else if pointer_released {
                    self.drag_src_panel = None;
                    self.drag_dst_panel = None;
                }

                // Perform reorder
                if let Some((src, dst)) = reorder {
                    self.reorder_panel(src, dst);
                }
                if let Some(i) = to_select {
                    self.active_panel = i;
                }
                ui.separator();
                if ui.button(&self.texts.workspace.new).clicked() {
                    self.add_panel(ui.ctx());
                }
                if self.cached_template_files.is_empty() {
                    self.refresh_template_files();
                }
                let template_files = self.cached_template_files.clone();
                ui.menu_button(&self.texts.workspace.templates, |ui| {
                    if template_files.is_empty() {
                        ui.label(&self.texts.workspace.templates_empty);
                    } else {
                        for (display_name, path) in &template_files {
                            let path = path.clone();
                            let display_name = display_name.clone();
                            ui.horizontal(|ui| {
                                if ui.button(display_name.as_str()).clicked() {
                                    self.pending_load_from_template = Some(path.clone());
                                    ui.close_menu();
                                }
                                if ui.small_button("×").clicked() {
                                    self.pending_delete_template = Some(path);
                                    ui.close_menu();
                                }
                            });
                        }
                    }
                });
                ui.separator();
                ui.allocate_ui_with_layout(
                    egui::vec2(
                        ui.available_width(),
                        shortcut_hint_available_height(ui.available_height()),
                    ),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.label(
                            egui::RichText::new(&self.texts.shortcut_labels.shortcuts_heading)
                                .small()
                                .strong()
                                .color(ui.visuals().weak_text_color()),
                        );
                        egui::ScrollArea::vertical()
                            .id_salt("navigation_shortcuts")
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for id in shortcut_hint_ids() {
                                    let label = shortcut_label_for(&self.texts, id).to_string();
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&label)
                                                .small()
                                                .color(ui.visuals().weak_text_color()),
                                        );
                                        let binding = self
                                            .settings
                                            .key_binds
                                            .get(id)
                                            .map(shortcut_display)
                                            .unwrap_or_else(|| {
                                                self.texts.settings.shortcuts.not_set.clone()
                                            });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(binding)
                                                        .small()
                                                        .color(ui.visuals().text_color()),
                                                );
                                            },
                                        );
                                    });
                                }
                            });
                    },
                );
            });

        // The sidebar can enter rename mode during this frame. Read the state again so the
        // terminal view cannot reclaim focus after the rename input requests it.
        let renaming = self.is_renaming();
        let terminal_count = self
            .dock_states
            .get(&self.active_panel)
            .map(|tree| {
                self.terminals
                    .keys()
                    .filter(|tab| tree.find_tab(tab).is_some())
                    .count()
            })
            .unwrap_or(0);
        let active_tab = self
            .dock_states
            .get_mut(&self.active_panel)
            .and_then(|t| t.find_active_focused().map(|(_, t)| t.clone()));
        egui::CentralPanel::default().show(ctx, |ui| {
            let is_locked = self.locked_panels.contains(&self.active_panel);
            if is_locked {
                let lock_color = egui::Color32::from_rgb(
                    self.settings.lock_color[0],
                    self.settings.lock_color[1],
                    self.settings.lock_color[2],
                );
                let avail = ui.available_size();
                let (rect, _) = ui.allocate_exact_size(avail, egui::Sense::click());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, lock_color);

                let form_width = 320.0;
                let heading_height = ui.fonts(|fonts| {
                    fonts
                        .layout_no_wrap(
                            self.texts.lock_overlay.title.clone(),
                            egui::FontId::proportional(24.0),
                            egui::Color32::WHITE,
                        )
                        .size()
                        .y
                });
                let row_height = ui.spacing().interact_size.y;
                let item_spacing = ui.spacing().item_spacing.y;
                let message_height = if self.pw_message.is_empty() {
                    0.0
                } else {
                    ui.text_style_height(&egui::TextStyle::Body)
                };
                let form_height = heading_height
                    + item_spacing
                    + 16.0
                    + row_height
                    + if message_height > 0.0 {
                        item_spacing + 4.0 + message_height
                    } else {
                        0.0
                    };
                let form_top =
                    rect.center().y - heading_height - item_spacing - 16.0 - row_height * 0.5;
                let form_rect = egui::Rect::from_center_size(
                    egui::pos2(rect.center().x, form_top + form_height * 0.5),
                    egui::vec2(form_width, form_height),
                );
                let pw_id = egui::Id::new("lock_overlay_pw_input");
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(form_rect), |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::RichText::new(&self.texts.lock_overlay.title)
                                .size(24.0)
                                .color(egui::Color32::WHITE),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&self.texts.lock_overlay.password_label)
                                    .color(egui::Color32::from_gray(200)),
                            );
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.lock_password_input)
                                    .password(true)
                                    .desired_width(160.0)
                                    .id(pw_id),
                            );
                            resp.request_focus();
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if ui.button(&self.texts.lock_overlay.unlock_button).clicked()
                                || (enter_pressed && resp.has_focus())
                            {
                                if self.settings.lock_password.is_empty()
                                    || self.lock_password_input == self.settings.lock_password
                                {
                                    self.locked_panels.remove(&self.active_panel);
                                    self.lock_password_input.clear();
                                    self.pw_message.clear();
                                } else {
                                    self.pw_message =
                                        self.texts.lock_overlay.wrong_password.clone();
                                    self.lock_password_input.clear();
                                }
                            }
                        });
                        if !self.pw_message.is_empty() {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(&self.pw_message)
                                    .color(egui::Color32::from_rgb(240, 100, 100)),
                            );
                        }
                    });
                });
            } else if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                DockArea::new(tree)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_add_buttons(true)
                    .show_add_popup(false)
                    .show_inside(
                        ui,
                        &mut TerminalTabViewer {
                            terminals: &mut self.terminals,
                            completion: &self.completion,
                            history_db: &self.history_db,
                            max_history: self.settings.max_history,
                            cell_spacing: self.settings.cell_spacing,
                            bg_color: self.settings.bg_color,
                            fg_color: self.settings.fg_color,
                            menu_bg_color: self.settings.menu_bg_color,
                            menu_fg_color: self.settings.menu_fg_color,
                            menu_font_size: self.settings.menu_font_size,
                            pending_close: &mut self.pending_close,
                            pending_new_terminal: &mut self.pending_new_terminal,
                            pending_split_after: &mut self.pending_split_after,
                            pending_split_vertical: &mut self.pending_split_vertical,
                            active_panel: self.active_panel,
                            terminal_count,
                            renaming_terminal: &mut self.renaming_terminal,
                            terminal_rename_buffer: &mut self.terminal_rename_buffer,
                            renaming,
                            rename_frame_count: self.rename_frame_count,
                            active_tab,
                            focused_terminal: &mut self.focused_terminal,
                            terminal_focus_id: &mut self.terminal_focus_id,
                            show_settings: self.show_settings,
                            texts: &self.texts,
                        },
                    );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(&self.texts.workspace.empty_hint);
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_key_binds, toggle_history_menu, update_alt_key_state, AltKeyState, AppSettings,
        HistoryNav, ShortcutBinding, TerminalStatePersist,
    };
    use egui_dock::DockState;

    #[test]
    fn shortcut_hint_labels_cover_all_configurable_actions() {
        let ids: Vec<_> = super::shortcut_hint_ids().iter().copied().collect();

        for id in [
            "new_terminal",
            "close_terminal",
            "workspace_up",
            "workspace_down",
            "panel_left",
            "panel_right",
            "lock_workspace",
            "history_menu",
            "history_prev",
            "history_next",
        ] {
            assert!(ids.contains(&id), "missing shortcut hint for {id}");
        }
    }

    #[test]
    fn shortcut_hint_area_uses_all_remaining_sidebar_height() {
        assert_eq!(super::shortcut_hint_available_height(240.0), 240.0);
        assert_eq!(super::shortcut_hint_available_height(-1.0), 0.0);
    }

    #[test]
    fn workspace_lock_icon_matches_lock_state() {
        assert_eq!(
            super::workspace_lock_icon(true),
            egui_phosphor::regular::LOCK
        );
        assert_eq!(
            super::workspace_lock_icon(false),
            egui_phosphor::regular::LOCK_OPEN
        );
    }

    #[test]
    fn panel_order_move_maps_source_to_destination() {
        assert_eq!(super::panel_order_after_move(0, 2, 4), vec![1, 2, 0, 3]);
        assert_eq!(super::panel_order_after_move(3, 1, 4), vec![0, 3, 1, 2]);
    }

    #[test]
    fn panel_index_remap_keeps_state_attached_to_workspace() {
        let order = super::panel_order_after_move(0, 2, 4);
        let mut old_to_new = vec![0; order.len()];
        for (new_index, old_index) in order.into_iter().enumerate() {
            old_to_new[old_index] = new_index;
        }

        assert_eq!(super::remap_panel_index(0, &old_to_new), Some(2));
        assert_eq!(super::remap_panel_index(2, &old_to_new), Some(1));
        assert_eq!(super::remap_panel_index(4, &old_to_new), None);
    }

    #[test]
    fn panel_rename_accepts_non_empty_value_and_keeps_name_for_empty_value() {
        let mut panel = super::Panel {
            name: "Original".into(),
            bound_file: None,
        };

        super::apply_panel_rename(&mut panel, "Renamed");
        assert_eq!(panel.name, "Renamed");
        super::apply_panel_rename(&mut panel, "");
        assert_eq!(panel.name, "Renamed");
    }

    #[test]
    fn workspace_rename_keeps_focus_out_of_terminal() {
        assert!(!super::terminal_should_have_focus(true, true, false));
        assert!(!super::terminal_should_have_focus(true, false, true));
        assert!(super::terminal_should_have_focus(true, false, false));
        assert!(!super::terminal_focus_lock_allowed(true, false));
        assert!(!super::terminal_focus_lock_allowed(false, true));
        assert!(super::terminal_focus_lock_allowed(false, false));
    }

    #[test]
    fn drag_feedback_marks_only_source_and_target_rows() {
        assert!(super::drag_row_is_source(2, Some(2)));
        assert!(!super::drag_row_is_source(1, Some(2)));
        assert!(super::drag_row_is_target(2, Some(2)));
        assert!(!super::drag_row_is_target(1, Some(2)));
    }

    #[test]
    fn drag_feedback_places_insertion_line_on_nearest_target_edge() {
        let rect = egui::Rect::from_min_max(egui::pos2(0.0, 10.0), egui::pos2(100.0, 30.0));

        assert_eq!(super::drag_insertion_y(rect, 12.0), rect.top());
        assert_eq!(super::drag_insertion_y(rect, 28.0), rect.bottom());
    }

    #[test]
    fn drag_feedback_drop_index_matches_insertion_edge() {
        assert_eq!(super::drag_drop_destination(0, 2, false, 4), 1);
        assert_eq!(super::drag_drop_destination(0, 2, true, 4), 2);
        assert_eq!(super::drag_drop_destination(3, 1, false, 4), 1);
        assert_eq!(super::drag_drop_destination(3, 1, true, 4), 2);
    }

    #[test]
    fn workspace_sidebar_uses_wider_default_and_drag_handle_width() {
        assert_eq!(super::WORKSPACE_SIDEBAR_DEFAULT_WIDTH, 192.0);
        assert_eq!(super::WORKSPACE_DRAG_HANDLE_WIDTH, 20.0);
    }

    #[test]
    fn last_terminal_tab_is_not_closeable() {
        assert!(!super::terminal_tab_is_closeable(1));
        assert!(super::terminal_tab_is_closeable(2));
    }

    #[test]
    fn escape_cancels_workspace_rename_without_changing_name() {
        let mut renaming_panel = Some(1);

        assert!(super::cancel_workspace_rename(&mut renaming_panel, true));
        assert_eq!(renaming_panel, None);
    }

    #[test]
    fn scene_serialization_preserves_workspace_order() {
        let scene = super::SceneState {
            panels: vec![
                super::ScenePanel {
                    name: "Workspace 2".into(),
                    dock_state: DockState::new(vec!["terminal-2".into()]),
                    terminals: [(
                        "terminal-2".into(),
                        super::TerminalStatePersist {
                            name: "Terminal 2".into(),
                            font_size: 14.0,
                            working_directory: ".".into(),
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
                super::ScenePanel {
                    name: "Workspace 1".into(),
                    dock_state: DockState::new(vec!["terminal-1".into()]),
                    terminals: [(
                        "terminal-1".into(),
                        super::TerminalStatePersist {
                            name: "Terminal 1".into(),
                            font_size: 14.0,
                            working_directory: ".".into(),
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
            ],
        };

        let serialized = serde_json::to_value(&scene).unwrap();
        assert_eq!(
            serialized["panels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|panel| panel["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["Workspace 2", "Workspace 1"]
        );
    }

    #[test]
    fn default_history_bindings_use_alt_menu_and_arrow_navigation() {
        let binds = default_key_binds();

        assert_eq!(binds["history_menu"].key, "Alt");
        assert!(binds["history_menu"].alt);
        assert_eq!(binds["history_prev"].key, "ArrowUp");
        assert_eq!(binds["history_next"].key, "ArrowDown");
    }

    #[test]
    fn standalone_alt_release_opens_menu_but_alt_combo_does_not() {
        let mut state = AltKeyState::default();

        assert!(!update_alt_key_state(&mut state, true, false));
        assert!(update_alt_key_state(&mut state, false, false));

        assert!(!update_alt_key_state(&mut state, true, false));
        assert!(!update_alt_key_state(&mut state, true, true));
        assert!(!update_alt_key_state(&mut state, false, false));
    }

    #[test]
    fn alt_combo_released_in_one_frame_does_not_open_menu() {
        let mut state = AltKeyState::default();

        assert!(!update_alt_key_state(&mut state, true, false));
        assert!(!update_alt_key_state(&mut state, false, true));
    }

    #[test]
    fn legacy_history_bindings_are_renamed_without_losing_custom_keys() {
        let mut settings = AppSettings::default();
        settings.key_binds.remove("history_menu");
        settings.key_binds.remove("history_prev");
        settings.key_binds.remove("history_next");
        settings.key_binds.insert(
            "history_up".into(),
            ShortcutBinding {
                key: "PageUp".into(),
                ctrl: true,
                shift: false,
                alt: false,
            },
        );
        settings.key_binds.insert(
            "history_down".into(),
            ShortcutBinding {
                key: "PageDown".into(),
                ctrl: false,
                shift: true,
                alt: false,
            },
        );

        let migrated = super::normalize_history_bindings(settings);
        assert_eq!(migrated.key_binds["history_menu"].key, "Alt");
        assert_eq!(migrated.key_binds["history_prev"].key, "PageUp");
        assert!(migrated.key_binds["history_prev"].ctrl);
        assert_eq!(migrated.key_binds["history_next"].key, "PageDown");
        assert!(migrated.key_binds["history_next"].shift);
        assert!(!migrated.key_binds.contains_key("history_up"));
        assert!(!migrated.key_binds.contains_key("history_down"));
    }

    #[test]
    fn history_navigation_stays_within_newest_to_oldest_entries() {
        let mut nav = HistoryNav {
            entries: vec!["newest".into(), "oldest".into()],
            selected: 0,
        };

        nav.move_previous();
        assert_eq!(nav.selected, 0);
        nav.move_next();
        assert_eq!(nav.selected, 1);
        nav.move_next();
        assert_eq!(nav.selected, 1);
    }

    #[test]
    fn history_menu_shortcut_toggles_open_and_closed() {
        let mut nav = None;
        toggle_history_menu(&mut nav, vec!["command".into()]);
        assert!(nav.is_some());
        toggle_history_menu(&mut nav, vec!["command".into()]);
        assert!(nav.is_none());
    }

    #[test]
    fn terminal_state_does_not_serialize_snapshots() {
        let state = TerminalStatePersist {
            name: "Terminal 1".into(),
            font_size: 14.0,
            working_directory: "/tmp".into(),
        };

        let value = serde_json::to_value(state).unwrap();
        assert!(!value.as_object().unwrap().contains_key("snapshot"));
    }

    #[test]
    fn legacy_terminal_state_ignores_snapshot_fields() {
        let state: TerminalStatePersist = serde_json::from_value(serde_json::json!({
            "name": "Terminal 1",
            "font_size": 14.0,
            "working_directory": "/tmp",
            "snapshot": {"grid": []},
            "process_info": {"pid": 1}
        }))
        .unwrap();

        assert_eq!(state.working_directory, "/tmp");
    }
}

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    completion: &'a crate::completion::CompletionEngine,
    history_db: &'a crate::history_db::HistoryDb,
    max_history: usize,
    cell_spacing: f32,
    bg_color: [u8; 3],
    fg_color: [u8; 3],
    menu_bg_color: [u8; 3],
    menu_fg_color: [u8; 3],
    menu_font_size: f32,
    pending_close: &'a mut Option<String>,
    pending_new_terminal: &'a mut Option<(usize, SurfaceIndex, NodeIndex)>,
    pending_split_after: &'a mut Option<String>,
    pending_split_vertical: &'a mut bool,
    active_panel: usize,
    terminal_count: usize,
    renaming_terminal: &'a mut Option<String>,
    terminal_rename_buffer: &'a mut String,
    renaming: bool,
    rename_frame_count: u32,
    active_tab: Option<String>,
    focused_terminal: &'a mut Option<String>,
    terminal_focus_id: &'a mut Option<egui::Id>,
    show_settings: bool,
    texts: &'a crate::i18n::Texts,
}

impl<'a> egui_dock::TabViewer for TerminalTabViewer<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.terminals
            .get(tab)
            .map(|d| d.name.clone().into())
            .unwrap_or_else(|| tab.clone().into())
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if self.renaming_terminal.as_ref() == Some(tab) {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(self.terminal_rename_buffer)
                        .font(egui::FontId::monospace(14.0))
                        .desired_width(200.0)
                        .hint_text(&self.texts.terminal.rename_hint)
                        .id_source("tab_rename"),
                );
                ui.memory_mut(|mem| mem.request_focus(response.id));
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if enter {
                    if !self.terminal_rename_buffer.is_empty() {
                        if let Some(data) = self.terminals.get_mut(tab) {
                            data.name = self.terminal_rename_buffer.clone();
                        }
                    }
                    *self.renaming_terminal = None;
                }
            });
            ui.separator();
        }

        if let Some(td) = self.terminals.get_mut(tab) {
            let mouse_over = ui.rect_contains_pointer(ui.clip_rect());
            if mouse_over {
                let scroll: f32 = ui.input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| {
                            if let egui::Event::MouseWheel {
                                delta, modifiers, ..
                            } = e
                            {
                                if modifiers.ctrl {
                                    Some(delta.y)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .sum()
                });
                if scroll > 0.0 {
                    td.font_size = (td.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                } else if scroll < 0.0 {
                    td.font_size = (td.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                }
            }
            if self.active_tab.as_ref() == Some(tab) {
                if ui.input(|i| i.key_pressed(egui::Key::Equals) && i.modifiers.ctrl) {
                    td.font_size = (td.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                }
                if ui.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
                    td.font_size = (td.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                }
            }

            let is_focused = terminal_should_have_focus(
                self.focused_terminal.as_ref() == Some(tab),
                self.renaming,
                self.renaming_terminal.is_some(),
            );

            // Auto-focus when this tab becomes active
            if self.active_tab.as_ref() == Some(tab) {
                *self.focused_terminal = Some(tab.clone());
            }

            // Close menu when terminal loses focus
            if !is_focused {
                td.instance.history_nav = None;
            }

            let terminal_view = {
                let mut tv = egui_term::TerminalView::new(ui, &mut td.instance.backend);
                tv = tv.set_theme(egui_term::TerminalTheme::new(Box::new(
                    egui_term::ColorPalette {
                        foreground: format!(
                            "#{:02x}{:02x}{:02x}",
                            self.fg_color[0], self.fg_color[1], self.fg_color[2]
                        ),
                        background: format!(
                            "#{:02x}{:02x}{:02x}",
                            self.bg_color[0], self.bg_color[1], self.bg_color[2]
                        ),
                        ..Default::default()
                    },
                )));
                tv = tv.set_font(egui_term::TerminalFont::new(egui_term::FontSettings {
                    font_type: egui::FontId::monospace(td.font_size),
                }));
                tv = tv.set_focus(is_focused);
                // Override keys handled by app-level history UI
                tv = tv.add_bindings(vec![
                    (
                        egui_term::Binding {
                            target: egui_term::InputKind::KeyCode(egui::Key::PageUp),
                            modifiers: egui::Modifiers::NONE,
                            terminal_mode_include: egui_term::TerminalMode::empty(),
                            terminal_mode_exclude: egui_term::TerminalMode::empty(),
                        },
                        egui_term::BindingAction::Ignore,
                    ),
                    (
                        egui_term::Binding {
                            target: egui_term::InputKind::KeyCode(egui::Key::PageDown),
                            modifiers: egui::Modifiers::NONE,
                            terminal_mode_include: egui_term::TerminalMode::empty(),
                            terminal_mode_exclude: egui_term::TerminalMode::empty(),
                        },
                        egui_term::BindingAction::Ignore,
                    ),
                    (
                        egui_term::Binding {
                            target: egui_term::InputKind::KeyCode(egui::Key::Enter),
                            modifiers: egui::Modifiers::NONE,
                            terminal_mode_include: egui_term::TerminalMode::empty(),
                            terminal_mode_exclude: egui_term::TerminalMode::empty(),
                        },
                        egui_term::BindingAction::Ignore,
                    ),
                    (
                        egui_term::Binding {
                            target: egui_term::InputKind::KeyCode(egui::Key::Escape),
                            modifiers: egui::Modifiers::NONE,
                            terminal_mode_include: egui_term::TerminalMode::empty(),
                            terminal_mode_exclude: egui_term::TerminalMode::empty(),
                        },
                        egui_term::BindingAction::Ignore,
                    ),
                ]);
                tv
            };
            let terminal_response = ui.add(terminal_view);

            if is_focused {
                *self.terminal_focus_id = Some(terminal_response.id);
            }

            // TerminalView owns keyboard focus and its arrow-key focus lock.
            if terminal_response.clicked() {
                *self.focused_terminal = Some(tab.clone());
            }

            if is_focused && !self.renaming && !self.show_settings {
                // Cache cell_h for overlay rendering
                let cell_h = ui.fonts(|f| f.row_height(&egui::FontId::monospace(td.font_size)));

                // History overlay: entries are [newest ... oldest], selected=0 is top
                if let Some(ref nav) = td.instance.history_nav {
                    let (_, cursor_row) = td.instance.cursor_position();
                    let list_width = 400.0;
                    let max_visible = 10;
                    let visible_count = nav.entries.len().min(max_visible);
                    let list_height = visible_count as f32 * cell_h;
                    let below_top =
                        terminal_response.rect.min.y + (cursor_row as f32 + 1.0) * cell_h;
                    let below_space = terminal_response.rect.max.y - below_top;
                    let list_top = if below_space >= list_height {
                        below_top
                    } else {
                        (terminal_response.rect.min.y + cursor_row as f32 * cell_h - list_height)
                            .max(terminal_response.rect.min.y)
                    };
                    let list_rect = egui::Rect::from_min_size(
                        egui::pos2(terminal_response.rect.min.x, list_top),
                        egui::vec2(list_width, list_height),
                    );
                    let menu_bg = egui::Color32::from_rgb(
                        self.menu_bg_color[0],
                        self.menu_bg_color[1],
                        self.menu_bg_color[2],
                    );
                    let menu_fg = egui::Color32::from_rgb(
                        self.menu_fg_color[0],
                        self.menu_fg_color[1],
                        self.menu_fg_color[2],
                    );
                    ui.painter().rect_filled(list_rect, 0.0, menu_bg);

                    let start_idx = if nav.selected >= max_visible {
                        nav.selected - max_visible + 1
                    } else {
                        0
                    };
                    for (i, entry) in nav.entries[start_idx..]
                        .iter()
                        .enumerate()
                        .take(max_visible)
                    {
                        let y = list_top + i as f32 * cell_h;
                        let is_selected = start_idx + i == nav.selected;
                        let item_bg = if is_selected {
                            egui::Color32::from_rgba_unmultiplied(60, 60, 80, 255)
                        } else {
                            menu_bg
                        };
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(terminal_response.rect.min.x, y),
                                egui::vec2(list_width, cell_h),
                            ),
                            0.0,
                            item_bg,
                        );
                        ui.painter().text(
                            egui::pos2(terminal_response.rect.min.x + 4.0, y),
                            egui::Align2::LEFT_TOP,
                            entry,
                            egui::FontId::monospace(self.menu_font_size),
                            menu_fg,
                        );
                    }
                }
            }
        } else {
            ui.label(&self.texts.terminal.not_found);
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        *self.pending_close = Some(tab.clone());
        true
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        terminal_tab_is_closeable(self.terminal_count)
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        *self.pending_new_terminal = Some((self.active_panel, surface, node));
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, surface: SurfaceIndex, node: NodeIndex) {
        ui.horizontal(|ui| {
            if ui.button(&self.texts.terminal.add_tab).clicked() {
                *self.pending_new_terminal = Some((self.active_panel, surface, node));
                ui.close_menu();
            }
        });
    }

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut Self::Tab,
        surface: SurfaceIndex,
        node: NodeIndex,
    ) {
        if ui.button(&self.texts.terminal.rename).clicked() {
            *self.renaming_terminal = Some(tab.clone());
            if let Some(data) = self.terminals.get(tab) {
                *self.terminal_rename_buffer = data.name.clone();
            }
            self.rename_frame_count = 0;
            ui.close_menu();
        }
        ui.separator();
        if ui.button(&self.texts.terminal.clear_history).clicked() {
            self.history_db.clear(tab);
            ui.close_menu();
        }
        ui.separator();
        if ui.button(&self.texts.terminal.new_tab).clicked() {
            *self.pending_new_terminal = Some((self.active_panel, surface, node));
            ui.close_menu();
        }
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

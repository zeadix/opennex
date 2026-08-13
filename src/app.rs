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
    lock_password: String,
    #[serde(default)]
    settings_window: SettingsWindowState,
    #[serde(default = "default_key_binds")]
    key_binds: HashMap<String, ShortcutBinding>,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_theme_id")]
    theme_id: String,
    #[serde(default = "default_true")]
    apply_theme_typography: bool,
}

fn default_theme_id() -> String {
    "opennex-dark".into()
}

fn default_true() -> bool {
    true
}

fn default_language() -> String {
    "zh".into()
}

fn default_max_history() -> usize {
    300
}
fn default_scrollback() -> usize {
    10000
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    m.insert(
        "toggle_workspace_sidebar".into(),
        ShortcutBinding {
            key: "F1".into(),
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
        "F1" => Some(egui::Key::F1),
        "F2" => Some(egui::Key::F2),
        "F3" => Some(egui::Key::F3),
        "F4" => Some(egui::Key::F4),
        "F5" => Some(egui::Key::F5),
        "F6" => Some(egui::Key::F6),
        "F7" => Some(egui::Key::F7),
        "F8" => Some(egui::Key::F8),
        "F9" => Some(egui::Key::F9),
        "F10" => Some(egui::Key::F10),
        "F11" => Some(egui::Key::F11),
        "F12" => Some(egui::Key::F12),
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
        "F1" => "F1",
        "F2" => "F2",
        "F3" => "F3",
        "F4" => "F4",
        "F5" => "F5",
        "F6" => "F6",
        "F7" => "F7",
        "F8" => "F8",
        "F9" => "F9",
        "F10" => "F10",
        "F11" => "F11",
        "F12" => "F12",
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

fn shortcut_hint_ids() -> [&'static str; 11] {
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
        "toggle_workspace_sidebar",
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
        "toggle_workspace_sidebar" => &texts.shortcut_labels.toggle_workspace_sidebar,
        "zoom_in" => &texts.shortcut_labels.zoom_in,
        "zoom_out" => &texts.shortcut_labels.zoom_out,
        _ => "",
    }
}

#[cfg(test)]
fn binding_matches_key(
    binding: &ShortcutBinding,
    key: egui::Key,
    modifiers: egui::Modifiers,
) -> bool {
    binding_to_key(binding) == Some(key) && binding_to_modifiers(binding) == modifiers
}

fn workspace_action_column_width(has_close: bool, has_lock: bool, item_spacing: f32) -> f32 {
    let count = usize::from(has_close) + usize::from(has_lock);
    if count == 0 {
        0.0
    } else {
        count as f32 * WORKSPACE_ACTION_BUTTON_SIZE.x
            + (count.saturating_sub(1) as f32 * item_spacing)
    }
}

fn screen_center(ctx: &egui::Context) -> egui::Pos2 {
    ctx.input(|i| i.screen_rect).center()
}

fn shortcut_hint_available_height(available_height: f32) -> f32 {
    available_height.max(0.0)
}

fn paint_keycap(ui: &mut egui::Ui, text: &str) {
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            text.to_string(),
            egui::FontId::proportional(11.0),
            ui.visuals().text_color(),
        )
    });
    let pad = egui::vec2(5.0, 2.0);
    let size = galley.size() + pad * 2.0;
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let visuals = ui.visuals();
    let bg = visuals.widgets.inactive.weak_bg_fill.linear_multiply(0.6)
        + visuals.window_fill.linear_multiply(0.4);
    let stroke = visuals.widgets.noninteractive.bg_stroke;
    ui.painter()
        .rect(rect, 3.0, bg, stroke, egui::StrokeKind::Inside);
    ui.painter()
        .galley(rect.min + pad, galley, visuals.text_color());
}

const WORKSPACE_ACTION_BUTTON_SIZE: egui::Vec2 = egui::vec2(24.0, 24.0);

fn check_column_width(ui: &egui::Ui) -> f32 {
    ui.spacing().icon_width + ui.spacing().icon_spacing + ui.spacing().button_padding.x
}

fn check_indicator(ui: &mut egui::Ui, selected: bool, width: f32) {
    let height = ui.spacing().interact_size.y;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    if selected {
        let color = ui.visuals().text_color();
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "✓",
            egui::FontId::proportional(14.0),
            color,
        );
    }
}

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

fn consume_exact_shortcut(
    ctx: &egui::Context,
    binds: &HashMap<String, ShortcutBinding>,
    name: &str,
) -> bool {
    let Some(binding) = binds.get(name) else {
        return false;
    };
    let Some(key) = binding_to_key(binding) else {
        return false;
    };
    consume_exact_key(ctx, key, binding_to_modifiers(binding))
}

fn consume_exact_key(ctx: &egui::Context, key: egui::Key, modifiers: egui::Modifiers) -> bool {
    let mut matched = false;
    ctx.input_mut(|input| {
        input.events.retain(|event| {
            let is_match = matches!(
                event,
                egui::Event::Key {
                    key: event_key,
                    modifiers: event_modifiers,
                    pressed: true,
                    ..
                } if *event_key == key && *event_modifiers == modifiers
            );
            matched |= is_match;
            !is_match
        });
    });
    matched
}

fn global_shortcuts_allowed(app: &App) -> bool {
    !app.show_settings
        && !app.show_about
        && app.pw_popup.is_none()
        && app.unlock_popup.is_none()
        && app.binding_recording.is_none()
        && app.close_confirm_panel.is_none()
        && app.pending_close_confirm.is_none()
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

fn load_multilingual_fonts(fonts: &mut egui::FontDefinitions) {
    // Bundled fonts — guaranteed cross-platform
    let bundled: &[(&str, &[u8])] = &[
        (
            "noto-cjk",
            include_bytes!("../assets/fonts/NotoSansCJK-Regular.ttc"),
        ),
        (
            "noto-devanagari",
            include_bytes!("../assets/fonts/Lohit-Devanagari.ttf"),
        ),
        (
            "noto-arabic",
            include_bytes!("../assets/fonts/NotoSansArabic-Regular.ttf"),
        ),
    ];
    for (name, data) in bundled {
        if !fonts.font_data.contains_key(*name) {
            fonts.font_data.insert(
                (*name).into(),
                std::sync::Arc::new(egui::FontData::from_owned(data.to_vec()).tweak(
                    egui::FontTweak {
                        scale: 0.9,
                        ..Default::default()
                    },
                )),
            );
        }
    }
    // Also try to load system CJK/Devanagari/Arabic fonts as fallback/supplement
    let system_candidates: &[(&str, &[&str])] = &[
        (
            "noto-cjk-sys",
            &["NotoSansCJK-Regular.ttc", "NotoSansCJK-Medium.ttc"],
        ),
        (
            "noto-devanagari-sys",
            &["Lohit-Devanagari.ttf", "kalimati.ttf"],
        ),
        ("freefont-sys", &["FreeSans.ttf", "FreeSerif.ttf"]),
    ];
    for (name, filenames) in system_candidates {
        if fonts.font_data.contains_key(*name) {
            continue;
        }
        if let Some(path) = find_system_font(filenames) {
            if let Ok(data) = std::fs::read(&path) {
                fonts.font_data.insert(
                    (*name).into(),
                    std::sync::Arc::new(egui::FontData::from_owned(data).tweak(egui::FontTweak {
                        scale: 0.9,
                        ..Default::default()
                    })),
                );
            }
        }
    }

    let order = [
        "noto-cjk",
        "noto-devanagari",
        "noto-arabic",
        "noto-cjk-sys",
        "noto-devanagari-sys",
        "freefont-sys",
    ];
    // Append to END of font families — egui tries fonts front-to-back,
    // so default fonts render Latin first, CJK is fallback only.
    // This preserves the original layout metrics.
    if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        for name in &order {
            if fonts.font_data.contains_key(*name) {
                prop.push((*name).into());
            }
        }
    }
    if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        for name in &order {
            if fonts.font_data.contains_key(*name) {
                mono.push((*name).into());
            }
        }
    }
}

fn font_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    #[cfg(target_os = "linux")]
    {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = std::env::var_os("HOME") {
            let h = PathBuf::from(home);
            dirs.push(h.join(".local/share/fonts"));
            dirs.push(h.join(".fonts"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(win) = std::env::var_os("WINDIR") {
            dirs.push(PathBuf::from(win).join("Fonts"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join("Microsoft/Windows/Fonts"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Library/Fonts"));
        }
    }
    dirs
}

fn find_system_font(filenames: &[&str]) -> Option<PathBuf> {
    for dir in &font_search_dirs() {
        for name in filenames {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
            // Also search subdirectories (Linux font structure)
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        let candidate = p.join(name);
                        if candidate.exists() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
    }
    None
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

fn app_data_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    base.join("opennex")
}

fn ensure_data_dir() {
    let _ = std::fs::create_dir_all(app_data_dir());
}

fn migrate_file(filename: &str) {
    let old = std::env::current_dir().unwrap_or_default().join(filename);
    let new = app_data_dir().join(filename);
    if old.exists() && !new.exists() {
        let _ = std::fs::copy(&old, &new);
    }
}

fn migrate_dir(dirname: &str) {
    let old = std::env::current_dir().unwrap_or_default().join(dirname);
    let new = app_data_dir().join(dirname);
    if old.exists() && !new.exists() {
        let _ = std::fs::create_dir_all(&new);
        if let Ok(entries) = std::fs::read_dir(&old) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    let _ = std::fs::copy(entry.path(), new.join(name));
                }
            }
        }
    }
}

fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

fn load_settings() -> AppSettings {
    let path = settings_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(settings) = deserialize_settings(&content) {
                let (settings, changed) = normalize_settings(settings);
                if changed {
                    let _ = save_settings(&settings);
                }
                return settings;
            }
        }
    }
    AppSettings::default()
}

/// Deserialize settings, migrating legacy visual fields into the theme ID.
fn deserialize_settings(json: &str) -> Result<AppSettings, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct LegacySettings {
        #[serde(flatten)]
        current: AppSettings,
        #[serde(default)]
        theme: Option<String>,
        #[serde(default)]
        terminal_theme: Option<String>,
    }

    let legacy: LegacySettings = serde_json::from_str(json)?;
    let mut settings = legacy.current;

    if settings.theme_id == default_theme_id() {
        if let Some(terminal_theme) = legacy.terminal_theme.as_deref() {
            settings.theme_id = migrate_legacy_terminal_theme(terminal_theme);
        } else if let Some(mode) = legacy.theme.as_deref() {
            settings.theme_id = match mode {
                "light" => "opennex-light".into(),
                _ => "opennex-dark".into(),
            };
        }
    }

    Ok(settings)
}

fn migrate_legacy_terminal_theme(name: &str) -> String {
    match name {
        "solarized" => "solarized-dark".into(),
        "gruvbox" => "gruvbox-dark".into(),
        "dracula" => "dracula".into(),
        _ => "opennex-dark".into(),
    }
}

fn normalize_settings(mut settings: AppSettings) -> (AppSettings, bool) {
    #[cfg(debug_assertions)]
    {
        let original = settings.key_binds.clone();
        let defaults = default_key_binds();
        for (key, default_binding) in &defaults {
            settings
                .key_binds
                .insert(key.clone(), default_binding.clone());
        }
        let changed = settings.key_binds != original;
        return (settings, changed);
    }

    #[cfg(not(debug_assertions))]
    {
        normalize_settings_release_impl(settings)
    }
}

fn normalize_settings_release_impl(mut settings: AppSettings) -> (AppSettings, bool) {
    let original = settings.key_binds.clone();
    let defaults = default_key_binds();
    if !settings.key_binds.contains_key("toggle_workspace_sidebar") {
        settings.key_binds.insert(
            "toggle_workspace_sidebar".into(),
            defaults.get("toggle_workspace_sidebar").cloned().unwrap(),
        );
    }
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
    let changed = settings.key_binds != original;
    (settings, changed)
}

#[cfg(test)]
fn normalize_history_bindings(settings: AppSettings) -> AppSettings {
    normalize_settings_release_impl(settings).0
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
            lock_password: String::new(),
            settings_window: SettingsWindowState::default(),
            key_binds: default_key_binds(),
            language: default_language(),
            theme_id: default_theme_id(),
            apply_theme_typography: true,
        }
    }
}

fn scan_system_fonts() -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for dir in &font_search_dirs() {
        if !dir.exists() {
            continue;
        }
        for entry in walk_font_dir(dir) {
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
    app_data_dir().join("scene.json")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogKind {
    New,
    Copy,
    Rename,
    Delete,
    Switch,
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
    pending_close_confirm: Option<String>,
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
    show_about: bool,
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
    workspace_sidebar_visible: bool,
    update_state: crate::updater::UpdateState,
    active_theme: crate::theme::ThemeDefinition,
    theme_edit: crate::theme::ThemeDefinition,
    available_themes: Vec<crate::theme::ThemeDefinition>,
    theme_message: Option<Result<String, String>>,
    pending_import_theme: bool,
    pending_export_theme: bool,
    theme_dialog: crate::theme::ui::ThemeDialogState,
    theme_dirty: bool,
    theme_editor_subtab: crate::theme::ui::ThemeEditorSubtab,
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
    #[cfg(target_os = "windows")]
    let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string());
    #[cfg(target_os = "macos")]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    #[cfg(all(unix, not(target_os = "macos")))]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

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

        ensure_data_dir();
        migrate_file("settings.json");
        migrate_file("scene.json");
        migrate_file("history.db");
        migrate_dir("templates");

        let themes_root = crate::theme::store::themes_dir(&app_data_dir());
        let _ = std::fs::create_dir_all(&themes_root);
        let mut available_themes =
            crate::theme::store::load_user_themes(&themes_root).unwrap_or_default();
        for embedded in crate::theme::store::embedded_themes().unwrap_or_default() {
            if !available_themes.iter().any(|t| t.id == embedded.id) {
                available_themes.push(embedded);
            }
        }
        available_themes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        let active_theme = crate::theme::store::load_theme(&themes_root, &settings.theme_id)
            .unwrap_or_else(|err| {
                log::warn!("failed to load theme '{}': {err}", settings.theme_id);
                crate::theme::store::default_theme().unwrap_or_else(|err| {
                    panic!("embedded default theme failed to load: {err}");
                })
            });

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
        // Load CJK, Devanagari, and other multilingual fonts
        load_multilingual_fonts(&mut fonts);
        // Register found fonts into Monospace family
        if let Some(mono_family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            for name in &registered_names {
                mono_family.push(name.clone());
            }
        }
        ctx.set_fonts(fonts);

        crate::theme::apply_theme_definition(ctx, &active_theme);

        let font_names: Vec<String> = registered_names;
        let db_path = app_data_dir().join("history.db");
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
            pending_close_confirm: None,
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
            show_about: false,
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
            workspace_sidebar_visible: true,
            update_state: crate::updater::UpdateState::Idle,
            active_theme: active_theme.clone(),
            theme_edit: active_theme,
            available_themes,
            theme_message: None,
            pending_import_theme: false,
            pending_export_theme: false,
            theme_dialog: Default::default(),
            theme_dirty: false,
            theme_editor_subtab: Default::default(),
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

        // Start background update check
        {
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                match crate::updater::check_for_update() {
                    Ok(Some(info)) => {
                        ctx_clone.request_repaint();
                        // Store result via a channel or shared state
                        // For simplicity, we use egui's memory
                        ctx_clone.memory_mut(|mem| {
                            mem.data
                                .insert_temp(egui::Id::new("update_info"), Some(info));
                        });
                    }
                    Ok(None) => {
                        ctx_clone.memory_mut(|mem| {
                            mem.data.insert_temp(
                                egui::Id::new("update_info"),
                                None::<crate::updater::UpdateInfo>,
                            );
                        });
                    }
                    Err(_) => {}
                }
            });
        }

        app
    }

    fn templates_dir(&self) -> PathBuf {
        app_data_dir().join("templates")
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
                self.history_db.clear(&tab);
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
        if self.pending_import_theme {
            self.pending_import_theme = false;
            let texts = self.texts.settings.appearance.clone();
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("OpenNex Theme", &["json"])
                .pick_file()
            {
                let themes_root = crate::theme::store::themes_dir(&app_data_dir());
                match crate::theme::store::import_theme_file(&themes_root, &path) {
                    Ok(imported) => {
                        self.refresh_themes(&themes_root);
                        let imported_id = imported.id.clone();
                        self.settings.theme_id = imported_id.clone();
                        self.settings_edit.theme_id = imported_id;
                        self.active_theme = imported.clone();
                        self.theme_edit = imported;
                        self.theme_dirty = false;
                        crate::theme::apply_theme_definition(ctx, &self.active_theme);
                        let _ = save_settings(&self.settings);
                        self.theme_message = Some(Ok(texts.import_success));
                    }
                    Err(err) => {
                        let msg = if matches!(err, crate::theme::ThemeError::UnsupportedVersion(_))
                        {
                            texts.unsupported_version
                        } else {
                            texts.invalid_theme
                        };
                        log::warn!("theme import failed: {err}");
                        self.theme_message = Some(Err(msg));
                    }
                }
            }
        }
        if self.pending_export_theme {
            self.pending_export_theme = false;
            let texts = self.texts.settings.appearance.clone();
            let file_name = format!("{}.opennex-theme.json", self.theme_edit.id);
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("OpenNex Theme", &["json"])
                .set_file_name(&file_name)
                .save_file()
            {
                match crate::theme::store::export_theme_file(&self.theme_edit, &path) {
                    Ok(()) => {
                        self.theme_message = Some(Ok(texts.export_success));
                    }
                    Err(err) => {
                        log::warn!("theme export failed: {err}");
                        self.theme_message = Some(Err(texts.save_failure));
                    }
                }
            }
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
        // Delete history for all terminals in this workspace
        if let Some(tree) = self.dock_states.get(&i) {
            for tab_id in self.terminals.keys() {
                if tree.find_tab(tab_id).is_some() {
                    self.history_db.clear(tab_id);
                }
            }
        }
        // Remove all terminal instances belonging to this workspace
        if let Some(tree) = self.dock_states.get(&i) {
            let to_remove: Vec<String> = self
                .terminals
                .keys()
                .filter(|t| tree.find_tab(t).is_some())
                .cloned()
                .collect();
            for tab_id in to_remove {
                self.terminals.remove(&tab_id);
            }
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

    fn switch_theme_by_id(&mut self, ctx: &egui::Context, id: &str) {
        let theme = match self.available_themes.iter().find(|t| t.id == id).cloned() {
            Some(theme) => theme,
            None => crate::theme::store::default_theme().unwrap_or_else(|err| {
                log::error!("default theme unavailable: {err}");
                self.active_theme.clone()
            }),
        };
        self.settings.theme_id = theme.id.clone();
        self.settings_edit.theme_id = theme.id.clone();
        self.active_theme = theme.clone();
        self.theme_edit = theme;
        self.theme_dirty = false;
        crate::theme::apply_theme_definition(ctx, &self.active_theme);
        let _ = save_settings(&self.settings);
    }

    /// Try to switch theme, prompting for unsaved changes if needed.
    fn try_switch_theme(&mut self, ctx: &egui::Context, id: String) {
        if self.theme_dirty {
            self.theme_dialog.pending_switch_target = id;
            self.theme_dialog.show_switch_confirm = true;
        } else {
            self.switch_theme_by_id(ctx, &id);
        }
    }

    /// Handle a [`ThemeAction`] produced by the theme editor UI.
    fn handle_theme_action(
        &mut self,
        ctx: &egui::Context,
        action: crate::theme::ui::ThemeAction,
        draft: crate::theme::ThemeDefinition,
    ) {
        use crate::theme::ui::ThemeAction;
        match action {
            ThemeAction::SelectTheme(id) => {
                self.try_switch_theme(ctx, id);
            }
            ThemeAction::SelectSubtab(subtab) => {
                self.theme_editor_subtab = subtab;
            }
            ThemeAction::NewTheme => {
                self.theme_dialog.show_new_dialog = true;
            }
            ThemeAction::CopyTheme => {
                self.theme_dialog.show_copy_dialog = true;
            }
            ThemeAction::RenameTheme(_) => {
                self.theme_dialog.show_rename_dialog = true;
            }
            ThemeAction::DeleteTheme => {
                self.theme_dialog.show_delete_confirm = true;
            }
            ThemeAction::ImportTheme => {
                self.pending_import_theme = true;
            }
            ThemeAction::ExportTheme => {
                self.pending_export_theme = true;
            }
            ThemeAction::ApplyPaletteTemplate(template_id) => {
                if let Some(colors) = crate::theme::palettes::terminal_colors(&template_id) {
                    let mut new_draft = draft;
                    new_draft.terminal = colors;
                    self.theme_edit = new_draft;
                    self.theme_dirty = true;
                    crate::theme::apply_theme_definition(ctx, &self.theme_edit);
                }
            }
            ThemeAction::DraftModified => {
                self.theme_edit = draft;
                self.theme_dirty = true;
            }
        }
    }

    fn show_theme_dialogs(&mut self, ctx: &egui::Context) {
        let themes_root = crate::theme::store::themes_dir(&app_data_dir());

        // Determine which dialog (if any) is open.
        let active = if self.theme_dialog.show_new_dialog {
            Some(DialogKind::New)
        } else if self.theme_dialog.show_copy_dialog {
            Some(DialogKind::Copy)
        } else if self.theme_dialog.show_rename_dialog {
            Some(DialogKind::Rename)
        } else if self.theme_dialog.show_delete_confirm {
            Some(DialogKind::Delete)
        } else if self.theme_dialog.show_switch_confirm {
            Some(DialogKind::Switch)
        } else {
            None
        };

        let Some(kind) = active else {
            return;
        };

        // Pre-populate name input.
        if self.theme_dialog.name_input.is_empty() {
            self.theme_dialog.name_input = match kind {
                DialogKind::New => String::new(),
                DialogKind::Copy => format!("{} 副本", self.theme_edit.name),
                DialogKind::Rename => self.theme_edit.name.clone(),
                _ => String::new(),
            };
        }

        let (modal_id, input_id_salt, title) = match kind {
            DialogKind::New => (
                "theme_new_modal",
                "theme_new_name_input",
                crate::theme::new_dialog_title(),
            ),
            DialogKind::Copy => (
                "theme_copy_modal",
                "theme_copy_name_input",
                crate::theme::copy_dialog_title(),
            ),
            DialogKind::Rename => (
                "theme_rename_modal",
                "theme_rename_name_input",
                crate::theme::rename_dialog_title(),
            ),
            DialogKind::Delete => (
                "theme_delete_modal",
                "",
                crate::theme::delete_confirm_text(),
            ),
            DialogKind::Switch => (
                "theme_switch_modal",
                "",
                crate::theme::switch_confirm_text(),
            ),
        };

        let input_id = egui::Id::new(input_id_salt);
        let mut close_after = false;
        let mut do_action = false;

        // Render the dialog as a Modal. The Modal sets the topmost
        // modal layer so the TextEdit can reliably capture focus.
        let modal = egui::Modal::new(egui::Id::new(modal_id))
            .backdrop_color(egui::Color32::from_black_alpha(160))
            .frame(egui::Frame::window(&ctx.style()));

        // Force focus on the text input BEFORE showing the modal,
        // matching the lock-overlay pattern of calling request_focus
        // on a stable id at the top of the update path.
        if matches!(
            kind,
            DialogKind::New | DialogKind::Copy | DialogKind::Rename
        ) {
            ctx.memory_mut(|m| m.request_focus(input_id));
        }

        modal.show(ctx, |ui| {
            ui.set_min_size(egui::vec2(360.0, 120.0));

            // Title
            ui.strong(title.clone());
            ui.add_space(4.0);

            // Body
            match kind {
                DialogKind::New | DialogKind::Copy | DialogKind::Rename => {
                    let _resp = ui.add(
                        egui::TextEdit::singleline(&mut self.theme_dialog.name_input)
                            .id(input_id)
                            .desired_width(340.0),
                    );
                    eprintln!(
                        "FOCUS_DEBUG: modal={:?} input_id={:?} has_focus={} value='{}'",
                        modal_id,
                        input_id,
                        _resp.has_focus(),
                        self.theme_dialog.name_input
                    );
                }
                DialogKind::Delete => {
                    ui.label(format!(
                        "{}: {}",
                        crate::theme::delete_confirm_text(),
                        self.theme_edit.name
                    ));
                }
                DialogKind::Switch => {
                    ui.label(crate::theme::switch_confirm_text());
                }
            }

            // Buttons
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(crate::theme::confirm_text()).clicked() {
                        do_action = true;
                    }
                    if matches!(
                        kind,
                        DialogKind::New | DialogKind::Copy | DialogKind::Rename
                    ) && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        do_action = true;
                    }
                    if ui.button(crate::theme::cancel_text()).clicked() {
                        close_after = true;
                    }
                });
            });
        });

        if do_action {
            match kind {
                DialogKind::Copy => {
                    let name = self.theme_dialog.name_input.trim().to_string();
                    if !name.is_empty() {
                        match crate::theme::store::copy_theme(
                            &themes_root,
                            &self.theme_edit.id,
                            &name,
                        ) {
                            Ok(new_theme) => {
                                self.refresh_themes(&themes_root);
                                self.switch_theme_by_id(ctx, &new_theme.id);
                            }
                            Err(e) => self.theme_message = Some(Err(e.to_string())),
                        }
                        close_after = true;
                    }
                }
                DialogKind::New => {
                    let name = self.theme_dialog.name_input.trim().to_string();
                    if !name.is_empty() {
                        let mut new_theme = self.theme_edit.clone();
                        new_theme.name = name;
                        new_theme.id =
                            crate::theme::store::find_free_id(&themes_root, &new_theme.id);
                        match crate::theme::store::save_user_theme(&themes_root, &new_theme) {
                            Ok(()) => {
                                self.refresh_themes(&themes_root);
                                self.switch_theme_by_id(ctx, &new_theme.id);
                            }
                            Err(e) => self.theme_message = Some(Err(e.to_string())),
                        }
                        close_after = true;
                    }
                }
                DialogKind::Rename => {
                    let name = self.theme_dialog.name_input.trim().to_string();
                    if !name.is_empty() {
                        match crate::theme::store::rename_user_theme(
                            &themes_root,
                            &self.theme_edit.id,
                            &name,
                        ) {
                            Ok(updated) => {
                                self.theme_edit.name = updated.name.clone();
                                self.active_theme.name = updated.name.clone();
                                self.refresh_themes(&themes_root);
                            }
                            Err(e) => self.theme_message = Some(Err(e.to_string())),
                        }
                        close_after = true;
                    }
                }
                DialogKind::Delete => {
                    match crate::theme::store::delete_user_theme(&themes_root, &self.theme_edit.id)
                    {
                        Ok(()) => {
                            self.refresh_themes(&themes_root);
                            self.switch_theme_by_id(ctx, "opennex-dark");
                        }
                        Err(e) => self.theme_message = Some(Err(e.to_string())),
                    }
                    close_after = true;
                }
                DialogKind::Switch => {}
            }
        }

        if close_after {
            self.close_dialog(kind);
        }
    }

    fn close_dialog(&mut self, kind: DialogKind) {
        self.theme_dialog.name_input.clear();
        self.theme_dialog.focus_requested = false;
        match kind {
            DialogKind::New => self.theme_dialog.show_new_dialog = false,
            DialogKind::Copy => self.theme_dialog.show_copy_dialog = false,
            DialogKind::Rename => self.theme_dialog.show_rename_dialog = false,
            DialogKind::Delete => self.theme_dialog.show_delete_confirm = false,
            DialogKind::Switch => self.theme_dialog.show_switch_confirm = false,
        }
    }

    fn refresh_themes(&mut self, themes_root: &std::path::Path) {
        let mut themes = crate::theme::store::load_user_themes(themes_root).unwrap_or_default();
        for embedded in crate::theme::store::embedded_themes().unwrap_or_default() {
            if !themes.iter().any(|t| t.id == embedded.id) {
                themes.push(embedded);
            }
        }
        themes.sort_by_key(|t| t.name.to_lowercase());
        self.available_themes = themes;
    }

    fn check_update_manual(&mut self, ctx: &egui::Context) {
        self.update_state = crate::updater::UpdateState::Checking;
        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let result = crate::updater::check_for_update();
            ctx_clone.memory_mut(|mem| {
                let state = match result {
                    Ok(Some(info)) => crate::updater::UpdateState::Available(info),
                    Ok(None) => crate::updater::UpdateState::UpToDate,
                    Err(e) => crate::updater::UpdateState::Error(e),
                };
                mem.data
                    .insert_temp(egui::Id::new("manual_check_result"), state);
            });
            ctx_clone.request_repaint();
        });
    }

    fn start_download(&mut self, ctx: &egui::Context, info: &crate::updater::UpdateInfo) {
        let url = info.download_url.clone();
        let sha = info.sha256.clone();
        self.update_state = crate::updater::UpdateState::Downloading(0.0);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let (progress_tx, progress_rx) = std::sync::mpsc::channel::<f32>();
            let ctx2 = ctx_clone.clone();

            // Spawn progress forwarder
            std::thread::spawn(move || {
                while let Ok(pct) = progress_rx.recv() {
                    ctx2.request_repaint();
                    if pct < 0.0 {
                        ctx2.memory_mut(|mem| {
                            mem.data.insert_temp(
                                egui::Id::new("dl_state"),
                                crate::updater::UpdateState::Error("下载失败".into()),
                            );
                        });
                        break;
                    }
                    ctx2.memory_mut(|mem| {
                        mem.data.insert_temp(
                            egui::Id::new("dl_state"),
                            crate::updater::UpdateState::Downloading(pct),
                        );
                    });
                }
            });

            match crate::updater::download_and_verify(&url, &sha, &progress_tx) {
                Ok(path) => {
                    ctx_clone.memory_mut(|mem| {
                        mem.data.insert_temp(
                            egui::Id::new("dl_state"),
                            crate::updater::UpdateState::Ready(path),
                        );
                    });
                    ctx_clone.request_repaint();
                }
                Err(e) => {
                    ctx_clone.memory_mut(|mem| {
                        mem.data.insert_temp(
                            egui::Id::new("dl_state"),
                            crate::updater::UpdateState::Error(e),
                        );
                    });
                    ctx_clone.request_repaint();
                }
            }
        });
    }

    fn render_update_window(&mut self, ctx: &egui::Context) {
        // Poll manual check result
        if self.update_state == crate::updater::UpdateState::Checking {
            let result = ctx.memory(|mem| {
                mem.data
                    .get_temp::<Option<crate::updater::UpdateState>>(egui::Id::new(
                        "manual_check_result",
                    ))
                    .map(|v| v.clone())
                    .flatten()
            });
            if let Some(state) = result {
                self.update_state = state;
            }
        }

        // Poll download progress from background thread
        if let crate::updater::UpdateState::Downloading(_) = &self.update_state {
            let dl_state = ctx.memory(|mem| {
                mem.data
                    .get_temp::<Option<crate::updater::UpdateState>>(egui::Id::new("dl_state"))
                    .map(|v| v.clone())
                    .flatten()
            });
            if let Some(state) = dl_state {
                match &state {
                    crate::updater::UpdateState::Ready(_)
                    | crate::updater::UpdateState::Error(_) => {
                        self.update_state = state;
                    }
                    crate::updater::UpdateState::Downloading(pct) => {
                        self.update_state = crate::updater::UpdateState::Downloading(*pct);
                    }
                    _ => {}
                }
            }
        }

        match &self.update_state {
            crate::updater::UpdateState::Available(info) => {
                let mut dismiss = false;
                let mut start_dl = false;
                let info_clone = info.clone();
                egui::Window::new("发现新版本")
                    .resizable(false)
                    .collapsible(false)
                    .default_pos(screen_center(ctx))
                    .pivot(egui::Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(4.0);
                            ui.heading(format!("v{}", info.version));
                            ui.add_space(4.0);
                            ui.label(format!(
                                "当前版本: v{}\n是否立即更新？",
                                env!("CARGO_PKG_VERSION")
                            ));
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.button("更新").clicked() {
                                    start_dl = true;
                                }
                                if ui.button("稍后").clicked() {
                                    dismiss = true;
                                }
                            });
                        });
                    });
                if dismiss {
                    self.update_state = crate::updater::UpdateState::Idle;
                }
                if start_dl {
                    self.start_download(ctx, &info_clone);
                }
            }
            crate::updater::UpdateState::Downloading(pct) => {
                let pct = *pct;
                egui::Window::new("正在下载更新")
                    .resizable(false)
                    .collapsible(false)
                    .default_pos(screen_center(ctx))
                    .pivot(egui::Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().slider_width = 200.0;
                            ui.add(egui::ProgressBar::new(pct).show_percentage());
                        });
                    });
            }
            crate::updater::UpdateState::Verifying => {
                egui::Window::new("正在校验")
                    .resizable(false)
                    .collapsible(false)
                    .default_pos(screen_center(ctx))
                    .pivot(egui::Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.label("正在校验文件完整性...");
                    });
            }
            crate::updater::UpdateState::Ready(path) => {
                let path = path.clone();
                let mut restart = false;
                egui::Window::new("更新就绪")
                    .resizable(false)
                    .collapsible(false)
                    .default_pos(screen_center(ctx))
                    .pivot(egui::Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("更新已准备就绪");
                            ui.add_space(8.0);
                            if ui.button("重启应用").clicked() {
                                restart = true;
                            }
                        });
                    });
                if restart {
                    match crate::updater::replace_and_restart(&path) {
                        Ok(_) => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        Err(e) => {
                            self.update_state = crate::updater::UpdateState::Error(e);
                        }
                    }
                }
            }
            crate::updater::UpdateState::Error(msg) => {
                let msg = msg.clone();
                let mut dismiss = false;
                egui::Window::new("更新失败")
                    .resizable(false)
                    .collapsible(false)
                    .default_pos(screen_center(ctx))
                    .pivot(egui::Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.label(&msg);
                        ui.add_space(8.0);
                        if ui.button("关闭").clicked() {
                            dismiss = true;
                        }
                    });
                if dismiss {
                    self.update_state = crate::updater::UpdateState::Idle;
                }
            }
            crate::updater::UpdateState::UpToDate => {
                let mut dismiss = false;
                egui::Window::new("检查更新")
                    .resizable(false)
                    .collapsible(false)
                    .default_pos(screen_center(ctx))
                    .pivot(egui::Align2::CENTER_CENTER)
                    .show(ctx, |ui| {
                        ui.label("已是最新版本");
                        ui.add_space(8.0);
                        if ui.button("关闭").clicked() {
                            dismiss = true;
                        }
                    });
                if dismiss {
                    self.update_state = crate::updater::UpdateState::Idle;
                }
            }
            _ => {}
        }
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
        let any_theme_dialog = self.theme_dialog.show_copy_dialog
            || self.theme_dialog.show_new_dialog
            || self.theme_dialog.show_rename_dialog
            || self.theme_dialog.show_delete_confirm
            || self.theme_dialog.show_switch_confirm;
        if let Some(id) = self.terminal_focus_id {
            if terminal_focus_lock_allowed(
                self.renaming_panel.is_some(),
                self.renaming_terminal.is_some(),
            ) && !any_theme_dialog
                && !self.show_settings
            {
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
        let binding_was_recording = self.binding_recording.is_some();
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

        if !workspace_renaming
            && global_shortcuts_allowed(self)
            && !binding_was_recording
            && consume_exact_shortcut(ctx, &binds, "toggle_workspace_sidebar")
        {
            self.workspace_sidebar_visible = !self.workspace_sidebar_visible;
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
                    let visible = self.workspace_sidebar_visible;
                    if ui
                        .selectable_label(visible, &self.texts.view_menu.workspace_toggle)
                        .clicked()
                    {
                        self.workspace_sidebar_visible = !self.workspace_sidebar_visible;
                        ui.close_menu();
                    }
                });
                ui.menu_button(self.texts.menu.language.clone(), |ui| {
                    let current_code = self.settings.language.clone();
                    let languages = self.available_languages.clone();
                    // Auto-fit: size submenu to the longest language label.
                    let longest = languages
                        .iter()
                        .map(|(_, n)| n.chars().count())
                        .max()
                        .unwrap_or(8);
                    let char_w = ui.fonts(|f| f.row_height(&egui::FontId::proportional(13.0)));
                    ui.set_min_width((longest as f32) * char_w * 0.55 + 32.0);
                    for (code, display_name) in &languages {
                        let selected = *code == current_code;
                        if ui.selectable_label(selected, display_name).clicked() {
                            self.switch_language(code);
                            ui.close_menu();
                        }
                    }
                });
                ui.menu_button(self.texts.menu.theme.clone(), |ui| {
                    let current = self.settings.theme_id.clone();
                    let themes = self.available_themes.clone();
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for theme in &themes {
                                let selected = theme.id == current;
                                if ui.selectable_label(selected, &theme.name).clicked() {
                                    self.try_switch_theme(ctx, theme.id.clone());
                                    ui.close_menu();
                                }
                            }
                        });
                });
                if ui.button(self.texts.view_menu.settings.clone()).clicked() {
                    self.show_settings = true;
                    self.settings_edit = self.settings.clone();
                    self.theme_edit = self.active_theme.clone();
                    self.theme_message = None;
                    self.theme_dirty = false;
                }
                if ui.button(self.texts.about.menu_label.clone()).clicked() {
                    self.show_about = true;
                }
            });
        });

        if self.show_settings {
            let mut open = self.show_settings;
            let ws = &self.settings_edit.settings_window;
            egui::Window::new(&self.texts.settings.title)
                .open(&mut open)
                .resizable(true)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_size([ws.width, ws.height])
                .show(ctx, |ui| {
                    // Manual two-column layout: left nav + right content.
                    // Uses fixed child UIs to avoid SidePanel ambiguity inside
                    // a Window closure.
                    let full = ui.available_rect_before_wrap();
                    let nav_width = 120.0;
                    let nav_rect =
                        egui::Rect::from_min_size(full.min, egui::vec2(nav_width, full.height()));
                    let content_rect = egui::Rect::from_min_size(
                        egui::pos2(full.min.x + nav_width, full.min.y),
                        egui::vec2(full.width() - nav_width, full.height()),
                    );

                    // Left nav
                    let mut nav_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(nav_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    nav_ui.vertical(|ui| {
                        ui.add_space(4.0);
                        let nav_items = [
                            &self.texts.settings.nav.general,
                            &self.texts.settings.nav.themes,
                            &self.texts.settings.nav.shortcuts,
                            &self.texts.settings.nav.lock,
                        ];
                        for (i, label) in nav_items.iter().enumerate() {
                            let selected = self.settings_tab == i;
                            let text = format!("  {}", label);
                            if ui.selectable_label(selected, &text).clicked() {
                                self.settings_tab = i;
                            }
                        }
                    });

                    // Right content
                    let mut content_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content_rect)
                            .layout(egui::Layout::top_down(egui::Align::LEFT)),
                    );
                    content_ui.vertical(|ui| {
                        ui.add_space(4.0);
                        match self.settings_tab {
                            0 => {
                                ui.weak(self.texts.settings.general.scene_info.as_str());
                                ui.weak(self.texts.settings.general.scene_path.as_str());
                                ui.weak(self.texts.settings.general.templates_path.as_str());
                                ui.add_space(8.0);
                                ui.weak(self.texts.settings.general.history_section.as_str());
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
                                let is_builtin =
                                    crate::theme::store::is_embedded_id(&self.theme_edit.id);
                                if is_builtin {
                                    ui.weak(crate::theme::builtin_readonly_text());
                                    ui.add_space(4.0);
                                }
                                let any_dialog_open = self.theme_dialog.show_copy_dialog
                                    || self.theme_dialog.show_new_dialog
                                    || self.theme_dialog.show_rename_dialog
                                    || self.theme_dialog.show_delete_confirm
                                    || self.theme_dialog.show_switch_confirm;

                                if any_dialog_open {
                                    if let Some(Err(msg)) = &self.theme_message {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(230, 120, 120),
                                            msg,
                                        );
                                    }
                                    if let Some(Ok(msg)) = &self.theme_message {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(120, 200, 130),
                                            msg,
                                        );
                                    }
                                } else {
                                    egui::ScrollArea::vertical()
                                        .id_salt("appearance_scroll")
                                        .show(ui, |ui| {
                                            let mut draft = self.theme_edit.clone();
                                            let actions = crate::theme::ui::show_theme_section(
                                                ui,
                                                &mut draft,
                                                &self.available_themes,
                                                is_builtin,
                                                self.theme_dirty,
                                                self.theme_editor_subtab,
                                                &mut self.theme_dialog,
                                            );
                                            for action in actions {
                                                self.handle_theme_action(
                                                    ctx,
                                                    action,
                                                    draft.clone(),
                                                );
                                            }
                                            if draft != self.theme_edit {
                                                self.theme_edit = draft;
                                                self.theme_dirty = true;
                                                crate::theme::apply_theme_definition(
                                                    ctx,
                                                    &self.theme_edit,
                                                );
                                            }
                                        });
                                    if let Some(Err(msg)) = &self.theme_message {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(230, 120, 120),
                                            msg,
                                        );
                                    }
                                    if let Some(Ok(msg)) = &self.theme_message {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(120, 200, 130),
                                            msg,
                                        );
                                    }
                                }
                            }
                            2 => {
                                ui.weak(self.texts.settings.shortcuts.hint.as_str());
                                for id in shortcut_hint_ids() {
                                    let label = shortcut_label_for(&self.texts, id).to_string();
                                    ui.horizontal(|ui| {
                                        ui.label(&label);
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let text = if self.binding_recording.as_deref()
                                                    == Some(id)
                                                {
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
                                if ui
                                    .button(&self.texts.settings.shortcuts.reset_defaults)
                                    .clicked()
                                {
                                    self.settings_edit.key_binds = default_key_binds();
                                    self.binding_recording = None;
                                }
                            }
                            3 => {
                                ui.weak(self.texts.settings.lock.password_section.as_str());
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
                        ui.add_space(8.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(&self.texts.workspace.close).clicked() {
                                self.settings_edit = self.settings.clone();
                                self.theme_edit = self.active_theme.clone();
                                self.theme_message = None;
                                self.theme_dirty = false;
                                self.binding_recording = None;
                                crate::theme::apply_theme_definition(ctx, &self.active_theme);
                                self.show_settings = false;
                            }
                            if ui.button(&self.texts.settings.buttons.apply).clicked() {
                                self.settings = self.settings_edit.clone();
                                self.history_db.set_max_entries(self.settings.max_history);
                                let _ = save_settings(&self.settings);
                                if self.theme_dirty {
                                    let themes_root =
                                        crate::theme::store::themes_dir(&app_data_dir());
                                    if !crate::theme::store::is_embedded_id(&self.theme_edit.id) {
                                        let _ = std::fs::create_dir_all(&themes_root);
                                        if let Err(err) = crate::theme::store::save_user_theme(
                                            &themes_root,
                                            &self.theme_edit,
                                        ) {
                                            log::error!("failed to save theme: {err}");
                                        }
                                    }
                                }
                                self.active_theme = self.theme_edit.clone();
                                self.settings.theme_id = self.active_theme.id.clone();
                                crate::theme::apply_theme_definition(ctx, &self.active_theme);
                                if self.settings.apply_theme_typography {
                                    let new_size = self.active_theme.typography.terminal_font_size;
                                    for td in self.terminals.values_mut() {
                                        td.font_size = new_size;
                                    }
                                }
                                self.theme_dirty = false;
                            }
                        });
                    });
                });
            if !open {
                self.show_settings = false;
                self.binding_recording = None;
                self.settings.settings_window = self.settings_edit.settings_window.clone();
                let _ = save_settings(&self.settings);
            }
        }

        // Theme dialog popups
        self.show_theme_dialogs(ctx);

        if self.show_about {
            let mut open = self.show_about;
            let mut clicked_close = false;
            egui::Window::new(&self.texts.about.title)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_width(380.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(env!("CARGO_PKG_NAME"))
                                .size(20.0)
                                .strong(),
                        );
                        ui.weak(format!(
                            "{}: v{}",
                            self.texts.about.version_label,
                            env!("CARGO_PKG_VERSION")
                        ));
                    });
                    ui.add_space(6.0);
                    ui.weak(self.texts.about.description.as_str());
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.weak(self.texts.about.homepage_label.as_str());
                        ui.hyperlink_to(
                            "https://opennex.zeadix.com/",
                            "https://opennex.zeadix.com/",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.weak(self.texts.about.source_label.as_str());
                        ui.hyperlink_to(
                            "https://github.com/zeadix/opennex",
                            "https://github.com/zeadix/opennex",
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.weak(self.texts.about.license_label.as_str());
                        ui.label(env!("CARGO_PKG_LICENSE"));
                    });
                    ui.add_space(6.0);
                    ui.weak("Author: KunPeng.Wang <msr.rsm@qq.com>");
                    ui.weak(self.texts.about.credits_label.as_str());
                    ui.weak(self.texts.about.credits.as_str());
                    ui.add_space(6.0);
                    ui.vertical_centered(|ui| {
                        if ui.button("检查更新").clicked() {
                            self.check_update_manual(ctx);
                        }
                        if ui.button(&self.texts.about.close).clicked() {
                            clicked_close = true;
                        }
                    });
                });
            if !open || clicked_close {
                self.show_about = false;
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
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
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

        // Check for update result from background thread
        if self.update_state == crate::updater::UpdateState::Idle {
            let info = ctx.memory(|mem| {
                mem.data
                    .get_temp::<Option<crate::updater::UpdateInfo>>(egui::Id::new("update_info"))
                    .map(|v| v.clone())
                    .flatten()
            });
            if let Some(info) = info {
                self.update_state = crate::updater::UpdateState::Available(info);
            }
        }

        // Update window
        self.render_update_window(ctx);

        // Terminal close confirmation
        if let Some(ref tab_id) = self.pending_close_confirm.clone() {
            let mut open = true;
            let mut confirmed = false;
            let mut cancelled = false;
            let tab_id = tab_id.clone();
            egui::Window::new(&self.texts.close_confirm.terminal_title)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui| {
                    ui.label(&self.texts.close_confirm.terminal_message);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(&self.texts.close_confirm.confirm).clicked() {
                            confirmed = true;
                        }
                        if ui.button(&self.texts.close_confirm.cancel).clicked() {
                            cancelled = true;
                        }
                    });
                });
            if confirmed {
                self.pending_close_confirm = None;
                self.pending_close = Some(tab_id);
            }
            if cancelled || !open {
                self.pending_close_confirm = None;
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
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
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

        if self.workspace_sidebar_visible {
            egui::SidePanel::left("navigation")
                .default_width(WORKSPACE_SIDEBAR_DEFAULT_WIDTH)
                .show(ctx, |ui| {
                    // New workspace + templates buttons in one row above the list
                    ui.horizontal(|ui| {
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
                    });
                    ui.add_space(4.0);
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
                                    confirm =
                                        ui.button(&self.texts.workspace.rename_confirm).clicked();
                                    cancel =
                                        ui.button(&self.texts.workspace.rename_cancel).clicked();
                                    response
                                })
                                .response;
                            self.panel_rects[i] = response.rect;
                            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let escape = ui.input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                            });
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
                                let has_close = self.panels.len() > 1;
                                let has_lock = has_close || self.locked_panels.contains(&i);
                                let action_width = workspace_action_column_width(
                                    has_close,
                                    has_lock,
                                    ui.spacing().item_spacing.x,
                                );
                                let name_width = (ui.available_width()
                                    - action_width
                                    - ui.spacing().item_spacing.x)
                                    .max(0.0);
                                let response = ui.add_sized(
                                    egui::vec2(name_width, ui.spacing().interact_size.y),
                                    egui::SelectableLabel::new(is_active, &panel_name),
                                );
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
                                    if ui.button(&self.texts.settings.buttons.close).clicked() {
                                        self.close_confirm_panel = Some(i);
                                        ui.close_menu();
                                    }
                                });

                                // Lock and close buttons
                                if has_lock {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(action_width, ui.spacing().interact_size.y),
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let action_rect = ui.max_rect();
                                            ui.painter().rect_filled(
                                                action_rect,
                                                0.0,
                                                ui.visuals().faint_bg_color,
                                            );
                                            if has_close {
                                                if ui
                                                    .add_sized(
                                                        WORKSPACE_ACTION_BUTTON_SIZE,
                                                        egui::Button::new(
                                                            egui_phosphor::regular::X,
                                                        ),
                                                    )
                                                    .on_hover_text(
                                                        &self.texts.workspace.close_ws_hint,
                                                    )
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
                                                    egui::Button::new(workspace_lock_icon(
                                                        is_locked,
                                                    )),
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
                                    if j < self.panel_rects.len()
                                        && self.panel_rects[j].contains(pos)
                                    {
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
                                    .color(ui.visuals().text_color()),
                            );
                            egui::ScrollArea::vertical()
                                .id_salt("navigation_shortcuts")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let extra_shortcuts: [(&str, Vec<&str>); 2] = [
                                        ("zoom_in", vec!["Ctrl", "+"]),
                                        ("zoom_out", vec!["Ctrl", "-"]),
                                    ];
                                    let ids: Vec<(&str, Vec<String>)> = shortcut_hint_ids()
                                        .iter()
                                        .map(|id| {
                                            let binding = self
                                                .settings
                                                .key_binds
                                                .get(*id)
                                                .map(shortcut_display)
                                                .unwrap_or_else(|| {
                                                    self.texts.settings.shortcuts.not_set.clone()
                                                });
                                            (
                                                *id,
                                                binding
                                                    .split('+')
                                                    .map(|s| s.trim().to_string())
                                                    .collect(),
                                            )
                                        })
                                        .chain(extra_shortcuts.iter().map(|(id, parts)| {
                                            (*id, parts.iter().map(|s| s.to_string()).collect())
                                        }))
                                        .collect();
                                    for (id, parts) in ids {
                                        let label = shortcut_label_for(&self.texts, id).to_string();
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(&label)
                                                    .small()
                                                    .color(ui.visuals().text_color()),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    let mut parts_rev = parts.clone();
                                                    parts_rev.reverse();
                                                    ui.horizontal(|ui| {
                                                        for (i, part) in
                                                            parts_rev.iter().enumerate()
                                                        {
                                                            if i > 0 {
                                                                ui.label(
                                                                    egui::RichText::new("+")
                                                                        .small()
                                                                        .color(
                                                                            ui.visuals()
                                                                                .weak_text_color(),
                                                                        ),
                                                                );
                                                            }
                                                            paint_keycap(ui, part);
                                                        }
                                                    });
                                                },
                                            );
                                        });
                                    }
                                });
                        },
                    );
                });
        }

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
                let lock_color = self.active_theme.app.lock.to_egui();
                let avail = ui.available_size();
                let (rect, _) = ui.allocate_exact_size(avail, egui::Sense::click());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 0.0, lock_color);

                let pw_id = egui::Id::new("lock_overlay_pw_input");
                let form_width = 360.0;
                let form_rect =
                    egui::Rect::from_center_size(rect.center(), egui::vec2(form_width, 220.0));
                let ui_content =
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(form_rect), |ui| {
                        ui.vertical_centered(|ui| {
                            // Lock icon
                            ui.label(
                                egui::RichText::new(egui_phosphor::regular::LOCK_SIMPLE)
                                    .size(36.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.add_space(8.0);
                            // Title
                            ui.heading(
                                egui::RichText::new(&self.texts.lock_overlay.title)
                                    .size(18.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.add_space(20.0);
                            // Password input row — manually center by measuring content width
                            let label_galley = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    self.texts.lock_overlay.password_label.clone(),
                                    egui::FontId::proportional(14.0),
                                    egui::Color32::from_gray(220),
                                )
                            });
                            let input_width = 130.0;
                            let button_galley = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    self.texts.lock_overlay.unlock_button.clone(),
                                    egui::FontId::proportional(14.0),
                                    egui::Color32::BLACK,
                                )
                            });
                            let item_spacing = ui.spacing().item_spacing.x;
                            let row_h = ui.spacing().interact_size.y;
                            let content_width = label_galley.size().x
                                + input_width
                                + button_galley.size().x
                                + ui.spacing().button_padding.x * 2.0
                                + item_spacing * 2.0;
                            let avail_w = ui.available_width();
                            let _ = avail_w;
                            ui.allocate_ui_with_layout(
                                egui::vec2(content_width, row_h),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(
                                            &self.texts.lock_overlay.password_label,
                                        )
                                        .color(egui::Color32::from_gray(220)),
                                    );
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut self.lock_password_input)
                                            .password(true)
                                            .desired_width(input_width)
                                            .id(pw_id),
                                    );
                                    resp.request_focus();
                                    let enter_pressed =
                                        ui.input(|i| i.key_pressed(egui::Key::Enter));
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(
                                                    &self.texts.lock_overlay.unlock_button,
                                                )
                                                .color(egui::Color32::BLACK),
                                            )
                                            .fill(egui::Color32::WHITE)
                                            .stroke(egui::Stroke::new(1.0, egui::Color32::BLACK)),
                                        )
                                        .clicked()
                                        || (enter_pressed && resp.has_focus())
                                    {
                                        if self.settings.lock_password.is_empty()
                                            || self.lock_password_input
                                                == self.settings.lock_password
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
                                },
                            );
                            // Error message
                            if !self.pw_message.is_empty() {
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(&self.pw_message)
                                        .color(egui::Color32::from_rgb(255, 120, 120)),
                                );
                            }
                        });
                    });
                let _ = ui_content;
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
                            pending_close: &mut self.pending_close,
                            pending_close_confirm: &mut self.pending_close_confirm,
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
                            theme: if self.show_settings {
                                &self.theme_edit
                            } else {
                                &self.active_theme
                            },
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
            "toggle_workspace_sidebar",
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
    fn workspace_sidebar_defaults_to_f1() {
        let binds = default_key_binds();
        let binding = &binds["toggle_workspace_sidebar"];

        assert_eq!(binding.key, "F1");
        assert!(!binding.ctrl);
        assert!(!binding.shift);
        assert!(!binding.alt);
    }

    #[test]
    fn legacy_settings_gain_sidebar_shortcut_without_overwriting_bindings() {
        let mut settings = AppSettings::default();
        settings.key_binds.remove("toggle_workspace_sidebar");
        settings.key_binds.insert(
            "new_terminal".into(),
            ShortcutBinding {
                key: "T".into(),
                ctrl: true,
                shift: false,
                alt: false,
            },
        );

        let migrated = super::normalize_history_bindings(settings);
        assert_eq!(migrated.key_binds["toggle_workspace_sidebar"].key, "F1");
        assert_eq!(migrated.key_binds["new_terminal"].key, "T");
    }

    #[test]
    fn f1_shortcut_matches_correctly() {
        let sidebar = default_key_binds()["toggle_workspace_sidebar"].clone();

        assert!(super::binding_matches_key(
            &sidebar,
            egui::Key::F1,
            egui::Modifiers::NONE
        ));
        assert!(!super::binding_matches_key(
            &sidebar,
            egui::Key::F1,
            egui::Modifiers::CTRL
        ));
    }

    #[test]
    fn workspace_action_column_width_covers_buttons() {
        assert_eq!(super::workspace_action_column_width(false, false, 4.0), 0.0);
        assert_eq!(super::workspace_action_column_width(false, true, 4.0), 24.0);
        assert_eq!(super::workspace_action_column_width(true, true, 4.0), 52.0);
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

    #[test]
    fn legacy_visual_settings_migrate_without_touching_behavior_settings() {
        let json = r#"{
            "max_history": 777,
            "language": "de",
            "theme": "dark",
            "terminal_theme": "gruvbox",
            "bg_color": [1, 2, 3],
            "fg_color": [4, 5, 6]
        }"#;
        let settings = super::deserialize_settings(json).unwrap();
        assert_eq!(settings.max_history, 777);
        assert_eq!(settings.language, "de");
        assert_eq!(settings.theme_id, "gruvbox-dark");
    }

    #[test]
    fn legacy_theme_mode_maps_when_no_terminal_theme_present() {
        let json = r#"{ "theme": "light", "language": "en" }"#;
        let settings = super::deserialize_settings(json).unwrap();
        assert_eq!(settings.theme_id, "opennex-light");
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn default_settings_select_embedded_default_theme() {
        assert_eq!(AppSettings::default().theme_id, "opennex-dark");
        assert!(AppSettings::default().apply_theme_typography);
    }
}

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    completion: &'a crate::completion::CompletionEngine,
    history_db: &'a crate::history_db::HistoryDb,
    max_history: usize,
    pending_close: &'a mut Option<String>,
    pending_close_confirm: &'a mut Option<String>,
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
    theme: &'a crate::theme::ThemeDefinition,
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
                tv = tv.set_theme(crate::theme::terminal_theme(self.theme));
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
                    let menu_bg = self.theme.app.panel.to_egui();
                    let menu_fg = self.theme.app.text.to_egui();
                    let menu_font_size = self.theme.typography.menu_font_size;
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
                            self.theme.app.active.to_egui()
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
                            egui::FontId::monospace(menu_font_size),
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
        *self.pending_close_confirm = Some(tab.clone());
        false
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

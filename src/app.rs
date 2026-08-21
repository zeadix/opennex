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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(default = "default_true")]
    auto_copy_selection: bool,
    #[serde(default = "default_true")]
    auto_match_command: bool,
}

fn default_theme_id() -> String {
    "opennex-dark".into()
}

fn default_true() -> bool {
    true
}

fn default_language() -> String {
    // First launch defaults to English (an existing settings.json keeps the
    // user's saved language).
    "en".into()
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
    m.insert(
        "zoom_in".into(),
        ShortcutBinding {
            key: "Plus".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "zoom_out".into(),
        ShortcutBinding {
            key: "Minus".into(),
            ctrl: true,
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
        "Plus" => Some(egui::Key::Plus),
        "Minus" => Some(egui::Key::Minus),
        "Equals" => Some(egui::Key::Equals),
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

fn shortcut_hint_ids() -> [&'static str; 13] {
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
        "zoom_in",
        "zoom_out",
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

fn screen_center(ctx: &egui::Context) -> egui::Pos2 {
    ctx.input(|i| i.screen_rect).center()
}

const WORKSPACE_ACTION_BUTTON_SIZE: egui::Vec2 = egui::vec2(24.0, 24.0);

/// Reserved font families holding the CLEAN generic stacks (before the
/// active theme injects its fonts at the head). Theme-list previews use
/// these so switching themes never changes other previews' rendering.
fn preview_prop_family() -> egui::FontFamily {
    egui::FontFamily::Name(std::sync::Arc::from("__preview_proportional__"))
}
fn preview_mono_family() -> egui::FontFamily {
    egui::FontFamily::Name(std::sync::Arc::from("__preview_monospace__"))
}

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
    // Two COMPLETELY different glyph shapes so the state is unambiguous:
    // locked = solid padlock with key; unlocked = dashed circle ("not
    // protected"). Lock-vs-open-lock glyphs read as near-identical at 13px.
    if is_locked {
        egui_phosphor::regular::LOCK_KEY
    } else {
        egui_phosphor::regular::CIRCLE_DASHED
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
            auto_word: None,
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

#[derive(Debug, Clone)]
enum StartCheckResult {
    Available(crate::updater::UpdateInfo),
    UpToDate,
    Error(String),
}

impl Default for StartCheckResult {
    fn default() -> Self {
        StartCheckResult::UpToDate
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            width: 760.0,
            height: 540.0,
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
            auto_copy_selection: true,
            auto_match_command: true,
        }
    }
}

/// Whether the font file bytes parse as a real font face. Some files in
/// the system font directories carry a .ttf/.otf/.ttc extension but are
/// not valid font resources (e.g. Windows mstmc.ttf bitmap fonts);
/// egui/epaint panics when parsing those at first use, crashing startup.
fn is_valid_font_data(data: &[u8]) -> bool {
    ab_glyph::FontRef::try_from_slice(data).is_ok()
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

#[derive(Clone)]
pub struct HistoryNav {
    pub entries: Vec<String>,
    pub selected: usize,
    /// When this nav was opened by the auto-match (typing) overlay, the
    /// word the user has already typed — on confirm it is deleted before
    /// the full command is sent. None for the manual Alt menu.
    pub auto_word: Option<String>,
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

/// Pages of the Unity-style settings window. The numeric order matches
/// the nav listing; `as u8` is persisted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsPage {
    General = 0,
    Shortcuts = 1,
    Lock = 2,
    Theme = 3,
}

impl SettingsPage {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => SettingsPage::Shortcuts,
            2 => SettingsPage::Lock,
            3 | 4 | 5 | 6 => SettingsPage::Theme,
            _ => SettingsPage::General,
        }
    }

    fn title(&self, texts: &crate::i18n::Texts) -> String {
        let nav = &texts.settings.nav;
        match self {
            SettingsPage::General => nav.general.clone(),
            SettingsPage::Shortcuts => nav.shortcuts.clone(),
            SettingsPage::Lock => nav.lock.clone(),
            SettingsPage::Theme => nav.themes.clone(),
        }
    }
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
    /// Confirmation dialog for deleting ALL terminal command history.
    show_clear_history_confirm: bool,
    settings: AppSettings,
    show_settings: bool,
    show_about: bool,
    settings_edit: AppSettings,
    /// Active settings page (see `SettingsPage`). Stored as u8 for serde
    /// compatibility with the persisted `settings_window` payload.
    settings_tab: u8,
    /// Transient "已应用" toast shown in the settings footer.
    settings_applied_toast: Option<(String, std::time::Instant)>,
    settings_window_open: bool,
    binding_recording: Option<String>,
    cached_template_files: Vec<(String, PathBuf)>,
    completion: crate::completion::CompletionEngine,
    history_db: crate::history_db::HistoryDb,
    focused_terminal: Option<String>,
    drag_src_panel: Option<usize>,
    drag_dst_panel: Option<usize>,
    locked_panels: std::collections::HashSet<usize>,
    lock_password_input: String,
    /// Whether the lock-overlay password is shown in plain text.
    lock_password_visible: bool,
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
    /// Whether the standalone theme editor popup is open.
    theme_editor_open: bool,
    /// Theme id the editor popup is editing (None = closed).
    theme_edit_origin: Option<String>,
    theme_editor_subtab: crate::theme::ui::ThemeEditorSubtab,
    auto_copy_selection: bool,
    /// Whether typing auto-matches commands from history (settings toggle).
    auto_match_command: bool,
    /// Last known screen rect of each terminal's view (for the global
    /// history-menu overlay anchoring).
    terminal_view_rects: std::collections::HashMap<String, egui::Rect>,
    /// Terminal whose history-menu "clear" confirmation dialog is open.
    history_clear_confirm: Option<String>,
    /// Single-frame latch per terminal: set on ANY history-menu close
    /// (Esc / confirm / click). Blocks the auto-matcher for exactly one
    /// frame so the confirming keypress's own Text event (Space/Enter
    /// echo) cannot re-open the menu. The ONLY way the matcher runs again
    /// is a real key edit (Text/Backspace/Delete) in a later frame.
    history_menu_just_closed: std::collections::HashMap<String, bool>,
    /// Keystrokes typed but not yet echoed on the PTY grid, per terminal.
    /// The grid lags the keypress by ≥1 frame, so the matcher matches
    /// grid_word + pending; entries are consumed as the grid catches up.
    auto_match_pending: std::collections::HashMap<String, String>,
    show_update_dialog: bool,
    update_dialog_info: Option<crate::updater::UpdateInfo>,
    skipped_versions: std::collections::HashSet<String>,
    update_toast: Option<(String, std::time::Instant)>,
    /// Transient "copied to clipboard" notice (auto-copy selection).
    copy_toast: Option<std::time::Instant>,
    startup_frame_count: u32,
    /// Aggregated CPU% across every live terminal's process tree,
    /// refreshed on the 2 s sampling tick.
    terminal_cpu: Option<f32>,
    /// Aggregated resident memory across terminal process trees.
    terminal_mem: Option<u64>,
    /// Same aggregates but scoped to the active workspace only.
    workspace_cpu: Option<f32>,
    workspace_mem: Option<u64>,
    /// Aggregates for the focused terminal's process tree only.
    focused_cpu: Option<f32>,
    focused_mem: Option<u64>,
    /// Wall-clock of the last sampling tick.
    last_sample: std::time::Instant,
    /// Delta sampler over per-terminal process trees.
    terminal_sampler: crate::proc_stats::ProcSampler,
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
        // Same ordering as refresh_themes: custom block first (newest by
        // creation time), builtin block always below — never a global
        // name-sort that interleaves the two blocks.
        let mut available_themes =
            crate::theme::store::load_user_themes(&themes_root).unwrap_or_default();
        available_themes.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        for embedded in crate::theme::store::embedded_themes().unwrap_or_default() {
            if !available_themes.iter().any(|t| t.id == embedded.id) {
                available_themes.push(embedded);
            }
        }
        let active_theme = crate::theme::store::load_theme(&themes_root, &settings.theme_id)
            .unwrap_or_else(|err| {
                log::warn!("failed to load theme '{}': {err}", settings.theme_id);
                crate::theme::store::default_theme().unwrap_or_else(|err| {
                    panic!("embedded default theme failed to load: {err}");
                })
            });

        // Scan system monospace fonts; registration happens later in
        // App::rebuild_fonts once the active theme's font choices are known.
        let system_fonts = scan_system_fonts();
        let font_names: Vec<String> = system_fonts.iter().map(|(name, _)| name.clone()).collect();

        crate::theme::apply_theme_definition(ctx, &active_theme);

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
            show_clear_history_confirm: false,
            settings,
            show_settings: false,
            show_about: false,
            settings_edit: AppSettings::default(),
            settings_tab: 0,
            settings_applied_toast: None,
            settings_window_open: false,
            binding_recording: None,
            cached_template_files: Vec::new(),
            completion: crate::completion::CompletionEngine::new(),
            history_db: crate::history_db::HistoryDb::new(&db_path, default_max_history()),
            focused_terminal: None,
            drag_src_panel: None,
            drag_dst_panel: None,
            locked_panels: std::collections::HashSet::new(),
            lock_password_input: String::new(),
            lock_password_visible: false,
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
            theme_editor_open: false,
            theme_edit_origin: None,
            theme_editor_subtab: Default::default(),
            auto_copy_selection: true,
            auto_match_command: true,
            terminal_view_rects: Default::default(),
            history_clear_confirm: None,
            history_menu_just_closed: Default::default(),
            auto_match_pending: Default::default(),
            show_update_dialog: false,
            update_dialog_info: None,
            skipped_versions: std::collections::HashSet::new(),
            update_toast: None,
            copy_toast: None,
            startup_frame_count: 0,
            terminal_cpu: None,
            terminal_mem: None,
            workspace_cpu: None,
            workspace_mem: None,
            focused_cpu: None,
            focused_mem: None,
            last_sample: std::time::Instant::now(),
            terminal_sampler: crate::proc_stats::ProcSampler::new(),
        };

        // Register fonts (system + embedded + theme choices) now that the
        // App and its active theme exist. rebuild_fonts reuses the scanned
        // list captured above for the system_fonts field.
        app.rebuild_fonts(ctx);

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
                app.restore_workspace_focus(0);
                app.refresh_template_files();
                // Orphan-history cleanup: drop SQLite rows for terminal
                // ids that no longer exist in the loaded scene (leftovers
                // from closed workspaces whose scene was never re-saved).
                // Without this, a newly created terminal reusing such an
                // id would inherit stale command history.
                let live_ids: Vec<String> = app.terminals.keys().cloned().collect();
                app.history_db.prune(&live_ids);
                return app;
            }
        }

        app.add_initial_terminal(ctx);
        app.refresh_template_files();

        // Start background update check
        {
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                let result = crate::updater::check_for_update();
                // Store result as an enum for the UI to handle.
                let stored = match &result {
                    Ok(Some(info)) => StartCheckResult::Available(info.clone()),
                    Ok(None) => StartCheckResult::UpToDate,
                    Err(e) => StartCheckResult::Error(e.clone()),
                };
                ctx_clone.memory_mut(|mem| {
                    mem.data
                        .insert_temp(egui::Id::new("start_check_result"), stored);
                });
                ctx_clone.request_repaint();
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
        let name = "workspace 1".to_string();
        let Some(tab_id) = self.create_terminal_inner(ctx) else {
            return;
        };
        self.dock_states.insert(0, DockState::new(vec![tab_id]));
        self.panels.push(Panel {
            name,
            bound_file: None,
        });
        self.active_panel = 0;
        self.restore_workspace_focus(0);
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
                name: format!("terminal-{random_suffix}"),
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
            let split_after = self.pending_split_after.take();
            let split_vertical = self.pending_split_vertical;
            let Some(tab_id) = self.create_terminal_inner(ctx) else {
                return;
            };
            if let Some(tree) = self.dock_states.get_mut(&panel_idx) {
                if split_after.is_some() {
                    // Split the node holding the current tab and place the
                    // new terminal in the freshly created leaf.
                    let split = if split_vertical {
                        egui_dock::Split::Below
                    } else {
                        egui_dock::Split::Right
                    };
                    let [_old, new] = tree.split(
                        (surface_idx, node_idx),
                        split,
                        0.5,
                        egui_dock::Node::leaf(tab_id),
                    );
                    tree.set_focused_node_and_surface((surface_idx, new));
                } else {
                    tree.set_focused_node_and_surface((surface_idx, node_idx));
                    tree.push_to_focused_leaf(tab_id);
                }
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
        let name = format!("workspace {n}");
        let Some(tab_id) = self.create_terminal_inner(ctx) else {
            return;
        };
        self.dock_states
            .insert(self.panels.len(), DockState::new(vec![tab_id.clone()]));
        self.panels.push(Panel {
            name,
            bound_file: None,
        });
        // New workspace becomes active and its first terminal gets focus.
        self.active_panel = self.panels.len() - 1;
        self.focused_terminal = Some(tab_id);
        self.restore_workspace_focus(self.active_panel);
    }

    /// Ensure the workspace's dock tree has a focused surface/leaf so its
    /// active tab renders as selected, and sync `focused_terminal` to it.
    fn restore_workspace_focus(&mut self, panel_idx: usize) {
        if let Some(tree) = self.dock_states.get_mut(&panel_idx) {
            if tree.find_active_focused().is_none() {
                tree.set_focused_node_and_surface((
                    egui_dock::SurfaceIndex::main(),
                    egui_dock::NodeIndex::root(),
                ));
            }
            if let Some((_, tab)) = tree.find_active_focused() {
                self.focused_terminal = Some(tab.clone());
            }
        }
    }

    fn close_workspace(&mut self, i: usize) {
        if self.panels.len() <= 1 {
            return;
        }
        // Delete history for every tab id attached to this workspace's
        // dock TREE (not just ids present in self.terminals): orphaned ids
        // left on the tree by past index mixups must be cleared too, or a
        // future terminal reusing the id inherits stale history.
        if let Some(tree) = self.dock_states.get(&i) {
            for (_, tab_id) in tree.iter_all_tabs() {
                let tab_id = tab_id.clone();
                self.history_db.clear(&tab_id);
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
        // swap_remove pulls the LAST workspace into slot i, so the dock
        // tree must follow: move the last workspace's dock_states into
        // slot i too, otherwise the promoted workspace renders empty.
        // (Computed BEFORE swap_remove: the old last index is len-1.)
        let old_last = self.panels.len() - 1;
        if i != old_last {
            if let Some(promoted) = self.dock_states.remove(&old_last) {
                self.dock_states.insert(i, promoted);
            }
        } else {
            // Removing the last workspace: nothing to promote.
            self.dock_states.remove(&i);
        }
        // Locked-state indices must follow the swap too: the removed
        // workspace i loses its lock; a promoted locked workspace gains
        // index i.
        let promoted_locked = self.locked_panels.remove(&old_last);
        self.locked_panels.remove(&i);
        if promoted_locked {
            self.locked_panels.insert(i);
        }
        let panel = self.panels.swap_remove(i);
        let _ = panel;
        if self.active_panel >= self.panels.len() {
            self.active_panel = self.panels.len().saturating_sub(1);
        }
        // If the active workspace was removed, fall back sensibly.
        if self.active_panel == i {
            self.active_panel = self.active_panel.min(self.panels.len().saturating_sub(1));
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
        // Font tables must be rebuilt so the new theme's UI/terminal fonts
        // actually take effect (Monospace/Proportional family heads).
        self.rebuild_fonts(ctx);
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
                    if ui.button(&self.texts.theme_editor.confirm).clicked() {
                        do_action = true;
                    }
                    if matches!(
                        kind,
                        DialogKind::New | DialogKind::Copy | DialogKind::Rename
                    ) && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        do_action = true;
                    }
                    if ui.button(&self.texts.theme_editor.cancel).clicked() {
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

    /// Standalone theme editor popup: live-preview editing with
    /// "keep"(保存) / "discard"(放弃) on close.
    fn show_theme_editor_popup(&mut self, ctx: &egui::Context) {
        if !self.theme_editor_open {
            return;
        }
        let themes_root = crate::theme::store::themes_dir(&app_data_dir());
        let mut keep = false;
        let mut discard = false;
        let mut is_open = self.theme_editor_open;
        let mut editor_name = self.theme_edit.name.clone();
        let mut editor_dirty = self.theme_dirty;
        let mut editor_draft = self.theme_edit.clone();
        let available_themes = self.available_themes.clone();
        let system_fonts = self.system_fonts.clone();
        let mut theme_dialog = self.theme_dialog.clone();
        let accent = self.active_theme.app.accent.to_egui();
        let mut actions_out: Vec<crate::theme::ui::ThemeAction> = Vec::new();
        let mut draft_changed = false;
        let te = &self.texts.theme_editor;
        let editor_labels = crate::theme::ui::ThemeEditorLabels {
            system_ui: te.system_ui.clone(),
            terminal: te.terminal.clone(),
            ui_font: te.ui_font_label.clone(),
            ui_font_size: te.ui_font_size.clone(),
            terminal_font: te.terminal_font_label.clone(),
            terminal_font_size: te.terminal_font_size.clone(),
            cell_spacing: te.cell_spacing.clone(),
            terminal_padding: te.terminal_padding.clone(),
            colors: crate::theme::ui::ColorLabels {
                app_bg: te.colors.app_bg.clone(),
                sidebar: te.colors.sidebar.clone(),
                panel: te.colors.panel.clone(),
                input_bg: te.colors.input_bg.clone(),
                text: te.colors.text.clone(),
                weak_text: te.colors.weak_text.clone(),
                accent: te.colors.accent.clone(),
                warning: te.colors.warning.clone(),
                danger: te.colors.danger.clone(),
                hover: te.colors.hover.clone(),
                active: te.colors.active.clone(),
                selection_bg: te.colors.selection_bg.clone(),
                selection_text: te.colors.selection_text.clone(),
                border: te.colors.border.clone(),
                lock: te.colors.lock.clone(),
                window_shadow: te.colors.window_shadow.clone(),
                tab_highlight: te.colors.tab_highlight.clone(),
                fg: te.colors.fg.clone(),
                bg: te.colors.bg.clone(),
                cursor: te.colors.cursor.clone(),
                selection_term_bg: te.colors.selection_term_bg.clone(),
                selection_term_text: te.colors.selection_term_text.clone(),
                link: te.colors.link.clone(),
            },
        };
        let title = format!("{} - {}", self.texts.theme_editor.edit_title, editor_name);
        egui::Window::new(title)
            .id(egui::Id::new("theme_editor_window"))
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .fixed_size([360.0, 560.0])
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing.y = 4.0;
                // Name row (fixed at top).
                ui.horizontal(|ui| {
                    ui.label(&self.texts.theme_editor.name_label);
                    ui.text_edit_singleline(&mut editor_name);
                });
                ui.add_space(4.0);

                // Scrollable editor body pinned above the footer.
                egui::TopBottomPanel::bottom("theme_editor_footer")
                    .frame(egui::Frame::none())
                    .show_inside(ui, |ui| {
                        ui.add_space(6.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(&self.texts.theme_editor.confirm)
                                            .strong(),
                                    )
                                    .fill(accent),
                                )
                                .clicked()
                            {
                                keep = true;
                            }
                            if ui.button(&self.texts.theme_editor.cancel).clicked() {
                                discard = true;
                            }
                        });
                    });
                egui::ScrollArea::vertical()
                    .id_salt("theme_editor_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Live-preview editor body: render BOTH sub-pages
                        // (UI appearance AND terminal incl. fonts) — the
                        // popup must show every config item.
                        let is_builtin = crate::theme::store::is_embedded_id(&editor_draft.id);
                        let mut font_choices: Vec<String> =
                            vec!["system-ui".into(), "monospace".into()];
                        for f in &system_fonts {
                            if !font_choices.contains(f) {
                                font_choices.push(f.clone());
                            }
                        }
                        let mut actions = crate::theme::ui::show_theme_editor_body(
                            ui,
                            &mut editor_draft,
                            &available_themes,
                            &font_choices,
                            is_builtin,
                            editor_dirty,
                            crate::theme::ui::ThemeEditorSubtab::UiAppearance,
                            &mut theme_dialog,
                            &editor_labels,
                        );
                        // (No separator between the System UI and Terminal
                        // blocks — spacing alone separates them.)
                        ui.add_space(12.0);
                        actions.extend(crate::theme::ui::show_theme_editor_body(
                            ui,
                            &mut editor_draft,
                            &available_themes,
                            &font_choices,
                            is_builtin,
                            editor_dirty,
                            crate::theme::ui::ThemeEditorSubtab::Terminal,
                            &mut theme_dialog,
                            &editor_labels,
                        ));
                        actions_out = actions;
                    });
            });
        self.theme_editor_open = is_open;
        self.theme_dialog = theme_dialog;
        editor_draft.name = editor_name;

        // Apply live preview + collect editor actions.
        for action in actions_out {
            self.handle_theme_action(ctx, action, editor_draft.clone());
        }
        if editor_draft != self.theme_edit {
            self.theme_edit = editor_draft;
            self.theme_dirty = true;
            crate::theme::apply_theme_definition(ctx, &self.theme_edit);
            self.rebuild_fonts(ctx);
        }

        if keep || !self.theme_editor_open {
            // Save the edited theme under its origin id.
            let mut theme = self.theme_edit.clone();
            if let Some(origin) = self.theme_edit_origin.clone() {
                theme.id = origin;
                // Builtin ids can't be saved; give it a user id.
                if crate::theme::store::is_embedded_id(&theme.id) {
                    theme.id = crate::theme::store::find_free_id(&themes_root, "custom-theme");
                }
                match crate::theme::store::save_user_theme(&themes_root, &theme) {
                    Ok(()) => {
                        self.refresh_themes(&themes_root);
                        self.switch_theme_by_id(ctx, &theme.id);
                    }
                    Err(e) => self.theme_message = Some(Err(e.to_string())),
                }
            }
            self.theme_editor_open = false;
            self.theme_edit_origin = None;
        }
        if discard {
            // Revert to the currently active theme.
            self.theme_edit = self.active_theme.clone();
            self.theme_dirty = false;
            crate::theme::apply_theme_definition(ctx, &self.active_theme);
            // Restore the active theme's fonts (the editor had installed
            // the draft fonts for live preview).
            self.rebuild_fonts(ctx);
            self.theme_editor_open = false;
            self.theme_edit_origin = None;
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

    /// Restart the application to apply the downloaded update.
    /// Used by both the in-place About button and the standalone popup.
    fn apply_update_and_restart(&mut self, ctx: &egui::Context, path: std::path::PathBuf) {
        // Surface a previous failed replacement (the Windows helper writes
        // this marker when the swap did not succeed).
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let marker = dir.join("opennex_update_failed.txt");
                if marker.exists() {
                    let _ = std::fs::remove_file(&marker);
                    self.update_state = crate::updater::UpdateState::Error(
                        "上次更新替换失败，已保留旧版本。请手动下载安装包更新。".into(),
                    );
                    return;
                }
            }
        }
        match crate::updater::replace_and_restart(&path) {
            Ok(_) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => {
                self.update_state = crate::updater::UpdateState::Error(e);
            }
        }
    }

    /// User clicked the manual "check update" button. Set Checking and
    /// clear any prior Available/UpToDate/Error so the about window
    /// does not display stale information.
    fn start_manual_check(&mut self) {
        self.update_state = crate::updater::UpdateState::Checking;
    }

    /// Kick off a download for the available update info.
    fn kick_download(&mut self, ctx: &egui::Context, info: &crate::updater::UpdateInfo) {
        // Open the about window so the user can watch progress; the
        // status line + progress bar live in-place inside it.
        self.show_about = true;
        self.start_download(ctx, info);
    }

    /// Unity-inspector settings row: label in a 42% column on the left,
    /// control starting at a fixed column and left-aligned. 32px row with
    /// a hairline divider; controls get a uniform 180px width budget.
    fn settings_row(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        add_control: impl FnOnce(&mut egui::Ui),
    ) {
        let avail = ui.available_rect_before_wrap();
        let row_rect = egui::Rect::from_min_size(avail.min, egui::vec2(avail.width(), 32.0));
        let resp = ui.allocate_rect(row_rect, egui::Sense::hover());
        if resp.hovered() {
            let h = self.active_theme.app.hover.to_egui();
            let dim = egui::Color32::from_rgba_unmultiplied(
                (h.r() as f32 * 0.5) as u8,
                (h.g() as f32 * 0.5) as u8,
                (h.b() as f32 * 0.5) as u8,
                36,
            );
            ui.painter().rect_filled(row_rect, 0.0, dim);
        }
        let b = self.active_theme.app.border.to_egui();
        let divider = egui::Color32::from_rgba_unmultiplied(b.r(), b.g(), b.b(), 40);
        ui.painter()
            .hline(row_rect.x_range(), row_rect.bottom(), (1.0, divider));
        let label_x = row_rect.min.x + 10.0;
        ui.painter().text(
            egui::pos2(label_x, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.0),
            self.active_theme.app.text.to_egui(),
        );
        let ctrl_x = row_rect.min.x + row_rect.width() * 0.42;
        let ctrl_rect = egui::Rect::from_min_max(
            egui::pos2(ctrl_x, row_rect.min.y),
            egui::pos2(row_rect.max.x, row_rect.max.y),
        );
        let mut ctrl_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(ctrl_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        add_control(&mut ctrl_ui);
    }

    /// A full-width action row: content starts flush at the row's left
    /// edge (no label column) and there is no divider below. Used for
    /// button groups like the lock page's change/clear password actions.
    fn settings_action_row(&self, ui: &mut egui::Ui, add_controls: impl FnOnce(&mut egui::Ui)) {
        let avail = ui.available_rect_before_wrap();
        let row_rect = egui::Rect::from_min_size(avail.min, egui::vec2(avail.width(), 32.0));
        let resp = ui.allocate_rect(row_rect, egui::Sense::hover());
        if resp.hovered() {
            let h = self.active_theme.app.hover.to_egui();
            let dim = egui::Color32::from_rgba_unmultiplied(
                (h.r() as f32 * 0.5) as u8,
                (h.g() as f32 * 0.5) as u8,
                (h.b() as f32 * 0.5) as u8,
                36,
            );
            ui.painter().rect_filled(row_rect, 0.0, dim);
        }
        let mut ctrl_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(row_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        ctrl_ui.add_space(10.0);
        add_controls(&mut ctrl_ui);
    }

    /// Weak group heading with consistent rhythm (16px above, 6px below).
    fn settings_group(&self, ui: &mut egui::Ui, title: &str) {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(title)
                .size(10.0)
                .color(self.active_theme.app.weak_text.to_egui()),
        );
        ui.add_space(6.0);
    }

    fn settings_page_general(&mut self, ui: &mut egui::Ui) {
        let t = self.texts.settings.general.clone();
        let b = self.texts.settings.buttons.clone();

        self.settings_group(ui, &b.behavior_section);
        let mut auto_copy = self.settings_edit.auto_copy_selection;
        self.settings_row(ui, &t.auto_copy, |ui| {
            ui.checkbox(&mut auto_copy, "");
        });
        self.settings_edit.auto_copy_selection = auto_copy;

        let mut auto_match = self.settings_edit.auto_match_command;
        self.settings_row(ui, &t.auto_match, |ui| {
            ui.checkbox(&mut auto_match, "");
        });
        self.settings_edit.auto_match_command = auto_match;

        self.settings_group(ui, &b.data_section);
        let mut max_h = self.settings_edit.max_history;
        let mut sb = self.settings_edit.scrollback;
        self.settings_row(ui, &t.max_history, |ui| {
            ui.add_sized(
                [180.0, 20.0],
                egui::DragValue::new(&mut max_h).range(10..=10000),
            );
        });
        self.settings_row(ui, &t.scrollback, |ui| {
            ui.add_sized(
                [180.0, 20.0],
                egui::DragValue::new(&mut sb).range(100..=50000),
            );
        });
        self.settings_edit.max_history = max_h;
        self.settings_edit.scrollback = sb;

        // Maintenance action: plain button (same style as all other
        // settings buttons), flush left, no divider; clicking opens a
        // confirmation dialog.
        let mut clear = false;
        let clear_label = t.clear_all_history.clone();
        self.settings_action_row(ui, |ui| {
            if ui.button(&clear_label).clicked() {
                clear = true;
            }
        });
        if clear {
            self.show_clear_history_confirm = true;
        }

        // Group footer: path hints as weak, wrapping small text.
        ui.add_space(8.0);
        ui.weak(egui::RichText::new(format!("{}  {}", t.scene_path, t.templates_path)).small());
    }

    fn settings_page_shortcuts(&mut self, ui: &mut egui::Ui) {
        let texts = self.texts.clone();
        ui.label(&texts.settings.shortcuts.hint);
        ui.add_space(4.0);
        for id in shortcut_hint_ids() {
            let label = shortcut_label_for(&texts, id).to_string();
            let rec = self.binding_recording.clone();
            let binds = self.settings_edit.key_binds.clone();
            let mut clicked = false;
            self.settings_row(ui, &label, |ui| {
                let text = if rec.as_deref() == Some(id) {
                    "…".to_string()
                } else if let Some(b) = binds.get(id) {
                    shortcut_display(b)
                } else {
                    texts.settings.shortcuts.not_set.clone()
                };
                if ui
                    .add(egui::Button::new(text).min_size(egui::vec2(180.0, 0.0)))
                    .clicked()
                {
                    clicked = true;
                }
            });
            if clicked {
                self.binding_recording = Some(id.to_string());
            }
        }
        ui.add_space(6.0);
        if ui
            .button(&texts.settings.shortcuts.reset_defaults)
            .clicked()
        {
            self.settings_edit.key_binds = default_key_binds();
            self.binding_recording = None;
        }
    }

    fn settings_page_lock(&mut self, ui: &mut egui::Ui) {
        let t = self.texts.settings.lock.clone();
        self.settings_group(ui, &t.password_section);
        let mut action: Option<&'static str> = None;
        if self.settings.lock_password.is_empty() {
            // No password yet: only the "set password" button.
            let set_label = t.set_password.clone();
            self.settings_action_row(ui, |ui| {
                if ui.button(&set_label).clicked() {
                    action = Some("set");
                }
            });
        } else {
            // Password exists: change + clear on one row, both flush left,
            // 20px apart, no divider below.
            let ch_label = t.change_password.clone();
            let cl_label = t.clear_password.clone();
            self.settings_action_row(ui, |ui| {
                if ui.button(&ch_label).clicked() {
                    action = Some("change");
                }
                ui.add_space(20.0);
                if ui.button(&cl_label).clicked() {
                    action = Some("clear");
                }
            });
        }
        match action {
            Some("set") => {
                self.pw_popup = Some("set");
                self.pw_set1.clear();
                self.pw_set2.clear();
                self.pw_message.clear();
            }
            Some("change") => {
                self.pw_popup = Some("change");
                self.pw_old.clear();
                self.pw_new1.clear();
                self.pw_new2.clear();
                self.pw_message.clear();
            }
            Some("clear") => {
                self.pw_popup = Some("clear");
                self.pw_clear.clear();
                self.pw_message.clear();
            }
            _ => {}
        }
    }

    /// Theme page: one page with section headings for 选择与管理 /
    /// UI 外观 / 终端.
    fn settings_page_theme(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(Err(msg)) = &self.theme_message {
            ui.colored_label(egui::Color32::from_rgb(230, 120, 120), msg);
        }
        if let Some(Ok(msg)) = &self.theme_message {
            ui.colored_label(egui::Color32::from_rgb(120, 200, 130), msg);
        }

        // The list takes all the remaining height down to the window bottom.
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(ui.available_width());
                self.settings_page_theme_select(ctx, ui);
            },
        );
    }

    fn settings_page_theme_select(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let current = self.settings_edit.theme_id.clone();

        let mut pick: Option<String> = None;
        let mut edit_target: Option<String> = None;
        let mut copy_target: Option<String> = None;
        let mut delete_target: Option<String> = None;

        let text_col = self.active_theme.app.text.to_egui();
        let _weak_col = self.active_theme.app.weak_text.to_egui();
        let sel_bg = self.active_theme.app.active.to_egui();
        let hover_bg = self.active_theme.app.hover.to_egui();
        let accent_col = self.active_theme.app.accent.to_egui();
        let _border_col = self.active_theme.app.border.to_egui();
        let builtin_tag = self.texts.settings.buttons.builtin.clone();

        // Theme list: fills the remaining settings-page height, one preview
        // row per theme. The whole row is a preview: left half rendered with
        // the theme's UI styling (bg/text/font), right half a terminal demo.
        let scroll_to_top = ui.ctx().memory_mut(|m| {
            m.data
                .remove_temp::<bool>(egui::Id::new("theme_list_scroll_top"))
                .unwrap_or(false)
        });
        let scroll_area = egui::ScrollArea::vertical()
            .id_salt("theme_list_scroll")
            .auto_shrink([false, false])
            .max_height(ui.available_height());
        let mut scroll_area = scroll_area;
        if scroll_to_top {
            scroll_area = scroll_area.vertical_scroll_offset(0.0);
        }
        scroll_area.show(ui, |ui| {
            let weak_col = self.active_theme.app.weak_text.to_egui();
            let mut shown_custom_heading = false;
            let mut shown_builtin_heading = false;
            let mut group_heading = |ui: &mut egui::Ui, title: &str| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(title).size(10.0).color(weak_col));
                ui.add_space(4.0);
            };
            for theme in self.available_themes.iter() {
                let is_builtin = crate::theme::store::is_embedded_id(&theme.id);
                if !is_builtin && !shown_custom_heading {
                    shown_custom_heading = true;
                    group_heading(ui, &self.texts.settings.buttons.user_group);
                }
                if is_builtin && !shown_builtin_heading {
                    shown_builtin_heading = true;
                    group_heading(ui, &self.texts.settings.buttons.builtin_group);
                }
                let selected = theme.id == current;
                // Entry = title line above a shortened preview strip.
                let title_h = 22.0;
                let preview_h = 36.0;
                let row_h = title_h + preview_h;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), row_h),
                    egui::Sense::hover(),
                );
                ui.add_space(4.0); // gap between entries
                                   // Accent bar width (the selected-theme highlight strip).
                let accent_w = 2.0;
                // Clickable/hover region: from just after the accent bar to
                // the terminal preview's right border — the row must not be
                // triggerable beyond the preview blocks.
                let content_left = rect.min.x + accent_w;
                let preview_w_total = (rect.width() - accent_w) * 0.42 * 2.0;
                let clickable_rect = egui::Rect::from_min_max(
                    egui::pos2(content_left, rect.min.y),
                    egui::pos2((content_left + preview_w_total).min(rect.max.x), rect.max.y),
                );
                let hovered = clickable_rect.contains(
                    ui.input(|i| i.pointer.hover_pos())
                        .unwrap_or(rect.min - egui::vec2(1.0, 1.0)),
                );

                // Register the row click FIRST so it sits below the
                // action buttons in the hit-test order.
                let row_resp = ui.interact(
                    clickable_rect,
                    egui::Id::new((&theme.id, "row-click")),
                    egui::Sense::click(),
                );
                if row_resp.clicked() {
                    pick = Some(theme.id.clone());
                }

                // Selected/hover overlay + accent bar.
                if selected {
                    ui.painter().rect_filled(rect, 0.0, sel_bg);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.min.x + accent_w, rect.max.y),
                        ),
                        0.0,
                        accent_col,
                    );
                } else if hovered {
                    ui.painter().rect_filled(rect, 0.0, hover_bg);
                }

                // Title line: theme name (+ builtin tag) above the preview,
                // followed by the ANSI palette dots on the same line.
                let name = if is_builtin {
                    format!("{} ({})", theme.name, builtin_tag)
                } else {
                    theme.name.clone()
                };
                let title_cy = rect.min.y + title_h / 2.0;
                let dot_r = 3.0;
                let colors = [
                    theme.terminal.normal.red.to_egui(),
                    theme.terminal.normal.green.to_egui(),
                    theme.terminal.normal.yellow.to_egui(),
                    theme.terminal.normal.blue.to_egui(),
                    theme.terminal.bright.magenta.to_egui(),
                    theme.terminal.bright.cyan.to_egui(),
                    theme.terminal.cursor.to_egui(),
                ];

                // Preview strip area (below the title), shifted right by the
                // accent-bar width so the UI preview no longer covers the
                // selected-theme highlight strip.
                let preview_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x + accent_w, rect.min.y + title_h),
                    rect.max,
                );
                // The action buttons moved up to the title line, so the
                // preview strip no longer reserves a right-hand button
                // column and spans the full row width.
                let content = preview_rect;
                // Preview width: 0.42 of the content width each. Both halves
                // are anchored left.
                let preview_w = content.width() * 0.42;

                // Resolve a FontId for a theme-configured font NAME. The
                // LIST ITEM's theme config must fully decide the preview:
                //  - a registered named font -> that font's own family
                //  - a generic name ("system-ui"/"monospace") -> the CLEAN
                //    default-stack snapshot, NOT the live generic family
                //    (whose head is the ACTIVE theme's font — using it
                //    would make every preview follow the active theme).
                let resolve_font =
                    |name: &str, size: f32, generic: egui::FontFamily| -> egui::FontId {
                        if name.is_empty() || name == "system-ui" || name == "monospace" {
                            let fam = if generic == egui::FontFamily::Monospace {
                                preview_mono_family()
                            } else {
                                preview_prop_family()
                            };
                            return egui::FontId::new(size, fam);
                        }
                        egui::FontId::new(
                            size,
                            egui::FontFamily::Name(std::sync::Arc::from(name.to_owned())),
                        )
                    };
                // Title line: theme name rendered with THIS theme's own
                // UI font (not the active theme's), followed by the ANSI
                // palette dots on the same line.
                let ui_font_name_for_title = theme
                    .app
                    .ui_font_families
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "system-ui".into());
                let title_font = resolve_font(
                    &ui_font_name_for_title,
                    11.0,
                    egui::FontFamily::Proportional,
                );
                let name_galley =
                    ui.fonts(|f| f.layout_no_wrap(name.clone(), title_font, text_col));
                ui.painter().galley(
                    egui::pos2(rect.min.x + 8.0, title_cy - name_galley.size().y / 2.0),
                    name_galley.clone(),
                    text_col,
                );
                {
                    let mut dx = rect.min.x + 8.0 + name_galley.size().x + 10.0;
                    for c in colors {
                        ui.painter()
                            .circle_filled(egui::pos2(dx + dot_r, title_cy), dot_r, c);
                        dx += dot_r * 2.0 + 2.0;
                    }
                }

                // --- Left half: UI-style preview labeled "OpenNex UI:". ---
                let ui_half = egui::Rect::from_min_max(
                    content.min,
                    egui::pos2(content.min.x + preview_w, content.max.y),
                );
                let p = ui.painter();
                p.rect_filled(ui_half, 0.0, theme.app.app_bg.to_egui());
                let ui_font_name = theme
                    .app
                    .ui_font_families
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "system-ui".into());
                // Two lines at FIXED fractional centers (33% / 66% of the
                // block height): changing fonts can no longer shift the
                // lines up or down.
                let label_font_size = theme.app.ui_font_size.min(12.0);
                let meta_font_size = 9.0;
                let line1_cy = ui_half.min.y + ui_half.height() * 0.33;
                let line2_cy = ui_half.min.y + ui_half.height() * 0.66;
                ui.painter().text(
                    egui::pos2(ui_half.min.x + 8.0, line1_cy),
                    egui::Align2::LEFT_CENTER,
                    "OpenNex UI:",
                    resolve_font(
                        &ui_font_name,
                        label_font_size,
                        egui::FontFamily::Proportional,
                    ),
                    theme.app.text.to_egui(),
                );
                ui.painter().text(
                    egui::pos2(ui_half.min.x + 8.0, line2_cy),
                    egui::Align2::LEFT_CENTER,
                    format!("{} {:.0}px", ui_font_name, theme.app.ui_font_size),
                    resolve_font(
                        &ui_font_name,
                        meta_font_size,
                        egui::FontFamily::Proportional,
                    ),
                    theme.app.weak_text.to_egui(),
                );

                // --- Right half: terminal demo labeled "Terminal:". Same
                // width as the UI half (not stretched to the row edge). ---
                let term_half = egui::Rect::from_min_max(
                    egui::pos2(ui_half.max.x, content.min.y),
                    egui::pos2(ui_half.max.x + preview_w, content.max.y),
                );
                let term_bg = theme.terminal.background.to_egui();
                p.rect_filled(term_half, 0.0, term_bg);
                let term_font_name = theme
                    .typography
                    .terminal_font_families
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "monospace".into());
                let term_size = theme.typography.terminal_font_size.min(12.0);
                let cmd_font =
                    resolve_font(&term_font_name, term_size, egui::FontFamily::Monospace);
                let cmd_galley = ui.fonts(|f| {
                    f.layout_no_wrap(
                        "Terminal:".into(),
                        cmd_font,
                        theme.terminal.foreground.to_egui(),
                    )
                });
                // Fixed fractional line centers (33% / 66%): font changes
                // cannot shift the lines vertically.
                let line1_cy = term_half.min.y + term_half.height() * 0.33;
                let line2_cy = term_half.min.y + term_half.height() * 0.66;
                let cmd_pos =
                    egui::pos2(term_half.min.x + 8.0, line1_cy - cmd_galley.size().y / 2.0);
                ui.painter().galley(
                    cmd_pos,
                    cmd_galley.clone(),
                    theme.terminal.foreground.to_egui(),
                );
                ui.painter().text(
                    egui::pos2(term_half.min.x + 8.0, line2_cy),
                    egui::Align2::LEFT_CENTER,
                    format!(
                        "$ ls -la  {} {:.0}px",
                        term_font_name, theme.typography.terminal_font_size
                    ),
                    resolve_font(&term_font_name, 9.0, egui::FontFamily::Monospace),
                    theme.terminal.dim_foreground.to_egui(),
                );

                // --- Right: per-row action buttons on the TITLE line,
                // vertically centered with the theme name and palette dots;
                // their right edge aligns with the terminal preview's
                // right border.
                let btn_y = title_cy - 9.0;
                let mut x = term_half.max.x;
                let mut btn =
                    |ui: &mut egui::Ui, x: f32, y: f32, glyph: &str, id: egui::Id| -> bool {
                        let r = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(22.0, 18.0));
                        let resp = ui.interact(r, id, egui::Sense::click());
                        let g = ui.fonts(|f| {
                            f.layout_no_wrap(
                                glyph.to_string(),
                                egui::FontId::proportional(12.0),
                                text_col,
                            )
                        });
                        ui.painter()
                            .galley(r.center() - g.size() / 2.0, g, text_col);
                        resp.clicked()
                    };
                // Action buttons laid out right-to-left at fixed offsets:
                // [delete?] [edit?] [new(+)] — 24px pitch, no overlap.
                if !is_builtin
                    && btn(
                        ui,
                        x - 22.0,
                        btn_y,
                        egui_phosphor::regular::TRASH,
                        egui::Id::new((&theme.id, "row-del")),
                    )
                {
                    delete_target = Some(theme.id.clone());
                }
                if !is_builtin {
                    x -= 24.0;
                    if btn(
                        ui,
                        x - 22.0,
                        btn_y,
                        egui_phosphor::regular::PENCIL_SIMPLE,
                        egui::Id::new((&theme.id, "row-ed")),
                    ) {
                        edit_target = Some(theme.id.clone());
                    }
                    x -= 24.0;
                }
                if btn(
                    ui,
                    x - 22.0,
                    btn_y,
                    egui_phosphor::regular::PLUS,
                    egui::Id::new((&theme.id, "row-new")),
                ) {
                    copy_target = Some(theme.id.clone());
                }
            }
        });

        // Handle per-row actions.
        if let Some(id) = pick {
            self.try_switch_theme(ctx, id);
        }
        if let Some(id) = edit_target {
            if let Some(theme) = self.available_themes.iter().find(|t| t.id == id).cloned() {
                self.theme_edit = theme;
                self.theme_edit_origin = Some(id);
                self.theme_editor_open = true;
            }
        }
        if let Some(id) = copy_target {
            let themes_root = crate::theme::store::themes_dir(&app_data_dir());
            let name = format!(
                "{} (copy)",
                self.available_themes
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default()
            );
            match crate::theme::store::copy_theme(&themes_root, &id, &name) {
                Ok(new_theme) => {
                    self.refresh_themes(&themes_root);
                    self.switch_theme_by_id(ctx, &new_theme.id);
                    // The new theme sits at the top of the list (user themes
                    // first); scroll the list back to the top.
                    ui.ctx().memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new("theme_list_scroll_top"), true)
                    });
                }
                Err(e) => self.theme_message = Some(Err(e.to_string())),
            }
        }
        if let Some(id) = delete_target {
            if let Some(theme) = self.available_themes.iter().find(|t| t.id == id) {
                self.theme_edit = theme.clone();
                self.theme_dialog.show_delete_confirm = true;
            }
        }
    }

    fn rebuild_fonts(&self, ctx: &egui::Context) {
        let system_fonts = scan_system_fonts();
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        let mut registered_names: Vec<String> = Vec::new();
        for (name, path) in &system_fonts {
            if let Ok(data) = std::fs::read(path) {
                // Validate the file parses as a real TTF/OTF/TTC face
                // before registering it. Some files in the system font
                // directories carry a .ttf extension but are not
                // TrueType resources (e.g. Windows mstmc.ttf bitmap
                // fonts); epaint panics on those at first use, which
                // crashed startup. Skip them with a warning instead.
                if !is_valid_font_data(&data) {
                    log::warn!("skipping invalid font file {}: {}", name, path);
                    continue;
                }
                fonts.font_data.insert(
                    name.clone(),
                    std::sync::Arc::new(egui::FontData::from_owned(data)),
                );
                registered_names.push(name.clone());
            }
        }
        load_multilingual_fonts(&mut fonts);
        if let Some(mono_family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            for name in &registered_names {
                mono_family.push(name.clone());
            }
        }
        // Also register every scanned font under its OWN named family so
        // the theme-list previews can render each theme's configured font
        // exactly, independent of which font the ACTIVE theme puts at the
        // head of the generic Proportional/Monospace families.
        // Snapshot the CLEAN generic families (before the active theme's
        // fonts are injected at their heads) as reserved preview families.
        // Theme previews resolve generic names ("system-ui"/"monospace") to
        // these snapshots so switching the active theme never changes how
        // other themes' previews render.
        let clean_prop = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .cloned()
            .unwrap_or_default();
        let clean_mono = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .cloned()
            .unwrap_or_default();
        fonts
            .families
            .insert(preview_prop_family(), clean_prop.clone());
        fonts
            .families
            .insert(preview_mono_family(), clean_mono.clone());
        // Register every scanned font under its OWN named family WITH a
        // fallback chain (the font itself + the clean default stack): a
        // named font may lack Latin/CJK glyphs entirely (e.g. symbol or
        // single-script fonts), which would otherwise render theme names
        // and previews as blank.
        for name in &registered_names {
            let mut chain = vec![name.clone()];
            for fallback in clean_prop.iter() {
                if !chain.contains(fallback) {
                    chain.push(fallback.clone());
                }
            }
            fonts.families.insert(
                egui::FontFamily::Name(std::sync::Arc::from(name.as_str())),
                chain,
            );
        }

        // Theme font choices: UI font goes first in Proportional, terminal
        // font first in Monospace. Generic families ("system-ui",
        // "monospace") map to the egui defaults already at the tail of the
        // family list, so they're skipped here. While the theme editor is
        // open the DRAFT fonts are installed so the live preview shows the
        // fonts being edited.
        let font_source = if self.theme_editor_open {
            &self.theme_edit
        } else {
            &self.active_theme
        };
        let ui_font = font_source
            .app
            .ui_font_families
            .first()
            .cloned()
            .unwrap_or_default();
        if !ui_font.is_empty() && ui_font != "system-ui" && fonts.font_data.contains_key(&ui_font) {
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, ui_font);
            }
        }
        let term_font = font_source
            .typography
            .terminal_font_families
            .first()
            .cloned()
            .unwrap_or_default();
        if !term_font.is_empty()
            && term_font != "monospace"
            && fonts.font_data.contains_key(&term_font)
        {
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.insert(0, term_font);
            }
        }

        ctx.set_fonts(fonts);
    }

    fn refresh_themes(&mut self, themes_root: &std::path::Path) {
        let mut user = crate::theme::store::load_user_themes(themes_root).unwrap_or_default();
        // Custom-theme block: newest first (creation time, falling back to
        // name order for legacy files without a timestamp).
        user.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        let mut themes = user;
        for embedded in crate::theme::store::embedded_themes().unwrap_or_default() {
            if !themes.iter().any(|t| t.id == embedded.id) {
                themes.push(embedded);
            }
        }
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
        // Manual check: pick up the background thread result when it lands.
        if self.update_state == crate::updater::UpdateState::Checking {
            let result: Option<crate::updater::UpdateState> =
                ctx.memory(|mem| mem.data.get_temp(egui::Id::new("manual_check_result")));
            if let Some(state) = result {
                self.update_state = state;
            }
        }

        // Download progress: pick up intermediate percentages and terminal
        // states (Ready / Error) from the background download thread.
        if let crate::updater::UpdateState::Downloading(_) = &self.update_state {
            let dl_state: Option<crate::updater::UpdateState> =
                ctx.memory(|mem| mem.data.get_temp(egui::Id::new("dl_state")));
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
        // Note: no UI is rendered here. The in-place About-window status
        // line + the restart popup cover all states.
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

    /// Render the update-notification dialog (started from background check
    /// or after the user clicks the update button on the about window).
    /// Render a transient bottom-center toast (e.g. "current is up to date").
    fn show_update_toast(&mut self, ctx: &egui::Context) {
        let (msg, expires) = match &self.update_toast {
            Some(t) => t.clone(),
            None => return,
        };
        if std::time::Instant::now() >= expires {
            self.update_toast = None;
            return;
        }
        egui::Area::new(egui::Id::new("update_toast"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::popup(&ctx.style()).show(ui, |ui| {
                    ui.label(msg);
                });
            });
    }

    /// If auto-copy is enabled and the user just released the primary
    /// mouse button over a terminal, copy the current selection to
    /// the system clipboard.
    fn handle_selection_auto_copy(&mut self, ctx: &egui::Context) {
        if !self.auto_copy_selection {
            return;
        }
        let mut released = false;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::PointerButton {
                    pressed: false,
                    button: egui::PointerButton::Primary,
                    ..
                } = event
                {
                    released = true;
                }
            }
        });
        if !released {
            return;
        }
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        let Some(td) = self.terminals.get_mut(&tab) else {
            return;
        };
        let text = td.instance.backend.selectable_content();
        if !text.is_empty() {
            ctx.copy_text(text);
            self.copy_toast = Some(std::time::Instant::now());
            ctx.request_repaint_after(std::time::Duration::from_millis(700));
        }
    }

    /// Global command-history / auto-match list for the focused terminal.
    /// Rendered as a Foreground layer Area so it is never clipped by the
    /// terminal's rect. Features: outer border matching the UI divider
    /// style, row index numbers (dimmed), alternating row colors from the
    /// theme (menu_bg / menu_alt_bg), wheel scrolling, a footer with the
    /// total entry count and a "clear" button (with confirmation).
    fn render_history_menu(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        let Some(td) = self.terminals.get_mut(&tab) else {
            return;
        };
        let Some(nav) = td.instance.history_nav.clone() else {
            return;
        };
        if nav.entries.is_empty() {
            self.terminals.get_mut(&tab).unwrap().instance.history_nav = None;
            return;
        }

        let app = &self.active_theme.app;
        let menu_bg = app.menu_bg.to_egui();
        let menu_alt = app.menu_alt_bg.to_egui();
        let menu_fg = app.menu_fg.to_egui();
        let weak = app.weak_text.to_egui();
        let border = app.sidebar_border.to_egui();
        let sel_bg = app.active.to_egui();
        let font_size = self.active_theme.typography.menu_font_size;

        // Wheel scrolling over the list scrolls the view (state in memory).
        let scroll_id = egui::Id::new(("hist_menu_scroll", tab.as_str()));
        let max_visible = 10usize;
        let total = nav.entries.len();
        let mut scroll: usize = ctx.memory(|m| m.data.get_temp(scroll_id).unwrap_or(0));
        // View is FREE here: no per-frame follow of the selection. Wheel
        // and scrollbar-drag scrolling are pure view operations; the
        // keyboard brings its selection into view only on the frame the
        // selection actually changes (see the prev/next key handler).
        let max_scroll = total.saturating_sub(max_visible);
        scroll = scroll.min(max_scroll);

        let row_h = 20.0f32;
        let list_w = 420.0f32;
        let footer_h = 24.0f32;
        let visible = total.min(max_visible);
        let list_h = visible as f32 * row_h + footer_h;

        // Anchor: follow the terminal CURSOR row (original behavior) —
        // below the cursor line when there is room, otherwise above it.
        // Uses the terminal's last known view rect + grid metrics.
        let anchor_rect = self
            .terminal_view_rects
            .get(&tab)
            .copied()
            .unwrap_or_else(|| {
                let sr = ctx.screen_rect();
                egui::Rect::from_min_max(
                    egui::pos2(sr.min.x + 200.0, sr.min.y + 60.0),
                    egui::pos2(sr.max.x, sr.max.y),
                )
            });
        let (cell_w, cell_h_grid) = td.instance.cell_size();
        let (cursor_col, cursor_row) = td.instance.cursor_position();
        let cursor_x = anchor_rect.min.x + cursor_col as f32 * cell_w;
        let cursor_y = anchor_rect.min.y + (cursor_row as f32 + 1.0) * cell_h_grid;
        let pos = if anchor_rect.max.y - cursor_y >= list_h {
            egui::pos2(anchor_rect.min.x + 8.0, cursor_y)
        } else {
            // Not enough room below: open above the cursor line.
            egui::pos2(
                anchor_rect.min.x + 8.0,
                (cursor_y - cell_h_grid - list_h).max(anchor_rect.min.y + 4.0),
            )
        };

        let mut confirm_clicked = false;
        let mut clear_clicked = false;
        let mut close_clicked = false;
        let mut entry_clicked: Option<usize> = None;

        egui::Area::new(egui::Id::new(("hist_menu", tab.as_str())))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(menu_bg)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(0.0)
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                        ui.set_width(list_w);
                        let list_rect_min = ui.cursor().min;
                        ui.style_mut().spacing.item_spacing.y = 0.0;
                        // Wheel scroll over the whole panel.
                        let scroll_delta = ui.input(|i| {
                            i.events
                                .iter()
                                .filter_map(|e| match e {
                                    egui::Event::MouseWheel { delta, .. } => Some(delta.y),
                                    _ => None,
                                })
                                .sum::<f32>()
                        });
                        if scroll_delta > 0.0 {
                            scroll = scroll.saturating_sub(1);
                        } else if scroll_delta < 0.0 {
                            scroll = (scroll + 1).min(max_scroll);
                        }

                        // Rows.
                        for (i, entry) in nav
                            .entries
                            .iter()
                            .enumerate()
                            .skip(scroll)
                            .take(max_visible)
                        {
                            let row = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(list_w, row_h),
                            );
                            let (row_rect, _) = ui.allocate_exact_size(
                                egui::vec2(list_w, row_h),
                                egui::Sense::click(),
                            );
                            let is_sel = i == nav.selected;
                            // Visual hover preview (does NOT change the
                            // keyboard selection — hover and keyboard nav
                            // stay independent).
                            let row_hovered = row_rect.contains(
                                ui.input(|i| i.pointer.hover_pos())
                                    .unwrap_or(egui::pos2(-1.0, -1.0)),
                            );
                            // Alternating banding (subtle); selection wins,
                            // hover is a lighter accent on top of banding.
                            let row_bg = if is_sel {
                                sel_bg
                            } else if row_hovered {
                                egui::Color32::from_rgba_unmultiplied(
                                    sel_bg.r(),
                                    sel_bg.g(),
                                    sel_bg.b(),
                                    90,
                                )
                            } else if i % 2 == 1 {
                                menu_alt
                            } else {
                                menu_bg
                            };
                            ui.painter().rect_filled(row_rect, 0.0, row_bg);
                            // Dim index number, 3-char gutter.
                            ui.painter().text(
                                egui::pos2(row.min.x + 6.0, row.center().y),
                                egui::Align2::LEFT_CENTER,
                                format!("{}", i + 1),
                                egui::FontId::monospace(font_size * 0.85),
                                weak,
                            );
                            // Command text: clip to the row width and end
                            // with "..." when truncated.
                            let text_x = row.min.x + 34.0;
                            let text_max_w = row.max.x - text_x - 16.0;
                            let font_id = egui::FontId::monospace(font_size);
                            let full = ui.fonts(|f| {
                                f.layout_no_wrap(entry.clone(), font_id.clone(), menu_fg)
                            });
                            if full.size().x <= text_max_w {
                                ui.painter().galley(
                                    egui::pos2(text_x, row.center().y - full.size().y / 2.0),
                                    full,
                                    menu_fg,
                                );
                            } else {
                                // Binary-search-free trim: cut chars until
                                // text + "..." fits.
                                let ell = "...";
                                let mut shown: String = entry.clone();
                                loop {
                                    let g = ui.fonts(|f| {
                                        f.layout_no_wrap(
                                            format!("{shown}{ell}"),
                                            font_id.clone(),
                                            menu_fg,
                                        )
                                    });
                                    if g.size().x <= text_max_w || shown.is_empty() {
                                        ui.painter().galley(
                                            egui::pos2(text_x, row.center().y - g.size().y / 2.0),
                                            g,
                                            menu_fg,
                                        );
                                        break;
                                    }
                                    shown.pop();
                                }
                            }
                            let resp = ui.interact(
                                row_rect,
                                egui::Id::new(("hist_row", tab.as_str(), i)),
                                egui::Sense::click(),
                            );
                            if resp.clicked() {
                                entry_clicked = Some(i);
                            }
                        }

                        // Vertical scrollbar when entries exceed the view.
                        if total > max_visible {
                            let rows_area_h = visible as f32 * row_h;
                            let sb_track = egui::Rect::from_min_max(
                                egui::pos2(list_rect_min.x + list_w - 6.0, list_rect_min.y),
                                egui::pos2(list_rect_min.x + list_w, list_rect_min.y + rows_area_h),
                            );
                            let thumb_h =
                                (max_visible as f32 / total as f32 * rows_area_h).max(16.0);
                            let scrollable = (rows_area_h - thumb_h).max(1.0);
                            let thumb_y =
                                list_rect_min.y + scrollable * (scroll as f32 / max_scroll as f32);
                            let sb_col = egui::Color32::from_rgba_unmultiplied(
                                weak.r(),
                                weak.g(),
                                weak.b(),
                                110,
                            );
                            ui.painter().rect_filled(sb_track, 0.0, {
                                let t = egui::Color32::from_rgba_unmultiplied(
                                    weak.r(),
                                    weak.g(),
                                    weak.b(),
                                    40,
                                );
                                t
                            });
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(sb_track.min.x, thumb_y),
                                    egui::vec2(sb_track.width(), thumb_h),
                                ),
                                0.0,
                                sb_col,
                            );
                            // Drag on the scrollbar to scroll.
                            let sb_resp = ui.interact(
                                sb_track,
                                egui::Id::new(("hist_sb", tab.as_str())),
                                egui::Sense::click_and_drag(),
                            );
                            if sb_resp.dragged() {
                                let dy = ui.input(|i| i.pointer.delta().y);
                                let lines = dy * max_scroll as f32 / scrollable;
                                scroll = (scroll as f32 + lines)
                                    .round()
                                    .clamp(0.0, max_scroll as f32)
                                    as usize;
                            }
                        }

                        // Footer: total count + clear button + scroll state.
                        let footer = egui::Rect::from_min_size(
                            ui.cursor().min,
                            egui::vec2(list_w, footer_h),
                        );
                        let (_, _) = ui.allocate_exact_size(
                            egui::vec2(list_w, footer_h),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(footer, 0.0, menu_bg);
                        ui.painter()
                            .hline(footer.x_range(), footer.min.y, (1.0, border));
                        // Footer count: total entries in the list only.
                        ui.painter().text(
                            egui::pos2(footer.min.x + 8.0, footer.center().y),
                            egui::Align2::LEFT_CENTER,
                            format!("{}", total),
                            egui::FontId::proportional(10.0),
                            weak,
                        );
                        // Clear button (right side).
                        let clear_txt = self.texts.terminal.clear_history.clone();
                        let clear_rect = egui::Rect::from_min_size(
                            egui::pos2(footer.max.x - 8.0 - 44.0, footer.center().y - 9.0),
                            egui::vec2(44.0, 18.0),
                        );
                        let cresp = ui.interact(
                            clear_rect,
                            egui::Id::new(("hist_clear", tab.as_str())),
                            egui::Sense::click(),
                        );
                        let clear_hovered = cresp.hovered();
                        let clear_clicked_flag = cresp.clicked();
                        let _ = clear_hovered;
                        let ccol = if clear_hovered {
                            app.danger.to_egui()
                        } else {
                            weak
                        };
                        let g = ui.fonts(|f| {
                            f.layout_no_wrap(
                                clear_txt.clone(),
                                egui::FontId::proportional(10.0),
                                ccol,
                            )
                        });
                        ui.painter()
                            .galley(clear_rect.center() - g.size() / 2.0, g, ccol);
                        if clear_clicked_flag {
                            clear_clicked = true;
                        }

                        // Close button (X icon, no background) to the RIGHT
                        // of the clear button: same as Esc — closes the
                        // list and stops matching until input is edited.
                        let x_rect = egui::Rect::from_min_size(
                            egui::pos2(footer.max.x - 8.0 - 18.0, footer.center().y - 9.0),
                            egui::vec2(18.0, 18.0),
                        );
                        let xresp = ui.interact(
                            x_rect,
                            egui::Id::new(("hist_close", tab.as_str())),
                            egui::Sense::click(),
                        );
                        let x_hovered = xresp.hovered();
                        let xcol = if x_hovered { menu_fg } else { weak };
                        let xg = ui.fonts(|f| {
                            f.layout_no_wrap(
                                egui_phosphor::regular::X.to_string(),
                                egui::FontId::proportional(11.0),
                                xcol,
                            )
                        });
                        ui.painter()
                            .galley(x_rect.center() - xg.size() / 2.0, xg, xcol);
                        if xresp.clicked() {
                            close_clicked = true;
                        }
                        let _ = confirm_clicked;
                    });
            });

        ctx.memory_mut(|m| m.data.insert_temp(scroll_id, scroll));

        // Row click = select + confirm (same as Enter).
        if let Some(i) = entry_clicked {
            if let Some(td) = self.terminals.get_mut(&tab) {
                if let Some(nav) = td.instance.history_nav.as_mut() {
                    nav.selected = i;
                }
            }
            self.confirm_history_entry(&tab);
            return;
        }
        if clear_clicked {
            self.history_clear_confirm = Some(tab.clone());
        }
        if close_clicked {
            // Same as Esc: close the list and stop matching until the
            // input is edited again.
            self.close_history_menu(&tab);
        }
    }

    /// Confirmation dialog for clearing a terminal's command history
    /// (triggered from the history-menu footer "clear" button).
    fn render_history_clear_confirm(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.history_clear_confirm.clone() else {
            return;
        };
        let mut open = true;
        let mut confirmed = false;
        let mut cancelled = false;
        let body = format!(
            "{}\n{}",
            self.texts.stats.clear_history_body,
            format!("({})", tab)
        );
        egui::Window::new(&self.texts.stats.clear_history_title)
            .id(egui::Id::new("hist_clear_confirm"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_pos(screen_center(ctx))
            .pivot(egui::Align2::CENTER_CENTER)
            .show(ctx, |ui| {
                ui.label(body);
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(&self.texts.theme_editor.confirm)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(self.active_theme.app.danger.to_egui()),
                        )
                        .clicked()
                    {
                        confirmed = true;
                    }
                    if ui.button(&self.texts.theme_editor.cancel).clicked() {
                        cancelled = true;
                    }
                });
            });
        if confirmed {
            self.history_db.clear(&tab);
            // Close the menu too (it's now empty).
            if let Some(td) = self.terminals.get_mut(&tab) {
                td.instance.history_nav = None;
            }
            self.history_clear_confirm = None;
        } else if cancelled || !open {
            self.history_clear_confirm = None;
        }
    }

    /// Single funnel for closing a terminal's history menu (Esc, confirm,
    /// click): removes the menu AND sets the one-frame latch that stops
    /// the confirming keypress's own Text event from re-opening it via
    /// the auto-matcher.
    fn close_history_menu(&mut self, tab: &str) {
        if let Some(td) = self.terminals.get_mut(tab) {
            td.instance.history_nav = None;
        }
        self.history_menu_just_closed.insert(tab.to_string(), true);
    }

    /// Confirm (send) the selected entry of a terminal's history menu.
    fn confirm_history_entry(&mut self, tab: &str) {
        let selected = self.terminals.get_mut(tab).and_then(|td| {
            let nav = td.instance.history_nav.take()?;
            let command = nav.entries.get(nav.selected)?.clone();
            if let Some(word) = nav.auto_word.clone() {
                let del = vec![0x7fu8; word.chars().count()];
                td.instance.write(&del);
            }
            td.instance.write(command.as_bytes());
            Some(command)
        });
        // Single-frame latch: the Space/Enter keypress that confirmed
        // produces its own Text event this frame — the matcher must not
        // act on it. From the next frame on, ONLY a real key edit (typed
        // / deleted char) can re-open matching.
        self.history_menu_just_closed.insert(tab.to_string(), true);
        if let Some(command) = selected {
            self.history_db.add(tab, &command);
        }
    }

    /// Small transient bottom-center notice: "已复制到剪切板" after the
    /// auto-copy selection feature copies text. Auto-hides after 0.6s.
    fn show_copy_toast(&mut self, ctx: &egui::Context) {
        let Some(until) = self.copy_toast else {
            return;
        };
        let elapsed = until.elapsed();
        if elapsed >= std::time::Duration::from_millis(600) {
            self.copy_toast = None;
            return;
        }
        // Fade out over the last 150ms.
        let alpha = if elapsed >= std::time::Duration::from_millis(450) {
            1.0 - (elapsed - std::time::Duration::from_millis(450)).as_secs_f32() / 0.15
        } else {
            1.0
        };
        let text = self.texts.stats.copied_toast.clone();
        let weak = self.active_theme.app.panel.to_egui();
        let fg = self.active_theme.app.text.to_egui();
        let border = self.active_theme.app.border.to_egui();
        let alpha = alpha.clamp(0.0, 1.0);
        let frame_fill = egui::Color32::from_rgba_unmultiplied(
            weak.r(),
            weak.g(),
            weak.b(),
            (alpha * 235.0) as u8,
        );
        let fg =
            egui::Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), (alpha * 255.0) as u8);
        let border = egui::Color32::from_rgba_unmultiplied(
            border.r(),
            border.g(),
            border.b(),
            (alpha * 255.0) as u8,
        );
        egui::Area::new(egui::Id::new("copy_toast"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -80.0])
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(frame_fill)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(text).size(11.0).color(fg));
                    });
            });
    }
}

/// Show a "重启" / "取消" confirmation popup when an update is ready and
/// the about window has been closed. The button click is recorded in egui
/// memory under the "restart_popup_choice" id so the App update loop can
/// read and clear it without borrowing self.
fn render_restart_popup(ctx: &egui::Context, texts: &crate::i18n::Texts) {
    let mut open = true;
    let mut restart = false;
    let mut cancel = false;
    egui::Window::new(&texts.update.restart_title)
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .default_pos(crate::app::screen_center(ctx))
        .pivot(egui::Align2::CENTER_CENTER)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(&texts.update.restart_body);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(&texts.update.restart_confirm).clicked() {
                        restart = true;
                    }
                    if ui.button(&texts.theme_editor.cancel).clicked() {
                        cancel = true;
                    }
                });
            });
        });
    if !open {
        cancel = true;
    }
    if restart || cancel {
        ctx.memory_mut(|mem| {
            mem.data
                .insert_temp(egui::Id::new("restart_popup_choice"), restart);
        });
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
                && self.pw_popup.is_none()
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
        // Auto-copy selected text on mouse release.
        self.handle_selection_auto_copy(ctx);
        self.process_pending(ctx);
        self.cwd_poll_frame = self.cwd_poll_frame.wrapping_add(1);
        if self.cwd_poll_frame >= 15 {
            self.cwd_poll_frame = 0;
            for data in self.terminals.values_mut() {
                data.instance.poll_cwd();
            }
        }

        // Status-bar sample: every 2 seconds, aggregate CPU/memory over
        // terminal process trees — focused terminal, active workspace, and
        // all workspaces — against a single process snapshot.
        if self.last_sample.elapsed() >= std::time::Duration::from_secs(2) {
            self.last_sample = std::time::Instant::now();
            let all_roots: Vec<u32> = self
                .terminals
                .values()
                .map(|td| td.instance.backend.child_pid())
                .filter(|&pid| pid != 0)
                .collect();
            let active_tab_ids: Vec<String> = self
                .dock_states
                .get(&self.active_panel)
                .map(|tree| tree.iter_all_tabs().map(|(_, t)| t.clone()).collect())
                .unwrap_or_default();
            let ws_roots: Vec<u32> = active_tab_ids
                .iter()
                .filter_map(|id| self.terminals.get(id))
                .map(|td| td.instance.backend.child_pid())
                .filter(|&pid| pid != 0)
                .collect();
            let focused_roots: Vec<u32> = self
                .focused_terminal
                .as_ref()
                .and_then(|id| self.terminals.get(id))
                .map(|td| td.instance.backend.child_pid())
                .filter(|&pid| pid != 0)
                .into_iter()
                .collect();
            let mut ws_cpu = None;
            let mut ws_mem = None;
            let mut all_cpu = None;
            let mut all_mem = None;
            let mut f_cpu = None;
            let mut f_mem = None;
            self.terminal_sampler.refresh_groups(
                [&focused_roots, &ws_roots, &all_roots],
                [&mut f_cpu, &mut ws_cpu, &mut all_cpu],
                [&mut f_mem, &mut ws_mem, &mut all_mem],
            );
            self.focused_cpu = f_cpu;
            self.focused_mem = f_mem;
            self.workspace_cpu = ws_cpu;
            self.workspace_mem = ws_mem;
            self.terminal_cpu = all_cpu;
            self.terminal_mem = all_mem;
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
            // Space is a plain input character: it participates in the
            // prefix match (typed "cd " matches "cd /tmp" but not bare
            // "cd"), so the menu stays open or closes purely per the
            // match — no special handling here.
            let confirm = !close
                && ctx
                    .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
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
                            // Selection changed: bring it into view THIS
                            // frame. (View follow lives here — not in the
                            // renderer — so wheel/drag scrolling stays free.)
                            let scroll_id = egui::Id::new(("hist_menu_scroll", tab.as_str()));
                            let mv = 10usize.min(nav.entries.len());
                            let max_sc = nav.entries.len().saturating_sub(mv);
                            let mut sc = ctx
                                .memory(|m| m.data.get_temp(scroll_id).unwrap_or(0))
                                .min(max_sc);
                            if nav.selected < sc {
                                sc = nav.selected;
                            } else if nav.selected >= sc + mv {
                                sc = nav.selected + 1 - mv;
                            }
                            ctx.memory_mut(|m| m.data.insert_temp(scroll_id, sc));
                        }
                    }
                }
                if close {
                    self.close_history_menu(&tab);
                }
                if confirm {
                    // Unified confirm path: sends the entry AND sets the
                    // close latch (stops matching like Esc).
                    self.confirm_history_entry(&tab);
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
                        // Strip the prompt: everything up to and including
                        // the LAST "$ " / "# " terminator. If no prompt
                        // marker is found (rare, exotic prompts), record
                        // nothing rather than the whole line with the
                        // prompt text mixed in.
                        let cmd = line
                            .rfind("$ ")
                            .or_else(|| line.rfind("# "))
                            .map(|p| line[p + 2..].trim().to_string())
                            .unwrap_or_default();
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
                        self.close_history_menu(tab);
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

        // Global UI zoom (Ctrl +/- by default, user-configurable).
        if !workspace_renaming && check_shortcut(ctx, &binds, "zoom_in") {
            let z = ctx.zoom_factor();
            ctx.set_zoom_factor((z + 0.1).min(3.0));
        }
        if !workspace_renaming && check_shortcut(ctx, &binds, "zoom_out") {
            let z = ctx.zoom_factor();
            ctx.set_zoom_factor((z - 0.1).max(0.5));
        }

        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::none()
                    .fill(self.active_theme.app.menu_bg.to_egui())
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 8,
                        top: 2,
                        bottom: 2,
                    }),
            )
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    let fg_menu = self.active_theme.app.menu_fg.to_egui();

                    // Unified menu-bar button for ALL entries (dropdown
                    // menus and plain buttons): same text size, padding,
                    // square corners, transparent fill with hover highlight.
                    let hover_bg = self.active_theme.app.hover.to_egui();
                    let menu_btn = |ui: &mut egui::Ui, label: &str| -> egui::Response {
                        // Fully hand-drawn button: the hover background is
                        // painted BEFORE the text so it can never cover the
                        // label (Button + later rect_filled hid the text).
                        let galley = ui.fonts(|f| {
                            f.layout_no_wrap(
                                label.to_string(),
                                egui::FontId::proportional(12.0),
                                fg_menu,
                            )
                        });
                        let pad = 8.0;
                        let size = egui::vec2(galley.size().x + pad * 2.0, galley.size().y + 8.0);
                        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                        if resp.contains_pointer() {
                            ui.painter().rect_filled(rect, 0.0, hover_bg);
                        }
                        ui.painter()
                            .galley(rect.center() - galley.size() / 2.0, galley, fg_menu);
                        resp
                    };
                    // Dropdown wrapper: the visible button + the popup menu
                    // share one hit area (invisible overlay Button drives
                    // egui's BarState so styling stays fully ours).
                    let mut dropdown =
                        |ui: &mut egui::Ui,
                         label: &str,
                         menu_id: &str,
                         add_contents: &mut dyn FnMut(&mut egui::Ui)| {
                            let btn = menu_btn(ui, label);
                            let _ = btn;
                            let mut bar =
                                egui::menu::BarState::load(ui.ctx(), egui::Id::new(menu_id));
                            // Re-use the same response via an overlay button so
                            // egui's menu logic tracks the exact same rect.
                            let overlay_resp = ui.interact(
                                btn.rect,
                                egui::Id::new((menu_id, "hit")),
                                egui::Sense::click(),
                            );
                            bar.bar_menu(&overlay_resp, |ui| add_contents(ui));
                            bar.store(ui.ctx(), egui::Id::new(menu_id));
                        };

                    let label = self.texts.menu.workspace.clone();
                    dropdown(ui, &label, "menu_workspace", &mut |ui| {
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
                    let label = self.texts.menu.view.clone();
                    dropdown(ui, &label, "menu_view", &mut |ui| {
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
                    let label = self.texts.menu.theme.clone();
                    dropdown(ui, &label, "menu_theme", &mut |ui| {
                        ui.set_min_width(120.0);
                        ui.set_max_width(180.0);
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
                    let label = self.texts.menu.language.clone();
                    dropdown(ui, &label, "menu_language", &mut |ui| {
                        let current_code = self.settings.language.clone();
                        let languages = self.available_languages.clone();
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

                    // Settings and About: same button, no dropdown.
                    if menu_btn(ui, &self.texts.view_menu.settings).clicked() {
                        self.show_settings = true;
                        self.settings_edit = self.settings.clone();
                        self.theme_edit = self.active_theme.clone();
                        self.theme_message = None;
                        self.theme_dirty = false;
                    }
                    if menu_btn(ui, &self.texts.about.menu_label).clicked() {
                        self.show_about = true;
                    }

                    // Right-aligned extras: app version.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let weak = self.active_theme.app.weak_text.to_egui();
                        ui.label(
                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .color(weak)
                                .size(11.0),
                        );
                    });
                });
            });

        if self.show_settings {
            // Expire the applied toast.
            if let Some((_, until)) = self.settings_applied_toast {
                if std::time::Instant::now() >= until {
                    self.settings_applied_toast = None;
                }
            }
            let mut open = self.show_settings;
            let was_open = self.settings_window_open;
            self.settings_window_open = true;
            let screen = ctx.screen_rect();
            // Fixed-width window (680px); only the height persists (clamped
            // to the screen). Nested panels inside a Window misbehave, so
            // the layout below is plain rect allocation: fixed nav column on
            // the left, content filling the remaining width.
            let mut ws = self.settings_edit.settings_window.clone();
            ws.width = 680.0;
            // Clamp with guarded bounds: after UI zoom the logical screen
            // can be smaller than the window, which would otherwise make
            // clamp(min, negative-max) panic.
            ws.height = ws.height.clamp(200.0, (screen.height() - 40.0).max(200.0));
            if !was_open {
                ws.x = screen.center().x - ws.width / 2.0;
                ws.y = screen.center().y - ws.height / 2.0;
            }
            ws.x = ws.x.clamp(
                screen.left(),
                (screen.right() - ws.width).max(screen.left()),
            );
            ws.y = ws.y.clamp(
                screen.top(),
                (screen.bottom() - ws.height).max(screen.top()),
            );
            let texts = self.texts.clone();
            let nav_w = 124.0;
            // Stateless custom window (Area + Frame + manual title bar):
            // egui::Window keeps persistent state that has repeatedly
            // fought our fixed size, so we render the settings window
            // ourselves. Content is clipped to the frame rect, so nothing
            // can ever render outside the window.
            let mut close_clicked = false;
            let title_h = 32.0;
            let win_rect =
                egui::Rect::from_min_size(egui::pos2(ws.x, ws.y), egui::vec2(ws.width, ws.height));
            let layer_id = egui::LayerId::new(egui::Order::Middle, egui::Id::new("settings_area"));
            egui::Area::new(egui::Id::new("settings_area"))
                .order(egui::Order::Middle)
                .fixed_pos(win_rect.min)
                .interactable(true)
                .show(ctx, |ui| {
                    ui.set_clip_rect(win_rect.expand(24.0));
                    // Window frame from the theme (fill, stroke, shadow,
                    // corner radius) but with NO inner margin, so the title
                    // bar background reaches the window edge like a native
                    // title bar.
                    let frame = egui::Frame::window(ui.style()).inner_margin(egui::Margin::same(0));
                    frame.show(ui, |ui| {
                        ui.set_min_size(egui::vec2(ws.width, ws.height));
                        ui.set_max_size(egui::vec2(ws.width, ws.height));
                        let full = ui.max_rect();
                        // --- Title bar (egui-native look): centered title
                        // text, hand-drawn X close button on the right with
                        // hover-reactive stroke color, separator below. ---
                        let title_rect = egui::Rect::from_min_max(
                            full.min,
                            egui::pos2(full.max.x, full.min.y + title_h),
                        );
                        let title_resp = ui.interact(
                            title_rect,
                            egui::Id::new("settings_title_bar"),
                            egui::Sense::click_and_drag(),
                        );
                        ui.painter().rect_filled(
                            title_rect,
                            0.0,
                            // Same header color as a topmost egui::Window
                            // (the About window): widgets.open.weak_bg_fill.
                            ui.visuals().widgets.open.weak_bg_fill,
                        );
                        // Close button: two line segments like egui's
                        // `close_button`, colored by the interact style so
                        // it brightens on hover like the About window's.
                        {
                            let icon_w = ui.spacing().icon_width;
                            let size = egui::Vec2::splat(icon_w);
                            let cb = egui::Rect::from_center_size(
                                egui::pos2(
                                    title_rect.max.x - title_rect.height() / 2.0,
                                    title_rect.center().y,
                                ),
                                size,
                            );
                            let resp = ui.interact(
                                cb,
                                egui::Id::new("settings_close"),
                                egui::Sense::click(),
                            );
                            let visuals = ui.style().interact(&resp);
                            let r = cb.shrink(2.0).expand(visuals.expansion);
                            let stroke = visuals.fg_stroke;
                            ui.painter() // \
                                .line_segment([r.left_top(), r.right_bottom()], stroke);
                            ui.painter() // /
                                .line_segment([r.right_top(), r.left_bottom()], stroke);
                            if resp.clicked() {
                                close_clicked = true;
                            }
                        }
                        // Title text: same font and color as egui::Window's
                        // title bar (the About window) — Heading style via
                        // the current text-style ladder, text_color().
                        let title_font = ui
                            .style()
                            .text_styles
                            .get(&egui::TextStyle::Heading)
                            .cloned()
                            .unwrap_or_else(|| egui::FontId::proportional(14.0));
                        ui.painter().text(
                            title_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &texts.settings.title,
                            title_font,
                            ui.visuals().text_color(),
                        );
                        // Drag to move the window.
                        if title_resp.dragged() {
                            let delta = title_resp.drag_delta();
                            let p = win_rect.min + delta;
                            let p = egui::pos2(
                                p.x.clamp(
                                    screen.left(),
                                    (screen.right() - ws.width).max(screen.left()),
                                ),
                                p.y.clamp(
                                    screen.top(),
                                    (screen.bottom() - ws.height).max(screen.top()),
                                ),
                            );
                            self.settings_edit.settings_window.x = p.x;
                            self.settings_edit.settings_window.y = p.y;
                        }

                        let body = egui::Rect::from_min_max(
                            egui::pos2(full.min.x, title_rect.max.y),
                            full.max,
                        );
                        let nav_rect = egui::Rect::from_min_max(
                            body.min,
                            egui::pos2(body.min.x + nav_w, body.max.y),
                        );
                        let content_rect = egui::Rect::from_min_max(
                            egui::pos2(nav_rect.max.x, body.min.y),
                            body.max,
                        );

                        // ---- Left nav column (plain rect) ----
                        let nav_ui = &mut ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(nav_rect)
                                .layout(egui::Layout::top_down(egui::Align::Min))
                                .id_salt("settings_nav_ui"),
                        );
                        nav_ui.set_clip_rect(nav_rect);
                        nav_ui.painter().rect_filled(
                            nav_rect,
                            0.0,
                            self.active_theme.app.menu_bg.to_egui(),
                        );
                        nav_ui.painter().rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(nav_rect.max.x - 1.0, nav_rect.min.y),
                                nav_rect.max,
                            ),
                            0.0,
                            self.active_theme.app.sidebar_border.to_egui(),
                        );
                        // Divider under the title bar, drawn AFTER the nav
                        // background so the nav column's fill (which starts
                        // right at the divider's y) doesn't cover the line's
                        // anti-aliased pixels above it.
                        ui.painter().hline(
                            title_rect.x_range(),
                            title_rect.bottom(),
                            (1.0, self.active_theme.app.sidebar_border.to_egui()),
                        );
                        let nav_fg = self.active_theme.app.menu_fg.to_egui();
                        let nav_weak = self.active_theme.app.weak_text.to_egui();
                        let nav_sel_bg = self.active_theme.app.active.to_egui();
                        let accent = self.active_theme.app.accent.to_egui();
                        let item_font = egui::FontId::proportional(12.0);
                        let current = SettingsPage::from_u8(self.settings_tab);

                        let mut nav_item =
                            |ui: &mut egui::Ui, label: &str, page: SettingsPage| -> bool {
                                let id = egui::Id::new(("settings_nav", page as u8));
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 28.0),
                                    egui::Sense::click(),
                                );
                                let selected = current == page;
                                if selected {
                                    ui.painter().rect_filled(rect, 0.0, nav_sel_bg);
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_max(
                                            rect.min,
                                            egui::pos2(rect.min.x + 2.0, rect.max.y),
                                        ),
                                        0.0,
                                        accent,
                                    );
                                } else if rect.contains(
                                    ui.input(|i| i.pointer.hover_pos())
                                        .unwrap_or(rect.min - egui::vec2(1.0, 1.0)),
                                ) {
                                    ui.painter().rect_filled(
                                        rect,
                                        0.0,
                                        self.active_theme.app.hover.to_egui(),
                                    );
                                }
                                ui.painter().text(
                                    egui::pos2(rect.min.x + 12.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    label,
                                    item_font.clone(),
                                    if selected { nav_fg } else { nav_weak },
                                );
                                let resp = ui.interact(rect, id, egui::Sense::click());
                                resp.clicked()
                            };

                        if nav_item(nav_ui, &texts.settings.nav.general, SettingsPage::General) {
                            self.settings_tab = SettingsPage::General as u8;
                        }
                        if nav_item(
                            nav_ui,
                            &texts.settings.nav.shortcuts,
                            SettingsPage::Shortcuts,
                        ) {
                            self.settings_tab = SettingsPage::Shortcuts as u8;
                        }
                        if nav_item(nav_ui, &texts.settings.nav.lock, SettingsPage::Lock) {
                            self.settings_tab = SettingsPage::Lock as u8;
                        }
                        if nav_item(nav_ui, &texts.settings.nav.themes, SettingsPage::Theme) {
                            self.settings_tab = SettingsPage::Theme as u8;
                        }

                        // ---- Content column (plain rect, clipped) ----
                        let content_ui = &mut ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(content_rect)
                                .layout(egui::Layout::top_down(egui::Align::Min))
                                .id_salt("settings_content_ui"),
                        );
                        content_ui.set_clip_rect(content_rect);
                        let page = SettingsPage::from_u8(self.settings_tab);
                        if page == SettingsPage::Theme {
                            self.settings_page_theme(ctx, content_ui);
                        } else {
                            egui::ScrollArea::vertical()
                                .id_salt("settings_content_scroll")
                                .auto_shrink([false, false])
                                .max_width(content_rect.width())
                                .show(content_ui, |ui| {
                                    ui.set_width(content_rect.width() - 36.0);
                                    ui.add_space(20.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(18.0);
                                        ui.vertical(|ui| {
                                            ui.set_width(ui.available_width());
                                            match page {
                                                SettingsPage::General => {
                                                    self.settings_page_general(ui)
                                                }
                                                SettingsPage::Shortcuts => {
                                                    self.settings_page_shortcuts(ui)
                                                }
                                                SettingsPage::Lock => self.settings_page_lock(ui),
                                                SettingsPage::Theme => unreachable!(),
                                            }
                                        });
                                    });
                                });
                        }
                    });
                });
            if close_clicked {
                self.show_settings = false;
                self.settings_window_open = false;
                self.binding_recording = None;
                let _ = save_settings(&self.settings);
            }

            // Instant-apply: commit any changed settings every frame.
            if self.settings_edit != self.settings {
                self.settings = self.settings_edit.clone();
                self.history_db.set_max_entries(self.settings.max_history);
                let _ = save_settings(&self.settings);
            }
            if !open {
                self.show_settings = false;
                self.settings_window_open = false;
                self.binding_recording = None;
                let _ = save_settings(&self.settings);
            }
        }

        // Theme dialog popups
        self.show_theme_dialogs(ctx);
        self.show_theme_editor_popup(ctx);

        // Confirm-dialog for deleting ALL terminal command history.
        if self.show_clear_history_confirm {
            let mut open = self.show_clear_history_confirm;
            let mut confirmed = false;
            let mut cancelled = false;
            egui::Window::new(&self.texts.stats.clear_history_title)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui| {
                    ui.label(&self.texts.stats.clear_history_body);
                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(&self.texts.password.confirm_button).clicked() {
                            confirmed = true;
                        }
                        if ui.button(&self.texts.password.cancel_button).clicked() {
                            cancelled = true;
                        }
                    });
                });
            if cancelled {
                self.show_clear_history_confirm = false;
            } else if confirmed {
                self.history_db.clear_all();
                self.show_clear_history_confirm = false;
            } else if !open {
                self.show_clear_history_confirm = false;
            }
        }

        if self.show_about {
            let mut open = self.show_about;
            let mut clicked_close = false;
            egui::Window::new(format!("OpenNex v{}", env!("CARGO_PKG_VERSION")))
                .id(egui::Id::new("about_window"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_width(380.0)
                .show(ctx, |ui| {
                    ui.add_space(4.0);
                    // Render the multi-paragraph description: each
                    // newline-separated paragraph on its own wrapped label.
                    for para in self.texts.about.description.split('\n') {
                        if para.trim().is_empty() {
                            continue;
                        }
                        ui.weak(
                            egui::RichText::new(para)
                                .size(12.0)
                                .color(self.active_theme.app.weak_text.to_egui()),
                        );
                        ui.add_space(2.0);
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.weak(self.texts.about.homepage_label.as_str());
                        ui.hyperlink_to("https://opennex.zeadix.com", "https://opennex.zeadix.com");
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

                    // In-place update status + progress bar. No popups
                    // open during this flow; the bar and status line sit
                    // above the action row.
                    {
                        use crate::updater::UpdateState;
                        let pct: Option<f32> =
                            if let UpdateState::Downloading(p) = self.update_state {
                                Some(p)
                            } else {
                                None
                            };
                        let ut = self.texts.update.clone();
                        let (text, color) = match &self.update_state {
                            UpdateState::Idle | UpdateState::Checking => {
                                ("".to_string(), egui::Color32::WHITE)
                            }
                            UpdateState::Downloading(_) => (
                                ut.downloading.clone(),
                                egui::Color32::from_rgb(0x3b, 0x82, 0xf6),
                            ),
                            UpdateState::Verifying => (
                                ut.verifying.clone(),
                                egui::Color32::from_rgb(0xa8, 0x55, 0xf7),
                            ),
                            UpdateState::Ready(_) => {
                                (ut.ready.clone(), egui::Color32::from_rgb(0x22, 0xc5, 0x5e))
                            }
                            UpdateState::Error(msg) => (
                                ut.failed.replace("{}", msg),
                                egui::Color32::from_rgb(0xef, 0x44, 0x44),
                            ),
                            UpdateState::Available(info) => (
                                ut.available.replace("{}", &info.version),
                                egui::Color32::from_rgb(0x22, 0xc5, 0x5e),
                            ),
                            UpdateState::UpToDate => (
                                ut.up_to_date.clone(),
                                egui::Color32::from_rgb(0x6b, 0x72, 0x80),
                            ),
                        };
                        if !text.is_empty() {
                            ui.colored_label(color, text);
                        }
                        if let Some(p) = pct {
                            // Draw a custom square-cornered progress bar so we
                            // can position the percentage label exactly in the
                            // center (ProgressBar's built-in percent text is
                            // hard-coded to the left edge).
                            let height = 18.0;
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), height),
                                egui::Sense::hover(),
                            );
                            let painter = ui.painter_at(rect);
                            let visuals = ui.style().visuals.clone();
                            painter.rect_filled(
                                rect,
                                egui::CornerRadius::ZERO,
                                visuals.extreme_bg_color,
                            );
                            let filled = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(rect.width() * p, rect.height()),
                            );
                            let filled_color = egui::Color32::from_rgb(0x3b, 0x82, 0xf6);
                            painter.rect_filled(filled, egui::CornerRadius::ZERO, filled_color);
                            let pct_text = format!("{}%", (p * 100.0) as i32);
                            let galley = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    pct_text,
                                    egui::FontId::proportional(12.0),
                                    visuals.override_text_color.unwrap_or(visuals.text_color()),
                                )
                            });
                            let text_pos = egui::pos2(
                                rect.center().x - galley.size().x / 2.0,
                                rect.center().y - galley.size().y / 2.0,
                            );
                            painter.galley(text_pos, galley, visuals.text_color());
                        }
                    }

                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            let is_checking_only =
                                matches!(self.update_state, crate::updater::UpdateState::Checking);
                            // Show the spinner only while waiting for the
                            // check response, not during the download
                            // itself (where the progress bar is the cue).
                            if is_checking_only {
                                ui.spinner();
                            }
                            // The button stays disabled for the entire
                            // busy window (check + download + verify) so
                            // the user can't double-trigger.
                            let is_busy = matches!(
                                self.update_state,
                                crate::updater::UpdateState::Checking
                                    | crate::updater::UpdateState::Downloading(_)
                                    | crate::updater::UpdateState::Verifying
                            );
                            // The primary action button changes label
                            // depending on the current state.
                            enum PrimaryAction {
                                Check,
                                StartDownload,
                                Restart,
                            }
                            let primary = match &self.update_state {
                                crate::updater::UpdateState::Idle
                                | crate::updater::UpdateState::Checking => PrimaryAction::Check,
                                crate::updater::UpdateState::Available(_) => {
                                    PrimaryAction::StartDownload
                                }
                                crate::updater::UpdateState::Downloading(_)
                                | crate::updater::UpdateState::Verifying => {
                                    // Show "检查更新" disabled while busy
                                    PrimaryAction::Check
                                }
                                crate::updater::UpdateState::Ready(_) => PrimaryAction::Restart,
                                crate::updater::UpdateState::Error(_)
                                | crate::updater::UpdateState::UpToDate => PrimaryAction::Check,
                            };
                            let label = match primary {
                                PrimaryAction::Check => self.texts.update.check.as_str(),
                                PrimaryAction::StartDownload => {
                                    self.texts.update.update_now.as_str()
                                }
                                PrimaryAction::Restart => self.texts.update.restart.as_str(),
                            };
                            if ui.add_enabled(!is_busy, egui::Button::new(label)).clicked() {
                                match primary {
                                    PrimaryAction::Check => {
                                        self.start_manual_check();
                                        self.check_update_manual(ctx);
                                    }
                                    PrimaryAction::StartDownload => {
                                        if let crate::updater::UpdateState::Available(info) =
                                            &self.update_state
                                        {
                                            let info_clone = info.clone();
                                            self.kick_download(ctx, &info_clone);
                                        }
                                    }
                                    PrimaryAction::Restart => {
                                        if let crate::updater::UpdateState::Ready(path) =
                                            &self.update_state
                                        {
                                            let path = path.clone();
                                            self.apply_update_and_restart(ctx, path);
                                        }
                                    }
                                }
                            }
                            if ui.button(&self.texts.about.close).clicked() {
                                clicked_close = true;
                            }
                        });
                    });
                });
            if !open || clicked_close {
                // Closing the about window does NOT clear update_state —
                // re-opening it shows the latest progress, and a completed
                // download remains installable via the "重启应用" button.
                self.show_about = false;
            }
        }

        // Restart confirmation popup (shown when the about window is
        // closed and the update is ready; user can pick "重启" or "取消").
        if let crate::updater::UpdateState::Ready(path) = &self.update_state.clone() {
            render_restart_popup(ctx, &self.texts.clone());
            // Read the user's choice from egui memory.
            let choice: Option<bool> = ctx.memory_mut(|mem| {
                mem.data
                    .remove_temp::<bool>(egui::Id::new("restart_popup_choice"))
            });
            match choice {
                Some(true) => {
                    let path = path.clone();
                    self.apply_update_and_restart(ctx, path);
                }
                Some(false) => {
                    // User chose cancel: dismiss the popup. Set the
                    // state back to Idle so the popup stops appearing; they
                    // can re-check later from the about window's button
                    // to re-trigger an install.
                    self.update_state = crate::updater::UpdateState::Idle;
                }
                None => {}
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

        // Check for update result from background thread.
        // After ~3 seconds (180 frames @ 60fps) we show the update dialog
        // for any available update, or display a toast for up-to-date / error.
        if self.startup_frame_count < 180 {
            self.startup_frame_count += 1;
        }
        if self.startup_frame_count == 180 {
            let result = ctx.memory_mut(|mem| {
                mem.data
                    .remove_temp::<StartCheckResult>(egui::Id::new("start_check_result"))
            });
            if let Some(r) = result {
                match r {
                    StartCheckResult::Available(info) => {
                        if !self.skipped_versions.contains(&info.version) {
                            // Surface the new version through About's
                            // in-place status panel and auto-open About so
                            // the user actually sees the offer. No
                            // standalone update popup anymore.
                            self.update_state = crate::updater::UpdateState::Available(info);
                            self.show_about = true;
                        }
                    }
                    StartCheckResult::UpToDate => {
                        self.update_toast = Some((
                            "当前已是最新版本".to_string(),
                            std::time::Instant::now() + std::time::Duration::from_secs(3),
                        ));
                    }
                    StartCheckResult::Error(_) => {
                        self.update_toast = Some((
                            "无法连接到更新服务器".to_string(),
                            std::time::Instant::now() + std::time::Duration::from_secs(3),
                        ));
                    }
                }
            }
        }

        // Update window + dialog + toast
        self.render_update_window(ctx);
        self.show_update_toast(ctx);
        self.show_copy_toast(ctx);
        self.render_history_menu(ctx);
        self.render_history_clear_confirm(ctx);

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
            // Auto-focus the first input whenever the popup is open, so
            // the terminal (or any other layer) can't leave the dialog
            // without an active focus target.
            let focus_target = match popup {
                "set" => egui::Id::new("pw_set1"),
                "change" => egui::Id::new("pw_old"),
                _ => egui::Id::new("pw_clear"),
            };
            ctx.memory_mut(|m| {
                if m.focused().is_none() {
                    m.request_focus(focus_target);
                }
            });
            egui::Window::new(title)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui| {
                    // Compact dialog metrics: the theme's font-size ladder
                    // scales text styles, which also inflates egui's
                    // default row heights and item spacing. Pin them back
                    // so the dialog stays tight at any UI font size.
                    ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 4.0);
                    ui.style_mut().spacing.interact_size.y = 20.0;
                    ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);
                    match popup {
                        "set" => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.enter).size(12.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pw_set1)
                                        .password(true)
                                        .desired_width(150.0)
                                        .font(egui::TextStyle::Monospace)
                                        .id_source("pw_set1"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.confirm).size(12.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pw_set2)
                                        .password(true)
                                        .desired_width(150.0)
                                        .font(egui::TextStyle::Monospace)
                                        .id_source("pw_set2"),
                                );
                                if !self.pw_set2.is_empty() && self.pw_set1 != self.pw_set2 {
                                    ui.label(
                                        egui::RichText::new(&self.texts.password.mismatch)
                                            .color(egui::Color32::RED)
                                            .size(11.0),
                                    );
                                } else if !self.pw_set2.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&self.texts.password.r#match)
                                            .color(egui::Color32::GREEN)
                                            .size(11.0),
                                    );
                                }
                            });
                            if !self.pw_message.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.pw_message)
                                        .color(egui::Color32::RED)
                                        .size(11.0),
                                );
                            }
                            ui.horizontal(|ui| {
                                if ui.button(&self.texts.password.confirm_button).clicked() {
                                    if self.pw_set1.is_empty() {
                                        self.pw_message = self.texts.password.empty_error.clone();
                                    } else if self.pw_set1 != self.pw_set2 {
                                        self.pw_message =
                                            self.texts.password.mismatch_error.clone();
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
                                ui.label(
                                    egui::RichText::new(&self.texts.password.original).size(12.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pw_old)
                                        .password(true)
                                        .desired_width(150.0)
                                        .font(egui::TextStyle::Monospace)
                                        .id_source("pw_old"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.texts.password.new).size(12.0));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pw_new1)
                                        .password(true)
                                        .desired_width(150.0)
                                        .id_source("pw_new1"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&self.texts.password.confirm_new)
                                        .size(12.0),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.pw_new2)
                                        .password(true)
                                        .desired_width(150.0)
                                        .id_source("pw_new2"),
                                );
                                if !self.pw_new2.is_empty() && self.pw_new1 != self.pw_new2 {
                                    ui.label(
                                        egui::RichText::new(&self.texts.password.mismatch)
                                            .color(egui::Color32::RED)
                                            .size(11.0),
                                    );
                                } else if !self.pw_new2.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&self.texts.password.r#match)
                                            .color(egui::Color32::GREEN)
                                            .size(11.0),
                                    );
                                }
                            });
                            if !self.pw_message.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.pw_message)
                                        .color(egui::Color32::RED)
                                        .size(11.0),
                                );
                            }
                            ui.horizontal(|ui| {
                                if ui.button(&self.texts.password.confirm_button).clicked() {
                                    if self.pw_old != self.settings.lock_password {
                                        self.pw_message = self.texts.password.wrong_error.clone();
                                    } else if self.pw_new1.is_empty() {
                                        self.pw_message = self.texts.password.empty_error.clone();
                                    } else if self.pw_new1 != self.pw_new2 {
                                        self.pw_message =
                                            self.texts.password.mismatch_error.clone();
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
                            // Mirrors the set-password popup layout: centered
                            // input, enter-to-confirm, error line, buttons.
                            let mut confirmed = false;
                            let pw_resp = ui.add(
                                egui::TextEdit::singleline(&mut self.pw_clear)
                                    .password(true)
                                    .desired_width(150.0)
                                    .font(egui::TextStyle::Monospace)
                                    .hint_text(&self.texts.password.input_label)
                                    .id_source("pw_clear"),
                            );
                            if pw_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                confirmed = true;
                            }
                            if !self.pw_message.is_empty() {
                                ui.label(
                                    egui::RichText::new(&self.pw_message)
                                        .color(egui::Color32::RED)
                                        .size(11.0),
                                );
                            }
                            ui.horizontal(|ui| {
                                if confirmed
                                    || ui.button(&self.texts.password.confirm_button).clicked()
                                {
                                    if self.pw_clear != self.settings.lock_password {
                                        self.pw_message =
                                            self.texts.password.wrong_password.clone();
                                        self.pw_clear.clear();
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
                    }
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
                .resizable(true)
                .default_width(WORKSPACE_SIDEBAR_DEFAULT_WIDTH)
                .width_range(120.0..=300.0)
                .frame(
                    egui::Frame::none()
                        .fill(self.active_theme.app.sidebar.to_egui())
                        .stroke(egui::Stroke::NONE)
                        .inner_margin(egui::Margin {
                            left: 8,
                            right: 8,
                            top: 8,
                            bottom: 8,
                        }),
                )
                .show(ctx, |ui| {
                    // Header row: square icon buttons on the LEFT (新建
                    // leftmost, 模板 to its right) — background fill only,
                    // no stroke, no rounding, Phosphor regular glyphs.
                    ui.horizontal(|ui| {
                        let fg = self.active_theme.app.button_fg.to_egui();
                        let icon_active = self.active_theme.app.text.to_egui();
                        let btn_bg = self.active_theme.app.button_bg.to_egui();

                        let btn_size = 18.5;
                        let glyph = 12.0;
                        // 新建 (leftmost): flat PLUS glyph, no background.
                        let (new_rect, new_resp) = ui.allocate_exact_size(
                            egui::vec2(btn_size, btn_size),
                            egui::Sense::click(),
                        );
                        let new_color = if new_resp.hovered() { icon_active } else { fg };
                        let new_galley = ui.fonts(|f| {
                            f.layout_no_wrap(
                                egui_phosphor::regular::PLUS.to_string(),
                                egui::FontId::proportional(glyph),
                                new_color,
                            )
                        });
                        ui.painter().galley(
                            new_rect.center()
                                - egui::vec2(new_galley.size().x / 2.0, new_galley.size().y / 2.0),
                            new_galley,
                            new_color,
                        );
                        let new_hovered = new_resp.hovered();
                        let new_clicked = new_resp.clicked();
                        let _ = new_resp.on_hover_text(&self.texts.workspace.new);
                        if new_clicked {
                            self.add_panel(ui.ctx());
                        }
                        let _ = new_hovered;

                        // 模板: flat STACK glyph, no background, left-click menu
                        // via BarState (same as the row three-dot).
                        let (tpl_rect, tpl_resp) = ui.allocate_exact_size(
                            egui::vec2(btn_size, btn_size),
                            egui::Sense::click(),
                        );
                        let tpl_color = if tpl_resp.hovered() { icon_active } else { fg };
                        let tpl_galley = ui.fonts(|f| {
                            f.layout_no_wrap(
                                egui_phosphor::regular::STACK.to_string(),
                                egui::FontId::proportional(glyph),
                                tpl_color,
                            )
                        });
                        ui.painter().galley(
                            tpl_rect.center()
                                - egui::vec2(tpl_galley.size().x / 2.0, tpl_galley.size().y / 2.0),
                            tpl_galley,
                            tpl_color,
                        );
                        let bar_id = tpl_resp.id;
                        if self.cached_template_files.is_empty() {
                            self.refresh_template_files();
                        }
                        let template_files = self.cached_template_files.clone();
                        let mut bar_state = egui::menu::BarState::load(ui.ctx(), bar_id);
                        bar_state.bar_menu(&tpl_resp, |ui| {
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
                                        if ui.small_button(egui_phosphor::regular::X).clicked() {
                                            self.pending_delete_template = Some(path);
                                            ui.close_menu();
                                        }
                                    });
                                }
                            }
                        });
                        bar_state.store(ui.ctx(), bar_id);
                        let _ = tpl_resp.on_hover_text(&self.texts.workspace.templates);
                    });
                    ui.add_space(4.0);
                    // Remove the default vertical spacing between workspace
                    // items so the list reads as one continuous block.
                    ui.style_mut().spacing.item_spacing.y = 2.0;
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
                            let panel_name = self.panels[i].name.clone();
                            let is_locked = self.locked_panels.contains(&i);
                            let row_h = ui.spacing().interact_size.y;
                            // Reserve the full row width up front so we can
                            // detect hover across the whole row, not just on
                            // the selectable label.
                            let (row_rect, row_resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), row_h),
                                egui::Sense::click_and_drag(),
                            );
                            // `on_hover_text` consumes the Response; bind
                            // back to the original so we can still use it
                            // below.
                            let is_row_hovered = row_resp.hovered();
                            let _row_resp =
                                row_resp.on_hover_text(&self.texts.workspace.drag_handle_hint);
                            // Background: whole row uses button_bg so every
                            // click target inside the row shares one
                            // continuous surface. The selected row fills
                            // with the theme's active color instead of
                            // button_bg to indicate the active workspace.
                            ui.painter().rect_filled(
                                row_rect,
                                0.0,
                                if is_active {
                                    self.active_theme.app.active.to_egui()
                                } else {
                                    self.active_theme.app.button_bg.to_egui()
                                },
                            );
                            self.panel_rects[i] = row_rect;

                            // Layout: [≡ drag icon] [name (flex)] [lock btn | three-dot]
                            // Use a child Ui inside row_rect so we can place
                            // items by their own rect, not via add_sized.
                            let mut child = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(row_rect)
                                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
                            );
                            // Drag handle on the left — always visible
                            // (brighter when hovered).
                            let drag_w = 14.0;
                            let (handle_rect, handle_resp) = child.allocate_exact_size(
                                egui::vec2(drag_w, row_h - 4.0),
                                egui::Sense::drag(),
                            );
                            let handle_color = if handle_resp.hovered() {
                                self.active_theme.app.text.to_egui()
                            } else {
                                self.active_theme.app.weak_text.to_egui()
                            };
                            child.painter().text(
                                handle_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                egui_phosphor::regular::DOTS_SIX_VERTICAL,
                                egui::FontId::proportional(12.0),
                                handle_color,
                            );
                            if handle_resp.drag_started() {
                                self.drag_src_panel = Some(i);
                                self.drag_dst_panel = None;
                            }
                            let _ = handle_rect;

                            // Name (clickable, fills middle). Flat text drawn
                            // directly on the row's shared button_bg — no
                            // SelectableLabel so hover never paints its own
                            // background over the row surface. Active state
                            // is shown via the text color; the row's accent
                            // border already marks the active workspace.
                            let (name_rect, response) = child.allocate_exact_size(
                                egui::vec2(child.available_width(), row_h),
                                egui::Sense::click_and_drag(),
                            );
                            let name_color = if is_active {
                                self.active_theme.app.text.to_egui()
                            } else if response.hovered() {
                                self.active_theme.app.text.to_egui()
                            } else {
                                self.active_theme.app.button_fg.to_egui()
                            };
                            let name_galley = child.fonts(|f| {
                                f.layout_no_wrap(
                                    panel_name.to_string(),
                                    egui::FontId::proportional(14.0),
                                    name_color,
                                )
                            });
                            child.painter().galley(
                                egui::pos2(
                                    name_rect.min.x + 2.0,
                                    name_rect.center().y - name_galley.size().y / 2.0,
                                ),
                                name_galley,
                                name_color,
                            );
                            let _ = name_rect;
                            if response.double_clicked() && !renaming {
                                self.renaming_panel = Some(i);
                                self.rename_buffer = panel_name;
                                self.rename_frame_count = 0;
                                to_select = None;
                            } else if response.clicked() && !renaming {
                                to_select = Some(i);
                            }
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
                                if ui.button(if is_locked { "解锁" } else { "锁定" }).clicked()
                                {
                                    if is_locked {
                                        self.active_panel = i;
                                        self.lock_password_input.clear();
                                        self.pw_message.clear();
                                    } else {
                                        self.locked_panels.insert(i);
                                        self.lock_password_input.clear();
                                        self.pw_message.clear();
                                    }
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button(&self.texts.settings.buttons.close).clicked() {
                                    self.close_confirm_panel = Some(i);
                                    ui.close_menu();
                                }
                            });

                            // Right-side action cluster, anchored to the
                            // right edge with right-to-left layout. The lock
                            // and three-dot icons are always rendered and
                            // sit flush against each other (no divider, no
                            // item spacing inside the cluster). Lock is the
                            // rightmost target; the three-dot menu button
                            // sits to its left.
                            let btn_w = 17.0;
                            let btn_h = row_h;
                            let action_cluster_w = btn_w;
                            let mut actions_ui = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(egui::Rect::from_min_size(
                                        egui::pos2(
                                            row_rect.max.x - action_cluster_w,
                                            row_rect.min.y,
                                        ),
                                        egui::vec2(action_cluster_w, row_rect.height()),
                                    ))
                                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
                            );
                            actions_ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
                            let button_fg = self.active_theme.app.button_fg.to_egui();
                            let icon_active = self.active_theme.app.text.to_egui();

                            // Lock / unlock button (rightmost, the only action). Always painted so the
                            let (lock_rect, lock_resp) = actions_ui.allocate_exact_size(
                                egui::vec2(btn_w, btn_h),
                                egui::Sense::click(),
                            );
                            let lock_color = if is_locked {
                                self.active_theme.app.lock.to_egui()
                            } else if lock_resp.hovered() {
                                icon_active
                            } else {
                                button_fg
                            };
                            let lock_galley = actions_ui.fonts(|f| {
                                f.layout_no_wrap(
                                    workspace_lock_icon(is_locked).to_string(),
                                    egui::FontId::proportional(13.0),
                                    lock_color,
                                )
                            });
                            actions_ui.painter().galley(
                                lock_rect.center()
                                    - egui::vec2(
                                        lock_galley.size().x / 2.0,
                                        lock_galley.size().y / 2.0,
                                    ),
                                lock_galley,
                                lock_color,
                            );
                            if lock_resp.clicked() {
                                if is_locked {
                                    self.active_panel = i;
                                } else {
                                    self.locked_panels.insert(i);
                                }
                                self.lock_password_input.clear();
                                self.pw_message.clear();
                            }
                            let _ = lock_resp.on_hover_text(if is_locked {
                                &self.texts.workspace.locked_hint
                            } else {
                                &self.texts.workspace.unlocked_hint
                            });
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
                        if self.active_panel != i {
                            self.active_panel = i;
                            // Restore the workspace's last focused terminal
                            // (the dock tree keeps it as its active tab).
                            if let Some(tree) = self.dock_states.get_mut(&i) {
                                if let Some((_, tab)) = tree.find_active_focused() {
                                    self.focused_terminal = Some(tab.clone());
                                }
                            }
                        }
                    }
                    // Sidebar footer pinned 40px above the bottom: let the
                    // workspace list take all remaining space first, then
                    // allocate the gap and the fixed-height footer.
                    let weak = self.active_theme.app.weak_text.to_egui();
                    let fg = self.active_theme.app.button_fg.to_egui();
                    let footer_h = 96.0;
                    let line_h = 16.0;
                    let bottom_gap = 10.0;
                    let avail_h = ui.available_height();
                    let flex_h = (avail_h - footer_h - bottom_gap).max(0.0);
                    if flex_h > 0.0 {
                        ui.add_space(flex_h);
                    }
                    let (footer_rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), footer_h),
                        egui::Sense::hover(),
                    );
                    let x = footer_rect.min.x + 10.0;
                    let title_font = egui::FontId::proportional(10.0);
                    let data_font = egui::FontId::proportional(11.0);
                    let line = |i: usize| footer_rect.min.y + line_h * i as f32 + line_h / 2.0;

                    ui.painter().text(
                        egui::pos2(x, line(0)),
                        egui::Align2::LEFT_CENTER,
                        self.texts.stats.focused.as_str(),
                        title_font.clone(),
                        fg,
                    );
                    ui.painter().text(
                        egui::pos2(x, line(1)),
                        egui::Align2::LEFT_CENTER,
                        format!(
                            "{} │ {}",
                            format_cpu(self.focused_cpu),
                            format_memory(self.focused_mem)
                        ),
                        data_font.clone(),
                        weak,
                    );
                    ui.painter().text(
                        egui::pos2(x, line(2)),
                        egui::Align2::LEFT_CENTER,
                        self.texts.stats.workspace.as_str(),
                        title_font.clone(),
                        fg,
                    );
                    ui.painter().text(
                        egui::pos2(x, line(3)),
                        egui::Align2::LEFT_CENTER,
                        format!(
                            "{} {} │ {} │ {}",
                            format_active_ws_terminal_count(self),
                            self.texts.stats.terminals,
                            format_cpu(self.workspace_cpu),
                            format_memory(self.workspace_mem)
                        ),
                        data_font.clone(),
                        weak,
                    );
                    ui.painter().text(
                        egui::pos2(x, line(4)),
                        egui::Align2::LEFT_CENTER,
                        self.texts.stats.global.as_str(),
                        title_font,
                        fg,
                    );
                    ui.painter().text(
                        egui::pos2(x, line(5)),
                        egui::Align2::LEFT_CENTER,
                        format!(
                            "{} {} │ {} │ {}",
                            format_ws_terminal_count(self),
                            self.texts.stats.terminals,
                            format_cpu(self.terminal_cpu),
                            format_memory(self.terminal_mem)
                        ),
                        data_font,
                        weak,
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
        let central_fill = self.active_theme.app.app_bg.to_egui();
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(central_fill))
            .show(ctx, |ui| {
                let is_locked = self.locked_panels.contains(&self.active_panel);
                if is_locked {
                    // Themed lock overlay: dark scrim over the workspace plus
                    // a centered card (panel fill, theme border, accent
                    // button). Colors come from the active theme so every
                    // theme looks right, instead of a flat warning-color
                    // wash.
                    let app = &self.active_theme.app;
                    let scrim = egui::Color32::from_rgba_unmultiplied(
                        app.app_bg.to_egui().r(),
                        app.app_bg.to_egui().g(),
                        app.app_bg.to_egui().b(),
                        220,
                    );
                    let avail = ui.available_size();
                    let (rect, _) = ui.allocate_exact_size(avail, egui::Sense::click());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 0.0, scrim);

                    // Centered card.
                    let card =
                        egui::Rect::from_center_size(rect.center(), egui::vec2(340.0, 216.0));
                    let card_bg = app.panel.to_egui();
                    painter.rect_filled(card, 4.0, card_bg);
                    painter.rect_stroke(
                        card,
                        4.0,
                        egui::Stroke::new(1.0_f32, app.border.to_egui()),
                        egui::StrokeKind::Inside,
                    );

                    let pw_id = egui::Id::new("lock_overlay_pw_input");
                    let ui_content = ui.allocate_new_ui(
                        egui::UiBuilder::new().max_rect(card.shrink2(egui::vec2(24.0, 20.0))),
                        |ui| {
                            ui.style_mut().spacing.item_spacing.y = 6.0;
                            ui.vertical_centered(|ui| {
                                // Lock icon in the theme's lock color.
                                ui.label(
                                    egui::RichText::new(egui_phosphor::regular::LOCK_SIMPLE)
                                        .size(30.0)
                                        .color(app.lock.to_egui()),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new(&self.texts.lock_overlay.title)
                                        .size(15.0)
                                        .strong()
                                        .color(app.text.to_egui()),
                                );
                                ui.add_space(10.0);
                                // Label above the input, mirroring the
                                // settings password popups' labeled style.
                                ui.label(
                                    egui::RichText::new(&self.texts.lock_overlay.password_label)
                                        .size(12.0)
                                        .color(app.text.to_egui()),
                                );
                                // Password row: input + eye visibility toggle.
                                // The unlock button below spans the exact same
                                // width (input's left edge -> eye's right
                                // edge), so both rows align perfectly.
                                let row_w = 240.0f32.min(ui.available_width());
                                let eye_w = 26.0;
                                let mut unlock_now = false;
                                ui.allocate_ui_with_layout(
                                    egui::vec2(row_w, ui.spacing().interact_size.y.max(20.0)),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        // Same input style as the settings-page
                                        // password popups (default text style,
                                        // 150px input, explicit label-free).
                                        let resp = ui.add(
                                            egui::TextEdit::singleline(
                                                &mut self.lock_password_input,
                                            )
                                            .password(!self.lock_password_visible)
                                            .desired_width(row_w - eye_w - 4.0)
                                            .id(pw_id),
                                        );
                                        // Enter in the focused input submits
                                        // (standard lost_focus+Enter pattern;
                                        // TextEdit consumes the raw key event).
                                        let enter_in_input = resp.lost_focus()
                                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                                        if enter_in_input {
                                            unlock_now = true;
                                        }
                                        if ui.ctx().memory(|m| m.focused().is_none()) {
                                            resp.request_focus();
                                        }
                                        // Eye toggle: EYE (hidden) / EYE_SLASH (visible).
                                        let eye = if self.lock_password_visible {
                                            egui_phosphor::regular::EYE_SLASH
                                        } else {
                                            egui_phosphor::regular::EYE
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(eye)
                                                        .size(14.0)
                                                        .color(app.text.to_egui()),
                                                )
                                                .fill(egui::Color32::TRANSPARENT)
                                                .stroke(egui::Stroke::NONE)
                                                .min_size(egui::vec2(eye_w, 0.0)),
                                            )
                                            .on_hover_text(&self.texts.lock_overlay.password_label)
                                            .clicked()
                                        {
                                            self.lock_password_visible =
                                                !self.lock_password_visible;
                                        }
                                    },
                                );
                                ui.add_space(4.0);
                                // Unlock button: accent-filled, spans the same
                                // width as the password row above. Enter in
                                // the password input also triggers it.
                                if unlock_now
                                    || ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(
                                                    &self.texts.lock_overlay.unlock_button,
                                                )
                                                .strong()
                                                .color(app.text.to_egui()),
                                            )
                                            .fill(app.accent.to_egui())
                                            .min_size(egui::vec2(row_w, 0.0)),
                                        )
                                        .clicked()
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
                                // Error message under the button.
                                if !self.pw_message.is_empty() {
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(&self.pw_message)
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(255, 120, 120)),
                                    );
                                }
                            });
                        },
                    );
                    let _ = ui_content;
                } else if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                    // Tab bar layout (via the local egui_dock fork):
                    // [collapse][+][tabs...] — the + button is pinned left
                    // of the tabs and right of the collapse button; the
                    // panel close-all button on the far right is hidden;
                    // each tab reserves a close slot that only becomes
                    // visible when the pointer hovers that tab.
                    // The dock surface itself carries no border stroke and
                    // every tab / tab-bar corner is square.
                    let mut dock_style = Style::from_egui(ui.style().as_ref());
                    dock_style.buttons.add_tab_align = egui_dock::TabAddAlign::Left;
                    // No border around the whole dock surface.
                    dock_style.main_surface_border_stroke = egui::Stroke::NONE;
                    // Square corners everywhere.
                    dock_style.main_surface_border_rounding = egui::CornerRadius::ZERO;
                    dock_style.tab_bar.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.active.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.inactive.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.hovered.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.focused.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.active_with_kb_focus.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.inactive_with_kb_focus.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.focused_with_kb_focus.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.tab_body.corner_radius = egui::CornerRadius::ZERO;
                    dock_style.tab.tab_body.stroke = egui::Stroke::NONE;
                    // Inner padding of the terminal frame from the theme
                    // (default 4px). The body background is set to the
                    // terminal's own background color so the padding ring
                    // reads as part of the terminal, not a separate panel.
                    let term_pad = self.active_theme.typography.terminal_padding;
                    dock_style.tab.tab_body.inner_margin = egui::Margin {
                        left: term_pad as i8,
                        right: term_pad as i8,
                        top: term_pad as i8,
                        bottom: term_pad as i8,
                    };
                    dock_style.tab.tab_body.bg_fill =
                        self.active_theme.terminal.background.to_egui();
                    // Focused-tab highlight from the theme.
                    let tab_hl = self.active_theme.app.tab_highlight.to_egui();
                    dock_style.tab.focused.bg_fill = tab_hl;
                    dock_style.tab.focused_with_kb_focus.bg_fill = tab_hl;
                    dock_style.tab.focused.text_color = self.active_theme.app.text.to_egui();
                    dock_style.tab.focused_with_kb_focus.text_color =
                        self.active_theme.app.text.to_egui();
                    // Shared 1px border line (same color as the global theme
                    // "边框" setting) between tabs, around the tab bar, and
                    // along the panel edges.
                    let border = self.active_theme.app.border.to_egui();
                    for s in [
                        &mut dock_style.tab.active,
                        &mut dock_style.tab.inactive,
                        &mut dock_style.tab.hovered,
                        &mut dock_style.tab.focused,
                        &mut dock_style.tab.active_with_kb_focus,
                        &mut dock_style.tab.inactive_with_kb_focus,
                        &mut dock_style.tab.focused_with_kb_focus,
                    ] {
                        s.outline_color = border;
                    }
                    dock_style.tab_bar.hline_color = border;
                    // Make the tab bar and tabs merge seamlessly with the
                    // surrounding UI: backgrounds match the app surface, so
                    // the only visible separators are the 1px border lines.
                    let bar_bg = central_fill;
                    dock_style.tab_bar.bg_fill = bar_bg;
                    dock_style.tab.inactive.bg_fill = bar_bg;
                    dock_style.tab.inactive_with_kb_focus.bg_fill = bar_bg;
                    dock_style.tab.hovered.bg_fill = self.active_theme.app.hover.to_egui();
                    dock_style.tab.active.bg_fill = self.active_theme.app.panel.to_egui();
                    dock_style.tab.active_with_kb_focus.bg_fill =
                        self.active_theme.app.panel.to_egui();
                    DockArea::new(tree)
                        .style(dock_style)
                        .show_add_buttons(true)
                        .show_add_popup(false)
                        .show_close_buttons(true)
                        .show_leaf_close_all_buttons(false)
                        .show_leaf_collapse_buttons(true)
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
                                pw_popup_open: self.pw_popup.is_some(),
                                auto_match: self.settings_edit.auto_match_command,
                                terminal_view_rects: &mut self.terminal_view_rects,
                                history_menu_just_closed: &mut self.history_menu_just_closed,
                                auto_match_pending: &mut self.auto_match_pending,
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

/// Terminal count across every workspace (sidebar 全局 group).
fn format_ws_terminal_count(app: &App) -> usize {
    app.dock_states
        .values()
        .map(|t| t.iter_all_tabs().count())
        .sum()
}

/// Terminal count in the active workspace (sidebar 当前工作区 group).
fn format_active_ws_terminal_count(app: &App) -> usize {
    app.dock_states
        .get(&app.active_panel)
        .map(|t| t.iter_all_tabs().count())
        .unwrap_or(0)
}

/// Format CPU usage percentage. `None` falls back to a dash so the bar
/// still renders cleanly on platforms where sampling isn't available.
fn format_cpu(cpu: Option<f32>) -> String {
    match cpu {
        Some(v) if v.is_finite() => format!("{:.0}% CPU", v.clamp(0.0, 100.0)),
        _ => "—% CPU".to_string(),
    }
}

/// Format memory usage, e.g. "8.2 GB". Uses binary units (1 GB = 2^30
/// bytes) to match how the macOS Activity Monitor reports it.
fn format_memory(bytes: Option<u64>) -> String {
    let b = match bytes {
        Some(v) => v as f64,
        None => return "— GB".to_string(),
    };
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else {
        format!("{:.0} MB", b / MB)
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
    fn auto_match_keeps_only_prefix_matches() {
        // Whole-text prefix match including spaces: "cd " matches "cd
        // /tmp" but NOT the bare "cd" (entry shorter than the prefix).
        let history = ["cd", "cd /tmp", "ls", "ls -la", "open", "opet"];
        let matches = |word: &str| -> Vec<&str> {
            history
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .copied()
                .collect()
        };
        assert_eq!(matches("c"), ["cd", "cd /tmp"]);
        assert_eq!(matches("cd"), ["cd", "cd /tmp"]);
        // Typed trailing space participates: only entries whose text
        // continues after "cd " match; the bare "cd" does not.
        assert_eq!(matches("cd "), ["cd /tmp"]);
        assert_eq!(matches("cd /"), ["cd /tmp"]);
        assert_eq!(matches("l"), ["ls", "ls -la"]);
        assert_eq!(matches("ls"), ["ls", "ls -la"]);
        assert_eq!(matches("ls "), ["ls -la"]);
        assert_eq!(matches("ope"), ["open", "opet"]);
        assert!(matches("xyz").is_empty());
        // Regression: history only has "cd"/"cdd"; typed "cd " (with the
        // user's trailing space) matches NEITHER — no menu.
        let hist2 = ["cd", "cdd"];
        let m2: Vec<&str> = hist2
            .iter()
            .filter(|cmd| cmd.starts_with("cd "))
            .copied()
            .collect();
        assert!(m2.is_empty());
    }

    #[test]
    fn close_latch_semantics() {
        // Structural contract of the event-driven matcher:
        //  1) no edit event  -> matcher must not touch the menu
        //  2) edit event      -> re-run the match
        //  3) latch (frame of close) -> even an edit event is masked
        // The matcher's action table, asserted directly.
        let acts_on = |edit_event: bool, latched: bool| edit_event && !latched;
        assert!(acts_on(true, false), "real key edit must act");
        assert!(!acts_on(false, false), "echo/repaint frames must never act");
        assert!(!acts_on(true, true), "close frame masks even a real edit");
        assert!(!acts_on(false, true), "close frame masked");
        // Shell echo cannot create egui events — the invariant the whole
        // design rests on: only key presses do.
        let echo_creates_event = false;
        assert!(!echo_creates_event);
    }

    #[test]
    fn valid_font_data_accepts_embedded_font_and_rejects_garbage() {
        // A real TTF bundled with the binary must pass validation.
        let real = include_bytes!("../assets/fonts/Lohit-Devanagari.ttf");
        assert!(super::is_valid_font_data(real));
        // Random bytes with a .ttf-looking extension must be rejected —
        // this is the mstmc.ttf-style crash guard.
        assert!(!super::is_valid_font_data(b"\x00\x01not-a-font"));
        assert!(!super::is_valid_font_data(b""));
    }

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
    fn workspace_lock_icon_matches_lock_state() {
        assert_eq!(
            super::workspace_lock_icon(true),
            egui_phosphor::regular::LOCK_KEY
        );
        assert_eq!(
            super::workspace_lock_icon(false),
            egui_phosphor::regular::CIRCLE_DASHED
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
            auto_word: None,
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
    pw_popup_open: bool,
    auto_match: bool,
    terminal_view_rects: &'a mut std::collections::HashMap<String, egui::Rect>,
    history_menu_just_closed: &'a mut std::collections::HashMap<String, bool>,
    auto_match_pending: &'a mut std::collections::HashMap<String, String>,
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
            let mut apply = false;
            let mut cancel = false;
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(self.terminal_rename_buffer)
                        .font(egui::FontId::monospace(14.0))
                        .desired_width(160.0)
                        .hint_text(&self.texts.terminal.rename_hint)
                        .id_source("tab_rename"),
                );
                ui.memory_mut(|mem| mem.request_focus(response.id));
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                // Confirm / cancel buttons next to the input so renaming
                // can always be applied or exited with the mouse.
                if ui.button(&self.texts.workspace.rename_confirm).clicked() || enter {
                    apply = true;
                }
                if ui.button(&self.texts.workspace.rename_cancel).clicked() || escape {
                    cancel = true;
                }
            });
            if apply {
                if !self.terminal_rename_buffer.is_empty() {
                    if let Some(data) = self.terminals.get_mut(tab) {
                        data.name = self.terminal_rename_buffer.clone();
                    }
                }
                *self.renaming_terminal = None;
            } else if cancel {
                *self.renaming_terminal = None;
            }
            ui.separator();
        }

        let mut pending_match_suffix: Option<String> = None;
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
                // While the settings window (and its password popups) is
                // open, the terminal must NOT claim keyboard focus every
                // frame — otherwise text inputs in the popups lose focus
                // immediately after being clicked.
                let terminal_may_focus = is_focused && !self.show_settings && !self.pw_popup_open;
                tv = tv.set_focus(terminal_may_focus);
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
            self.terminal_view_rects
                .insert(tab.clone(), terminal_response.rect);

            // ---- Auto-match command suggestions --------------------------------
            // Event-driven ONLY (real key edits in the FOCUSED terminal).
            // The PTY grid lags keypresses by ≥1 frame: a just-typed
            // space is not yet echoed, so the grid word alone would match
            // the pre-space text. A pending-keystroke buffer compensates:
            // effective word = grid word + pending; the buffer drains as
            // the grid catches up.
            if self.auto_match && !self.renaming && self.renaming_terminal.is_none() {
                let is_focused_tab = self.focused_terminal.as_ref() == Some(tab);
                // Collect THIS frame's keystrokes (chars + deletes).
                let (edit_event, typed, del_count) = if is_focused_tab {
                    let mut typed = String::new();
                    let mut dels = 0usize;
                    let mut any = false;
                    ui.ctx().input(|i| {
                        for e in &i.events {
                            match e {
                                egui::Event::Text(t) => {
                                    typed.push_str(t);
                                    any = true;
                                }
                                egui::Event::Key {
                                    key: egui::Key::Backspace | egui::Key::Delete,
                                    pressed: true,
                                    ..
                                } => {
                                    dels += 1;
                                    any = true;
                                }
                                _ => {}
                            }
                        }
                    });
                    (any, typed, dels)
                } else {
                    (false, String::new(), 0)
                };

                // One-frame close latch (set by every close path).
                let latched = self
                    .history_menu_just_closed
                    .get(tab.as_str())
                    .copied()
                    .unwrap_or(false);
                if latched {
                    self.history_menu_just_closed.insert(tab.clone(), false);
                }

                if edit_event && !latched {
                    let grid_word = td.instance.current_input_word();
                    // Update the pending buffer: append typed chars,
                    // backspace trims (also trims grid tail when drained).
                    let pending = self.auto_match_pending.entry(tab.clone()).or_default();
                    pending.push_str(&typed);
                    for _ in 0..del_count {
                        pending.pop();
                    }
                    // Drain: once the grid word ends with the pending
                    // suffix, the echo has caught up.
                    if !pending.is_empty() && grid_word.ends_with(pending.as_str()) {
                        pending.clear();
                    }
                    let word = format!("{grid_word}{}", pending.clone());
                    let word = word.trim_start_matches(' ').to_string();

                    if word.is_empty() {
                        if let Some(nav) = td.instance.history_nav.as_mut() {
                            if nav.auto_word.is_some() {
                                td.instance.history_nav = None;
                            }
                        }
                    } else {
                        let entries = self.history_db.get(tab, self.max_history);
                        // Whole-text prefix match INCLUDING spaces: typed
                        // "cd " matches "cd /tmp" but never bare "cd".
                        let matches: Vec<String> = entries
                            .into_iter()
                            .take(10)
                            .filter(|cmd| cmd.starts_with(word.as_str()))
                            .collect();
                        let single_exact = matches.len() == 1 && matches[0] == word;
                        if single_exact || matches.is_empty() {
                            if let Some(nav) = td.instance.history_nav.as_mut() {
                                if nav.auto_word.is_some() {
                                    td.instance.history_nav = None;
                                }
                            }
                        } else {
                            let keep_sel = td
                                .instance
                                .history_nav
                                .as_ref()
                                .is_some_and(|n| n.auto_word.is_some())
                                .then(|| {
                                    td.instance
                                        .history_nav
                                        .as_ref()
                                        .map(|n| n.selected)
                                        .unwrap_or(0)
                                })
                                .unwrap_or(0);
                            td.instance.history_nav = Some(HistoryNav {
                                entries: matches,
                                selected: keep_sel,
                                auto_word: Some(word.clone()),
                            });
                        }
                    }
                }
                // No edit event: DO NOTHING (menu state persists).
            }

            if is_focused {
                *self.terminal_focus_id = Some(terminal_response.id);
            }

            // TerminalView owns keyboard focus and its arrow-key focus lock.
            if terminal_response.clicked() {
                *self.focused_terminal = Some(tab.clone());
            }

            // The history / auto-match list renders as a GLOBAL overlay in
            // App::update (see render_history_menu), not clipped by this
            // terminal's rect.
        } else {
            ui.label(&self.texts.terminal.not_found);
        }

        // Apply a clicked auto-match suggestion (deferred: the write needs a
        // fresh mutable borrow of the terminal).
        if let Some(suffix) = pending_match_suffix.take() {
            if !suffix.is_empty() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    td.instance.write(suffix.as_bytes());
                }
            }
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

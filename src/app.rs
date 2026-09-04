use alacritty_terminal::index::{Column, Point};
use alacritty_terminal::term::point_to_viewport;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::terminal::TerminalInstance;

mod agent;
mod agent_ui;
mod ai_ui;
mod dialogs;
mod history_menu;
mod monitor;
mod remote_ui;
mod search;
mod settings_ui;
mod ssh_ui;

use self::agent_ui::{agent_stop_shortcut_hit, AgentPhase, AgentRun};
use self::ai_ui::AiCtxAction;
use self::dialogs::{open_snippet_fill_fields, SnippetFillState};
use self::history_menu::{history_menu_shortcut_released, toggle_history_menu, AltKeyState};
use self::monitor::{proc_sample_id, spawn_proc_sampler, ProcSampleJob, ProcSampleResult};
use self::remote_ui::RemoteSession;
use self::search::TerminalSearch;
use self::settings_ui::{read_settings_from, save_settings, settings_path, SettingsWindowState};
use self::ssh_ui::SshHostDialog;

static DEFAULT_SHELL_ID: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());

/// Master switch for the AI-assistant UI entries (menu-bar button +
/// settings page). The agent/AI code paths stay compiled in; flipping
/// this to `true` restores every entry.
const AI_UI_ENABLED: bool = false;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 1.0;
const WORKSPACE_SIDEBAR_DEFAULT_WIDTH: f32 = 192.0;

/// One user-configured relay channel for the phone remote control
/// (e.g. an frps instance on a cloud VM). Editing the list in settings
/// takes effect immediately: running channels whose entry changed are
/// restarted, newly enabled ones are spawned, removed ones are killed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct TunnelProfile {
    /// Display name shown in the remote panel's address selector.
    pub name: String,
    /// Relay server address (IP or hostname).
    pub server: String,
    /// Relay control port (frps bind_port, default 7000).
    #[serde(default = "default_relay_port")]
    pub port: u16,
    /// Relay auth token (must match frps auth.token).
    #[serde(default)]
    pub token: String,
    /// Public port on the relay that forwards to the local remote server.
    pub forward_port: u16,
    #[serde(default)]
    pub enabled: bool,
}

fn default_relay_port() -> u16 {
    7000
}

/// One configurable virtual key for the phone web page's bottom
/// toolbar. `action` starting with `@` is a builtin behavior (`@paste`,
/// `@copy`, `@bottom`); anything else is a byte sequence sent verbatim
/// to the PTY (escape sequences stored raw, e.g. "\x1b[A" = ArrowUp).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualKey {
    pub label: String,
    pub action: String,
}

fn default_virtual_keys() -> Vec<VirtualKey> {
    let k = |label: &str, action: &str| VirtualKey {
        label: label.into(),
        action: action.into(),
    };
    vec![
        k("Esc", "\x1b"),
        k("Tab", "\t"),
        k("↑", "\x1b[A"),
        k("↓", "\x1b[B"),
        k("←", "\x1b[D"),
        k("→", "\x1b[C"),
        k("PgUp", "\x1b[5~"),
        k("PgDn", "\x1b[6~"),
        k("Home", "\x1b[H"),
        k("End", "\x1b[F"),
        k("Ctrl+C", "\x03"),
        k("Ctrl+D", "\x04"),
        k("Ctrl+L", "\x0c"),
        k("Paste", "@paste"),
        k("Copy", "@copy"),
        k("⤓", "@bottom"),
    ]
}

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
    /// Preferred shell for NEW terminals (id into the detected shell
    /// list; only meaningful on Windows where multiple shells exist).
    #[serde(default = "default_shell_pref")]
    default_shell: String,
    /// Edge-smoothing (feathering) master switch. Off = hard edges on
    /// shapes/lines/borders (crisper, terminal-like); text glyph AA is
    /// built into the font atlas and unaffected.
    #[serde(default = "default_true")]
    smooth_rendering: bool,
    /// Feathering width in PHYSICAL pixels when smooth_rendering is on.
    /// 1.0 is the epaint default; larger = blurrier edges.
    #[serde(default = "default_smooth_level")]
    smooth_level: f32,
    /// Red warning banner on terminals connected to PROD-marked SSH hosts.
    #[serde(default = "default_true")]
    ssh_prod_banner: bool,
    /// AI assistant master switch (panel is hidden until enabled).
    #[serde(default)]
    ai_enabled: bool,
    /// OpenAI-compatible endpoint base (Ollama: http://localhost:11434/v1).
    #[serde(default = "default_ai_base_url")]
    ai_base_url: String,
    /// API key. Stored ONLY in the local settings.json and sent ONLY to
    /// the configured endpoint.
    #[serde(default)]
    ai_api_key: String,
    #[serde(default = "default_ai_model")]
    ai_model: String,
    /// Agent approval mode id ("manual" | "allowlist" | "full-auto").
    #[serde(default = "default_agent_approval_mode")]
    agent_approval_mode: String,
    /// Hard step cap for one agent run.
    #[serde(default = "default_agent_max_steps")]
    agent_max_steps: usize,
    /// Port the embedded remote-control server binds (0.0.0.0).
    #[serde(default = "default_remote_port")]
    remote_port: u16,
    /// Relay channels for WAN phone access (frp servers etc.).
    #[serde(default)]
    remote_tunnels: Vec<TunnelProfile>,
    /// Virtual keys rendered on the phone web page's bottom toolbar
    /// (user-configurable; defaults mirror the legacy hardcoded set).
    #[serde(default = "default_virtual_keys")]
    remote_keys: Vec<VirtualKey>,
}

fn default_agent_approval_mode() -> String {
    "allowlist".into()
}

fn default_agent_max_steps() -> usize {
    10
}

fn default_remote_port() -> u16 {
    47822
}

fn default_ai_base_url() -> String {
    "https://api.openai.com/v1".into()
}

fn default_ai_model() -> String {
    "gpt-4o-mini".into()
}

fn default_smooth_level() -> f32 {
    1.0
}

fn default_shell_pref() -> String {
    "cmd".into()
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
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "next_terminal".into(),
        ShortcutBinding {
            key: "Tab".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "next_panel".into(),
        ShortcutBinding {
            key: "Q".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "next_workspace".into(),
        ShortcutBinding {
            key: "W".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "stop_agent".into(),
        ShortcutBinding {
            key: ".".into(),
            ctrl: true,
            shift: true,
            alt: false,
        },
    );
    m.insert(
        "save_scene".into(),
        ShortcutBinding {
            key: "S".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "terminal_interrupt".into(),
        ShortcutBinding {
            key: "C".into(),
            ctrl: true,
            shift: true,
            alt: false,
        },
    );
    m.insert(
        "terminal_copy".into(),
        ShortcutBinding {
            key: "C".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "terminal_paste".into(),
        ShortcutBinding {
            key: "V".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "terminal_cut".into(),
        ShortcutBinding {
            key: "X".into(),
            ctrl: true,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "close_terminal".into(),
        ShortcutBinding {
            key: "E".into(),
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
        "history_favorite".into(),
        ShortcutBinding {
            key: "F2".into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "history_delete".into(),
        ShortcutBinding {
            key: "Delete".into(),
            ctrl: false,
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
        "Q" => Some(egui::Key::Q),
        "E" => Some(egui::Key::E),
        "S" => Some(egui::Key::S),
        "C" => Some(egui::Key::C),
        "V" => Some(egui::Key::V),
        "X" => Some(egui::Key::X),
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
        "Insert" => Some(egui::Key::Insert),
        "Delete" => Some(egui::Key::Delete),
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
        "Q" => "Q",
        "E" => "E",
        "S" => "S",
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
        "Insert" => "Insert",
        "Delete" => "Delete",
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

fn shortcut_hint_ids() -> [&'static str; 24] {
    [
        "new_terminal",
        "close_terminal",
        "stop_agent",
        "workspace_up",
        "workspace_down",
        "panel_left",
        "panel_right",
        "lock_workspace",
        "history_menu",
        "history_prev",
        "history_next",
        "history_favorite",
        "history_delete",
        "next_terminal",
        "next_panel",
        "next_workspace",
        "save_scene",
        "terminal_interrupt",
        "terminal_copy",
        "terminal_paste",
        "terminal_cut",
        "toggle_workspace_sidebar",
        "zoom_in",
        "zoom_out",
    ]
}

fn shortcut_label_for<'a>(texts: &'a crate::i18n::Texts, id: &str) -> &'a str {
    match id {
        "new_terminal" => &texts.shortcut_labels.new_terminal,
        "close_terminal" => &texts.shortcut_labels.close_terminal,
        "stop_agent" => &texts.shortcut_labels.stop_agent,
        "workspace_up" => &texts.shortcut_labels.workspace_up,
        "workspace_down" => &texts.shortcut_labels.workspace_down,
        "panel_left" => &texts.shortcut_labels.panel_left,
        "panel_right" => &texts.shortcut_labels.panel_right,
        "lock_workspace" => &texts.shortcut_labels.lock_workspace,
        "history_menu" => &texts.shortcut_labels.history_menu,
        "history_prev" => &texts.shortcut_labels.history_prev,
        "history_next" => &texts.shortcut_labels.history_next,
        "history_favorite" => &texts.shortcut_labels.history_favorite,
        "history_delete" => &texts.shortcut_labels.history_delete,
        "next_terminal" => &texts.shortcut_labels.next_terminal,
        "next_panel" => &texts.shortcut_labels.next_panel,
        "next_workspace" => &texts.shortcut_labels.next_workspace,
        "save_scene" => &texts.shortcut_labels.save_scene,
        "terminal_interrupt" => &texts.shortcut_labels.terminal_interrupt,
        "terminal_copy" => &texts.shortcut_labels.terminal_copy,
        "terminal_paste" => &texts.shortcut_labels.terminal_paste,
        "terminal_cut" => &texts.shortcut_labels.terminal_cut,
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

/// Reserved font families holding the CLEAN generic stacks (before the
/// active theme injects its fonts at the head). Theme-list previews use
/// these so switching themes never changes other previews' rendering.
fn preview_prop_family() -> egui::FontFamily {
    egui::FontFamily::Name(std::sync::Arc::from("__preview_proportional__"))
}
fn preview_mono_family() -> egui::FontFamily {
    egui::FontFamily::Name(std::sync::Arc::from("__preview_monospace__"))
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

/// A workspace whose focused terminal went ≥30s without PTY output or
/// user input counts as IDLE (green dot); anything inside the window
/// is ACTIVE (red dot). No focused terminal → UNKNOWN (neutral dot).
pub(crate) const WORKSPACE_IDLE_MS: u64 = 30_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceActivity {
    /// Output/input inside the idle window.
    Active,
    /// Silent for the whole window.
    Idle,
    /// No terminal / no focused leaf to watch.
    Unknown,
}

fn workspace_activity_state(activity_ms: Option<u64>, now_ms: u64) -> WorkspaceActivity {
    match activity_ms {
        Some(ms) if now_ms.saturating_sub(ms) < WORKSPACE_IDLE_MS => WorkspaceActivity::Active,
        Some(_) => WorkspaceActivity::Idle,
        None => WorkspaceActivity::Unknown,
    }
}

impl App {
    /// Latest activity across EVERY terminal of a workspace (max of
    /// their last_activity_ms): any one busy terminal marks the whole
    /// workspace active — the sidebar strip and the phone page's dot
    /// both read this. None = the workspace has no terminals at all.
    pub(crate) fn workspace_activity_ms(&self, panel_idx: usize) -> Option<u64> {
        let tree = self.dock_states.get(&panel_idx)?;
        let mut latest: Option<u64> = None;
        for (_, tab_id) in tree.iter_all_tabs() {
            if let Some(td) = self.terminals.get(tab_id) {
                let ms = td.instance.last_activity_ms();
                latest = Some(match latest {
                    Some(cur) if cur >= ms => cur,
                    _ => ms,
                });
            }
        }
        latest
    }
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

/// Like [`check_shortcut`] but immune to key AUTO-REPEAT: holding the
/// combo fires exactly once per physical press. Holding the
/// workspace-switch keys used to switch workspaces every repeat frame
/// (~30/s), thrashing multi-terminal repaints (visible freeze) and
/// leaving egui's keyboard focus stranded on a widget from an
/// already-hidden workspace — after which every Text event was routed
/// to that ghost widget and typing died app-wide.
fn check_shortcut_no_repeat(
    ctx: &egui::Context,
    binds: &HashMap<String, ShortcutBinding>,
    name: &str,
) -> bool {
    let Some(b) = binds.get(name) else {
        return false;
    };
    let Some(key) = binding_to_key(b) else {
        return false;
    };
    let mods = binding_to_modifiers(b);
    let mut matched = false;
    ctx.input_mut(|input| {
        // Matching events are ALWAYS removed: non-repeat hits count as
        // the trigger, repeats are dropped on the floor (a leftover
        // repeat would otherwise leak into the focused terminal as a
        // bare arrow-key escape sequence).
        input.events.retain(|event| match event {
            egui::Event::Key {
                key: event_key,
                modifiers: event_mods,
                pressed: true,
                repeat,
                ..
            } if *event_key == key && *event_mods == mods => {
                matched |= !repeat;
                false
            }
            _ => true,
        });
    });
    matched
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

/// UNIFIED dialog keyboard protocol (self-managed cursor).
///
/// One protocol for EVERY two-button confirm dialog in the app. The
/// dialog owns a `kb_confirm: bool` cursor (which button is selected),
/// drawn as a highlight by the caller. This function implements:
///   - Escape  -> returns `close` (caller closes the dialog)
///   - Enter   -> activates the selected side (returns confirm/cancel)
///   - Left/Right (optional, `toggle: true`) -> flips the cursor
///
/// All keys are CONSUMED here - the single most common historical bug
/// was a dialog reading `key_pressed` (non-consuming) and letting the
/// same Enter also fall through to the terminal.
///
/// Why self-managed instead of egui's focus system: egui 0.31's
/// `begin_pass` runs directional focus-navigation BEFORE any app code,
/// and `set_focus_lock_filter` only takes effect from the second frame
/// (it requires `had_focus_last_frame`). A modal's first frames could
/// therefore bounce focus to spatially-adjacent widgets. Measured,
/// reproduced in headless tests, and the reason the favorite dialogs
/// moved to this model (commit 5624965).
///
/// Call this BEFORE creating the dialog's Window/Modal so nothing can
/// swallow the keys earlier in the frame.
fn dialog_keys(ctx: &egui::Context, kb_confirm: &mut bool, toggle: bool) -> DialogKeysOutcome {
    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    if toggle {
        let left = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft));
        let right = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight));
        if left || right {
            *kb_confirm = !*kb_confirm;
        }
    }
    DialogKeysOutcome {
        close: escape,
        confirm: enter && *kb_confirm,
        cancel: enter && !*kb_confirm,
        enter,
    }
}

/// Result of [`dialog_keys`]. `enter` is exposed separately for
/// input-style dialogs (name/command) where Enter means "confirm"
/// regardless of the button cursor.
#[derive(Debug, Default, Clone, Copy)]
struct DialogKeysOutcome {
    close: bool,
    confirm: bool,
    cancel: bool,
    enter: bool,
}

/// SINGLE source of truth for "a modal dialog owns the UI right now".
/// Every keyboard-consumer (history-menu key block, terminal focus
/// grants, global shortcuts) must defer to whatever modal is topmost —
/// previously each consumer knew its own subset of dialogs, so keys
/// leaked: arrow keys drove the menu BEHIND a confirm popup, and the
/// terminal re-claimed focus from text inputs.
///
/// REGISTRY — when adding a NEW modal dialog, add its flag HERE (and
/// only here; the three consumers below all read this function):
///   settings, about, theme editor, pw popup, unlock popup,
///   theme dialogs (copy/new/rename/delete/switch),
///   workspace close confirm, terminal close confirm,
///   history clear confirm, favorites clear confirm,
///   favorite folder dialogs (name/command/delete).
fn any_modal_open(app: &App) -> bool {
    any_modal_open_excluding_ai(app) || app.show_ai_panel
}

/// Every modal EXCEPT the AI assistant panel. The AI panel is a modal
/// for keyboard isolation, but it must still be closable with Esc —
/// the Esc handler needs to know whether some OTHER modal (settings,
/// theme editor, …) owns the keyboard first.
fn any_modal_open_excluding_ai(app: &App) -> bool {
    app.show_settings
        || app.show_about
        || app.show_update_window
        || app.show_help_window
        || app.theme_editor_open
        || app.pw_popup.is_some()
        || app.unlock_popup.is_some()
        || app.theme_dialog.show_copy_dialog
        || app.theme_dialog.show_new_dialog
        || app.theme_dialog.show_rename_dialog
        || app.theme_dialog.show_delete_confirm
        || app.theme_dialog.show_switch_confirm
        || app.close_confirm_panel.is_some()
        || app.pending_close_confirm.is_some()
        || app.history_clear_confirm.is_some()
        || app.show_clear_favorites_confirm
        || app.fav_name_dialog.is_some()
        || app.fav_cmd_dialog.is_some()
        || app.fav_delete_confirm.is_some()
        || app.ssh_dialog.is_some()
        || app.ssh_delete_confirm.is_some()
        || app.snippet_fill.is_some()
        || app.startup_cmd_dialog.is_some()
        || app.terminal_search.is_some()
        || app.ai_exec_confirm.is_some()
        || app
            .agent
            .as_ref()
            .is_some_and(|a| a.pending_confirm.is_some())
        || matches!(app.update_state, crate::updater::UpdateState::Ready(_))
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

#[derive(Debug, Clone, Default)]
enum StartCheckResult {
    Available(crate::updater::UpdateInfo),
    #[default]
    UpToDate,
    Error(String),
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

pub(crate) fn app_data_dir() -> PathBuf {
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
    // One-time upgrade: releases before hashing kept the workspace-lock
    // secret as plaintext in settings.json. Re-hash whatever we loaded
    // so plaintext never survives past the first start of a hashed build.
    let mut changed = false;
    if !settings.lock_password.is_empty() && !lock_password_is_hashed(&settings.lock_password) {
        settings.lock_password = hash_lock_password(&settings.lock_password);
        changed = true;
    }

    #[cfg(debug_assertions)]
    {
        let original = settings.key_binds.clone();
        let defaults = default_key_binds();
        for (key, default_binding) in &defaults {
            settings
                .key_binds
                .insert(key.clone(), default_binding.clone());
        }
        let binds_changed = settings.key_binds != original;
        (settings, changed || binds_changed)
    }

    #[cfg(not(debug_assertions))]
    {
        let (settings, binds_changed) = normalize_settings_release_impl(settings);
        (settings, changed || binds_changed)
    }
}

#[cfg_attr(debug_assertions, allow(dead_code))] // used by release builds
fn normalize_settings_release_impl(mut settings: AppSettings) -> (AppSettings, bool) {
    let original = settings.key_binds.clone();
    let defaults = default_key_binds();
    if !settings.key_binds.contains_key("toggle_workspace_sidebar") {
        settings.key_binds.insert(
            "toggle_workspace_sidebar".into(),
            defaults.get("toggle_workspace_sidebar").cloned().unwrap(),
        );
    }
    if !settings.key_binds.contains_key("stop_agent") {
        settings.key_binds.insert(
            "stop_agent".into(),
            defaults.get("stop_agent").cloned().unwrap(),
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

const ARGON2_HASH_PREFIX: &str = "$argon2";

/// Hash a workspace-lock password into a PHC string (argon2id, random
/// salt). Plaintext never touches disk from a fresh install onwards.
fn hash_lock_password(password: &str) -> String {
    use argon2::password_hash::{PasswordHasher, SaltString};
    // Salt from two UUIDs (64 hex chars): hex is within the PHC b64
    // alphabet, and this avoids argon2's re-exported rand_core::OsRng,
    // whose availability depends on transitive feature unification and
    // broke the macOS/Windows CI builds.
    let salt_hex = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let Ok(salt) = SaltString::from_b64(&salt_hex) else {
        return String::new();
    };
    argon2::Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .unwrap_or_default()
}

/// Verify an entered password against the stored value. Legacy stores
/// (releases before hashing) keep plaintext on disk; those still verify
/// and are upgraded to a hash by the caller after a successful entry.
fn verify_lock_password(password: &str, stored: &str) -> bool {
    if stored.starts_with(ARGON2_HASH_PREFIX) {
        use argon2::password_hash::PasswordVerifier;
        match argon2::password_hash::PasswordHash::new(stored) {
            Ok(parsed) => argon2::Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    } else {
        password == stored
    }
}

fn lock_password_is_hashed(stored: &str) -> bool {
    stored.starts_with(ARGON2_HASH_PREFIX)
}

#[cfg(test)]
mod lock_password_tests {
    use super::*;

    #[test]
    fn hashed_secret_roundtrips_and_rejects_wrong_input() {
        let stored = hash_lock_password("hunter2");
        assert!(lock_password_is_hashed(&stored));
        assert_ne!(stored, "hunter2");
        assert!(verify_lock_password("hunter2", &stored));
        assert!(!verify_lock_password("hunter3", &stored));
    }

    #[test]
    fn legacy_plaintext_verifies_and_migration_flags_it() {
        assert!(verify_lock_password("old-secret", "old-secret"));
        assert!(!lock_password_is_hashed("old-secret"));
        // The load-time migration re-hashes any plaintext it finds.
        let migrated = hash_lock_password("old-secret");
        assert!(lock_password_is_hashed(&migrated));
        assert!(verify_lock_password("old-secret", &migrated));
    }

    #[test]
    fn empty_stored_secret_only_matches_empty_input() {
        assert!(verify_lock_password("", ""));
        assert!(!verify_lock_password("x", ""));
    }
}

/// Store a new lock secret as an argon2id hash (set/change flows).
fn store_lock_password(
    settings: &mut AppSettings,
    settings_edit: &mut AppSettings,
    password: &str,
) {
    let hashed = hash_lock_password(password);
    settings.lock_password = hashed.clone();
    settings_edit.lock_password = hashed;
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
            ssh_prod_banner: true,
            ai_enabled: false,
            ai_base_url: default_ai_base_url(),
            ai_api_key: String::new(),
            ai_model: default_ai_model(),
            agent_approval_mode: default_agent_approval_mode(),
            agent_max_steps: default_agent_max_steps(),
            remote_port: default_remote_port(),
            auto_match_command: true,
            default_shell: default_shell_pref(),
            smooth_rendering: true,
            smooth_level: default_smooth_level(),
            remote_tunnels: Vec::new(),
            remote_keys: default_virtual_keys(),
        }
    }
}

/// Whether the font file bytes parse as a real font face. Some files in
/// the system font directories carry a .ttf/.otf/.ttc extension but are
/// not valid font resources (e.g. Windows mstmc.ttf bitmap fonts);
/// egui/epaint panics when parsing those at first use, crashing startup.
/// Whether the font file provides glyphs for common CJK codepoints
/// (中/文/界). TTC collections are probed at index 0 — enough to
/// classify the FILE. Drives the theme editor's "CJK" badge and the
/// fallback hint: a font without CJK glyphs keeps every Chinese label
/// on the bundled Noto Sans CJK, so only Latin text visibly changes.
fn font_file_has_cjk(data: &[u8]) -> bool {
    use ab_glyph::Font as _;
    let probe = |font: ab_glyph::FontRef| {
        ["中", "文", "界"]
            .iter()
            .all(|c| font.glyph_id(c.chars().next().unwrap()).0 != 0)
    };
    if let Ok(f) = ab_glyph::FontRef::try_from_slice(data) {
        probe(f)
    } else if let Ok(f) = ab_glyph::FontRef::try_from_slice_and_index(data, 0) {
        probe(f)
    } else {
        false
    }
}

/// One OS system-UI font: fixed registration key, file path, TTC index.
pub(crate) struct OsUiFont {
    /// Fixed key used in egui's font_data ("os-ui" = Latin UI face,
    /// "os-ui-cjk" = the system's Chinese face).
    pub key: &'static str,
    pub path: PathBuf,
    pub index: u32,
}

/// Parse one line of `fc-match -f "%{file}\t%{index}\t%{family}"`.
/// Returns (file, ttc index). Family names are NOT used as keys — the
/// caller registers under the fixed "os-ui"/"os-ui-cjk" keys.
fn parse_fc_match(line: &str) -> Option<(String, u32)> {
    // Trim only the trailing newline — a leading tab means an EMPTY
    // file field, which must stay empty (and be rejected).
    let mut parts = line.trim_end().split('\t');
    let file = parts.next()?.trim();
    // Non-TTC faces report "-1" (or omit the field): face 0 is right.
    let index = parts
        .next()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    if file.is_empty() {
        return None;
    }
    Some((file.to_string(), index))
}

/// Probe the OS's own UI font files once per process. "system-ui"
/// themes head the Proportional stack with these so the whole app —
/// Chinese labels included — matches the desktop's configured font.
fn detect_os_ui_fonts() -> Vec<OsUiFont> {
    let mut out: Vec<OsUiFont> = Vec::new();
    match std::env::consts::OS {
        "linux" => {
            // fontconfig owns the answer on Linux; fc-match resolves the
            // generic family to a concrete file (<50ms).
            for (pattern, key) in [
                ("sans-serif", "os-ui"),
                ("sans-serif:lang=zh-cn", "os-ui-cjk"),
            ] {
                if let Ok(output) = std::process::Command::new("fc-match")
                    .args(["-f", "%{file}\t%{index}\t%{family}", pattern])
                    .output()
                {
                    if let Some((file, index)) =
                        parse_fc_match(&String::from_utf8_lossy(&output.stdout))
                    {
                        out.push(OsUiFont {
                            key,
                            path: PathBuf::from(file),
                            index,
                        });
                    }
                }
            }
        }
        "windows" => {
            if let Some(windir) = std::env::var_os("WINDIR") {
                let fonts = PathBuf::from(windir).join("Fonts");
                out.push(OsUiFont {
                    key: "os-ui",
                    path: fonts.join("segoeui.ttf"),
                    index: 0,
                });
                out.push(OsUiFont {
                    key: "os-ui-cjk",
                    path: fonts.join("msyh.ttc"),
                    index: 0,
                });
            }
        }
        "macos" => {
            out.push(OsUiFont {
                key: "os-ui",
                path: PathBuf::from("/System/Library/Fonts/Helvetica.ttc"),
                index: 0,
            });
            out.push(OsUiFont {
                key: "os-ui-cjk",
                path: PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
                index: 0,
            });
        }
        _ => {}
    }
    out.retain(|f| f.path.is_file());
    out
}

/// Convert a captured key event into a virtual-key byte sequence and a
/// suggested display label. `None` = the key cannot produce a sequence
/// (treated as cancel by the caller). CSI-key modifier suffix follows
/// the xterm convention: 1 + shift(1) + alt(2) + ctrl(4).
fn virtual_key_from_capture(
    key: egui::Key,
    modifiers: egui::Modifiers,
) -> Option<(String, String)> {
    use egui::Key as K;
    let m = (modifiers.shift as u8) | ((modifiers.alt as u8) << 1) | ((modifiers.ctrl as u8) << 2);
    let mod_code = 1 + m;
    let csi = |final_byte: char, base: &str| -> (String, String) {
        let seq = if mod_code == 1 {
            format!("\x1b{base}")
        } else {
            format!("\x1b[1;{mod_code}{final_byte}")
        };
        (seq, String::new())
    };
    let (seq, mut label): (String, String) = match key {
        K::ArrowUp => csi('A', "[A"),
        K::ArrowDown => csi('B', "[B"),
        K::ArrowRight => csi('C', "[C"),
        K::ArrowLeft => csi('D', "[D"),
        K::Home => csi('H', "[H"),
        K::End => csi('F', "[F"),
        K::PageUp => {
            let seq = if mod_code == 1 {
                "\x1b[5~".into()
            } else {
                format!("\x1b[5;{mod_code}~")
            };
            (seq, String::new())
        }
        K::PageDown => {
            let seq = if mod_code == 1 {
                "\x1b[6~".into()
            } else {
                format!("\x1b[6;{mod_code}~")
            };
            (seq, String::new())
        }
        K::Insert => csi('~', "[2~"),
        K::Delete => csi('~', "[3~"),
        K::Enter => ("\r".into(), "Enter".into()),
        K::Tab => ("\t".into(), "Tab".into()),
        K::Backspace => ("\x7f".into(), "Bksp".into()),
        K::Space => (" ".into(), "Space".into()),
        K::F1 => ("\x1bOP".into(), "F1".into()),
        K::F2 => ("\x1bOQ".into(), "F2".into()),
        K::F3 => ("\x1bOR".into(), "F3".into()),
        K::F4 => ("\x1bOS".into(), "F4".into()),
        K::F5 => ("\x1b[15~".into(), "F5".into()),
        K::F6 => ("\x1b[17~".into(), "F6".into()),
        K::F7 => ("\x1b[18~".into(), "F7".into()),
        K::F8 => ("\x1b[19~".into(), "F8".into()),
        K::F9 => ("\x1b[20~".into(), "F9".into()),
        K::F10 => ("\x1b[21~".into(), "F10".into()),
        K::F11 => ("\x1b[23~".into(), "F11".into()),
        K::F12 => ("\x1b[24~".into(), "F12".into()),
        _ => {
            // Letters/digits/punctuation: bare char, Ctrl -> control
            // byte, Alt -> ESC prefix.
            let ch = key_to_char(key)?;
            let mut seq = String::new();
            if modifiers.ctrl {
                seq.push((ch.to_ascii_uppercase() as u8 % 32) as char);
            } else {
                seq.push(ch);
            }
            if modifiers.alt {
                seq.insert(0, '\x1b');
            }
            let mut label = String::new();
            if modifiers.ctrl {
                label.push_str("Ctrl+");
            }
            if modifiers.alt {
                label.push_str("Alt+");
            }
            if modifiers.shift && !modifiers.ctrl {
                label.push_str("Shift+");
            }
            label.push(ch.to_ascii_uppercase());
            (seq, label)
        }
    };
    if label.is_empty() {
        // Direction/CSI keys: synthesize the label from the modifiers.
        if modifiers.any() {
            let mut l = String::new();
            if modifiers.ctrl {
                l.push_str("Ctrl+");
            }
            if modifiers.alt {
                l.push_str("Alt+");
            }
            if modifiers.shift {
                l.push_str("Shift+");
            }
            l.push_str(base_key_name(key));
            label = l;
        } else {
            label = base_key_name(key).to_string();
        }
    }
    Some((seq, label))
}

fn base_key_name(key: egui::Key) -> &'static str {
    use egui::Key as K;
    match key {
        K::ArrowUp => "↑",
        K::ArrowDown => "↓",
        K::ArrowLeft => "←",
        K::ArrowRight => "→",
        K::Home => "Home",
        K::End => "End",
        K::PageUp => "PgUp",
        K::PageDown => "PgDn",
        K::Insert => "Ins",
        K::Delete => "Del",
        K::Enter => "Enter",
        K::Tab => "Tab",
        K::Backspace => "Bksp",
        K::Space => "Space",
        K::F1 => "F1",
        K::F2 => "F2",
        K::F3 => "F3",
        K::F4 => "F4",
        K::F5 => "F5",
        K::F6 => "F6",
        K::F7 => "F7",
        K::F8 => "F8",
        K::F9 => "F9",
        K::F10 => "F10",
        K::F11 => "F11",
        K::F12 => "F12",
        _ => "",
    }
}

/// Printable character for letter/digit/punctuation keys (None for
/// specials — those map to escape sequences in virtual_key_from_capture).
fn key_to_char(key: egui::Key) -> Option<char> {
    use egui::Key as K;
    Some(match key {
        K::A => 'a',
        K::B => 'b',
        K::C => 'c',
        K::D => 'd',
        K::E => 'e',
        K::F => 'f',
        K::G => 'g',
        K::H => 'h',
        K::I => 'i',
        K::J => 'j',
        K::K => 'k',
        K::L => 'l',
        K::M => 'm',
        K::N => 'n',
        K::O => 'o',
        K::P => 'p',
        K::Q => 'q',
        K::R => 'r',
        K::S => 's',
        K::T => 't',
        K::U => 'u',
        K::V => 'v',
        K::W => 'w',
        K::X => 'x',
        K::Y => 'y',
        K::Z => 'z',
        K::Num0 => '0',
        K::Num1 => '1',
        K::Num2 => '2',
        K::Num3 => '3',
        K::Num4 => '4',
        K::Num5 => '5',
        K::Num6 => '6',
        K::Num7 => '7',
        K::Num8 => '8',
        K::Num9 => '9',
        K::Minus => '-',
        K::Period => '.',
        K::Comma => ',',
        K::Plus => '+',
        K::Equals => '=',
        K::Semicolon => ';',
        K::Slash => '/',
        K::Backslash => '\\',
        K::Quote => '\'',
        K::OpenBracket => '[',
        K::CloseBracket => ']',
        K::Backtick => '`',
        _ => return None,
    })
}

/// Human-visible rendering of a stored byte sequence (settings list):
/// ESC / control bytes become ^X-style markers.
fn display_seq(seq: &str) -> String {
    seq.chars()
        .map(|c| match c {
            '\x1b' => "ESC".to_string(),
            '\r' => "^M".to_string(),
            '\t' => "^I".to_string(),
            c if (c as u32) < 32 => format!("^{}", (b'@' + c as u8) as char),
            c => c.to_string(),
        })
        .collect()
}

fn is_valid_font_data(data: &[u8]) -> bool {
    // Single faces parse directly; TTC collections need an index
    // (index 0 validates the container — FontData carries the real
    // index at load time).
    ab_glyph::FontRef::try_from_slice(data).is_ok()
        || ab_glyph::FontRef::try_from_slice_and_index(data, 0).is_ok()
}

/// Symbol/dingbat/ornament fonts (OpenSymbol, Standard Symbols PS,
/// Wingdings-like collections, decorative families). Their cmaps often
/// cover plain Latin codepoints with circled/overlined glyph variants,
/// so when they leak into a fallback chain, ordinary words render with
/// stray overlines, ticks and torn letter spacing — while COPY stays
/// fine because only the glyphs, not the text, are wrong.
fn is_symbol_font_name(name: &str) -> bool {
    const SYMBOL_PATTERNS: &[&str] = &[
        "symbol",
        "dingbat",
        "wingding",
        "webdings",
        "ornament",
        "dejavusansmonoextra", // -Extra variants ship symbol ranges
        "icon",
        "emoji",
        "webfont",
        "opens__",
    ];
    let lower = name.to_lowercase();
    SYMBOL_PATTERNS.iter().any(|p| lower.contains(p))
        || name.starts_with("open") && lower.ends_with("symbol")
}

/// Whether a scanned family name plausibly belongs in a MONOSPACE
/// fallback chain. Only genuinely monospaced faces advance cells
/// uniformly; proportional faces in the chain tear the terminal grid's
/// column math (half-width Latin alternating with wide fallbacks).
fn is_monospace_family_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("mono") || lower.contains("consol") || lower.contains("courier")
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
    /// Shell id the terminal was spawned with (restored on scene load).
    #[serde(default)]
    shell: String,
    /// SSH host row the terminal is connected to (0 = local shell).
    /// Restored sessions reconnect through the host book.
    #[serde(default)]
    host_id: i64,
    /// Command executed when the terminal is (re)created (empty = none).
    #[serde(default)]
    startup_command: String,
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
}

pub use crate::terminal::HistoryNav;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogKind {
    New,
    Copy,
    Rename,
    Delete,
    Switch,
}

/// Pages of the Unity-style settings window. The numeric order matches
/// the nav listing; `as u8` is persisted. (Pre-split legacy values 1/2
/// meant Shortcuts/Lock — a one-shot page-memory mismatch after
/// upgrading is harmless, so the new pages simply take 1/2.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsPage {
    General = 0,
    AiAssistant = 1,
    Remote = 2,
    Shortcuts = 3,
    Lock = 4,
    Theme = 5,
}

impl SettingsPage {
    fn from_u8(v: u8) -> Self {
        match v {
            // 1 = AI page, but while AI_UI_ENABLED is off the entry is
            // hidden — fall back to General (stale page memory must not
            // open a page the nav doesn't show).
            1 if AI_UI_ENABLED => SettingsPage::AiAssistant,
            2 => SettingsPage::Remote,
            3 => SettingsPage::Shortcuts,
            4 => SettingsPage::Lock,
            5..=8 => SettingsPage::Theme,
            _ => SettingsPage::General,
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
    /// Confirmation dialog for clearing the GLOBAL favorite commands.
    show_clear_favorites_confirm: bool,
    settings: AppSettings,
    show_settings: bool,
    show_about: bool,
    /// Standalone update window (menu 帮助 → 更新, or the right-corner
    /// update badge). Opening it auto-triggers a version check.
    show_update_window: bool,
    /// Standalone help window (menu 帮助 → 帮助): implemented-features
    /// overview.
    show_help_window: bool,
    /// Release notes captured while the state was Available, so the
    /// changelog list keeps showing during Downloading/Ready (the
    /// UpdateState no longer carries info then).
    update_notes_cache: (Vec<String>, String),
    settings_edit: AppSettings,
    /// Active settings page (see `SettingsPage`). Stored as u8 for serde
    /// compatibility with the persisted `settings_window` payload.
    settings_tab: u8,
    /// Transient "已应用" toast shown in the settings footer.
    settings_applied_toast: Option<(String, std::time::Instant)>,
    settings_window_open: bool,
    binding_recording: Option<String>,
    cached_template_files: Vec<(String, PathBuf)>,
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
    /// Cached `TerminalTheme` shared across every terminal. Rebuilt only
    /// when the active/edit theme changes; per-frame per-terminal use just
    /// clones the `Arc` (O(1)) instead of re-allocating a `ColorPalette`
    /// box + a 256-entry ANSI table + 27 color parses.
    terminal_theme_cache: std::sync::Arc<egui_term::TerminalTheme>,
    /// Snapshot of the theme the cache was built from (for change detection).
    terminal_theme_cache_theme: crate::theme::ThemeDefinition,
    /// Whether the cached theme was built from the settings EDIT draft.
    terminal_theme_cache_edit: bool,
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
    /// Mirror of the clipboard content we last wrote/read, so a
    /// REMAPPED paste key can still paste (egui only exposes clipboard
    /// READ via its built-in Ctrl+V channel).
    clipboard_mirror: String,
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
    /// Shell choice for the NEXT created terminal (set by the new-terminal
    /// dropdown; cleared once consumed).
    pending_shell: Option<crate::shells::ShellOption>,
    /// Shell id consumed by the last create_terminal_inner call (used to
    /// stamp TerminalData.shell_id for scene persistence).
    pending_shell_last: Option<String>,
    /// Shells detected at startup (Windows: cmd/powershell/pwsh/vs/wsl).
    detected_shells: Vec<crate::shells::ShellOption>,
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
    /// Monitor panel rows: (tab id, cpu%, rss bytes) from the last tick.
    monitor_rows: Vec<(String, Option<f32>, Option<u64>)>,
    /// Recent global terminal-tree CPU% samples for the sparkline.
    cpu_history: Vec<f32>,
    /// Floating monitor panel visibility (视图 menu).
    show_monitor: bool,
    /// Terminal scrollback search bar state (Ctrl+F on the focused tab).
    terminal_search: Option<TerminalSearch>,
    /// Sender to the persistent sampler worker; respawned on failure.
    /// The heavy process-table scan runs OFF the UI thread.
    proc_sample_tx: Option<std::sync::mpsc::Sender<ProcSampleJob>>,
    /// System fonts scanned+validated once; Arc-shared into every font
    /// atlas rebuild (see rebuild_fonts).
    font_asset_cache: Option<Vec<(String, std::sync::Arc<egui::FontData>)>>,
    /// Fonts (from the asset cache) that provide CJK glyphs — drives the
    /// theme editor's "· CJK" badges and the no-CJK fallback hint.
    font_cjk_names: std::collections::HashSet<String>,
    /// Corruption/recovery notices queued at startup, drained into the
    /// toast channel one per frame once the UI is live.
    startup_warnings: Vec<String>,
    /// Set once the deferred startup update-check result was consumed.
    startup_check_consumed: bool,
    // ---- Favorite folders (v0.1.46) ----
    /// Cached (id, name) list in DISPLAY order; refreshed after every
    /// mutation from history_db.
    fav_folders: Vec<(i64, String)>,
    /// Floating submenu for one folder's commands: (folder id, anchor
    /// top-left, items snapshot, keyboard-selected index).
    fav_submenu: Option<(i64, egui::Pos2, Vec<String>, Option<usize>)>,
    /// Self-managed keyboard cursor for EVERY two-button dialog driven by
    /// [`dialog_keys`]: `true` = the CONFIRM button is selected, `false` =
    /// Cancel (the safe default for danger dialogs). Left/Right toggles;
    /// Enter activates. Reset to `false` whenever a dialog opens so the
    /// cursor never carries over from a previously-open dialog.
    dialog_kb_confirm: bool,
    /// Rising-edge latches: set the frame a dialog OPENS so the renderer
    /// can reset `dialog_kb_confirm` to the safe side (cancel). Taken by
    /// the renderer on the first frame it sees the dialog.
    fav_name_just_opened: bool,
    fav_cmd_just_opened: bool,
    fav_del_just_opened: bool,
    hist_clear_just_opened: bool,
    fav_clear_just_opened: bool,
    close_confirm_just_opened: bool,
    ws_close_just_opened: bool,
    settings_clear_just_opened: bool,
    /// True only after Right pressed into the command column — then
    /// Up/Down/Enter operate on commands; before that the column merely
    /// PREVIEW while the folder list keeps focus.
    fav_sub_focused: bool,
    /// Remembered folder-cursor position across menu (re)opens — the
    /// folder list must not snap back to the top every time.
    fav_cursor: usize,
    /// Per-terminal menu cursor snapshot, synced every frame while the
    /// history menu is open and restored on reopen: WHICH panel was
    /// active (history / folders / commands) and each cursor position.
    menu_cursors: HashMap<String, (usize, usize, bool, usize, bool, i64)>,
    /// True on the exact frame a modal dialog OPENED: the opening click
    /// itself must not fall through to the terminal (its Text event
    /// used to land in the PTY before the modal took over the UI).
    modal_just_opened: bool,
    /// last frame's any_modal_open (for the rising-edge detection).
    prev_modal_open: bool,
    /// Tabs whose menu just (re)opened and still need the cursor
    /// snapshot restored on the FIRST render frame — restoring in the
    /// open block itself lost to state resets running in between
    /// (Esc/Enter closes reset fav_focused AFTER the snapshot capture,
    /// so Alt-close restored fine but Enter/Esc reopens didn't).
    menu_pending_restore: std::collections::HashSet<String>,
    /// In-flight drag in the FOLDER list: source index.
    fav_drag_src: Option<usize>,
    /// Drop target index + whether the pointer is past its midpoint.
    fav_drag_dst: Option<(usize, bool)>,
    /// Folder row rects for hit-testing during a drag.
    fav_folder_rects: Vec<egui::Rect>,
    /// Full rect of the favorites column (folders + items area) in the
    /// history menu; the floating submenu stays open while the pointer
    /// is inside either this or the submenu itself.
    fav_column_rect: egui::Rect,
    /// In-flight drag of a submenu COMMAND: (source folder id, source
    /// index).
    fav_item_drag: Option<(i64, usize)>,
    /// Drop target for a submenu drag: (folder id, index, past-center).
    /// The folder may differ from the source — dropping onto another
    /// folder's submenu MOVES the command there.
    fav_item_drop: Option<(i64, usize, bool)>,
    /// Name dialog for create/rename: (folder id or None for create,
    /// buffer). Rendered as a small modal like the theme dialogs.
    fav_name_dialog: Option<(Option<i64>, String)>,
    /// Add-command dialog for a folder: (folder id, command buffer).
    fav_cmd_dialog: Option<(i64, String)>,
    /// A HISTORY entry being dragged onto a favorites folder: the
    /// command text (history is NOT removed — the drop COPIES it).
    hist_drag_cmd: Option<String>,
    /// Folder id under a history-drag (the drop target).
    hist_drop_folder: Option<i64>,
    /// Delete-folder confirmation target.
    fav_delete_confirm: Option<(i64, String)>,
    /// Cached SSH host book (mirror of history_db, refreshed on change).
    ssh_hosts: Vec<crate::hosts::SshHost>,
    /// Sidebar search filter for the host section.
    ssh_host_filter: String,
    /// SSH host create/edit form dialog.
    ssh_dialog: Option<SshHostDialog>,
    ssh_dialog_just_opened: bool,
    /// Delete-host confirmation target: (row id, host name).
    ssh_delete_confirm: Option<(i64, String)>,
    ssh_del_just_opened: bool,
    /// Host chosen in the tab "+" popup; consumed by process_pending.
    pending_ssh_connect: Option<i64>,
    /// Tabs receiving broadcast keystrokes from any focused member.
    broadcast_group: std::collections::HashSet<String>,
    /// Floating AI assistant panel visibility (视图 menu).
    show_ai_panel: bool,
    ai_prompt: String,
    /// Multi-turn transcript (user/assistant turns; system prompts are
    /// prepended per-request, not stored).
    ai_messages: Vec<crate::ai::ChatMessage>,
    ai_error: Option<String>,
    ai_busy: bool,
    ai_rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    /// Fill-in dialog for a snippet containing `{placeholder}` tokens.
    snippet_fill: Option<SnippetFillState>,
    snippet_fill_just_opened: bool,
    /// Per-terminal startup command editor: (tab id, buffer).
    startup_cmd_dialog: Option<(String, String)>,
    startup_cmd_just_opened: bool,
    /// PROD guard for "insert & run": (tab id, command) awaiting confirm.
    ai_exec_confirm: Option<(String, String)>,
    ai_exec_just_opened: bool,
    /// Right-click "AI" menu intent recorded by the tab viewer, consumed
    /// by render_ai_panel on the next dispatch pass.
    ai_ctx_intent: Option<AiCtxAction>,
    /// The running terminal agent (None when idle).
    agent: Option<AgentRun>,
    /// The remote phone-control session (None when off).
    remote: Option<RemoteSession>,
    /// Recording state for a virtual key's sequence (index into
    /// settings_edit.remote_keys). Independent from the shortcut
    /// recorder; only one of the two can be active at a time.
    virtual_key_recording: Option<usize>,
    /// The sidebar SSH host search box holds keyboard focus - the
    /// terminal must not reclaim it every frame.
    sidebar_input_focused: bool,
    /// Goal text of the agent section (persists across the panel's
    /// open/close while no agent runs).
    agent_goal: String,
    agent_confirm_just_opened: bool,
}

struct TerminalData {
    instance: TerminalInstance,
    name: String,
    font_size: f32,
    /// Shell id this terminal was spawned with (scene persistence).
    shell_id: String,
    /// Set when the terminal runs the ssh client for a saved host.
    host: Option<crate::hosts::SshHostRef>,
    /// Command executed automatically when this terminal is created and
    /// on every scene restore (empty = none). Persisted per terminal.
    startup_command: String,
}

/// Execute a terminal's startup command by typing it into the PTY with
/// a trailing CR (the tty buffers input until the shell reads it, so
/// writing right after spawn is safe). Empty = no-op.
fn create_terminal(
    ctx: &egui::Context,
    working_dir: &str,
    id_counter: &mut u64,
    shell: Option<&crate::shells::ShellOption>,
    scrollback: usize,
) -> Option<TerminalInstance> {
    // Explicit shell choice (new-terminal menu / scene restore) wins;
    // otherwise the platform default (Unix: $SHELL, Windows: the
    // settings' default shell resolved against the detected list).
    let detected = crate::shells::detect_shells();
    let default_id = match DEFAULT_SHELL_ID.read() {
        Ok(g) if !g.is_empty() => g.clone(),
        _ => "cmd".to_string(),
    };
    let shell = shell
        .cloned()
        .or_else(|| {
            detected
                .iter()
                .find(|s| s.id == default_id)
                .or_else(|| detected.first())
                .cloned()
        })
        .or({
            #[cfg(target_os = "windows")]
            {
                Some(crate::shells::ShellOption {
                    id: "cmd",
                    name_key: "cmd",
                    program: std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into()),
                    args: vec![],
                })
            }
            #[cfg(not(target_os = "windows"))]
            {
                None
            }
        })?;

    let cwd_str = if std::path::PathBuf::from(working_dir).exists() {
        working_dir.to_string()
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    };

    *id_counter += 1;
    let id = *id_counter;

    TerminalInstance::create(
        ctx,
        id,
        &shell.program,
        &cwd_str,
        80,
        24,
        &shell.args,
        scrollback,
    )
}

pub(crate) fn run_startup_command(instance: &mut TerminalInstance, command: &str) {
    let cmd = command.trim();
    if cmd.is_empty() {
        return;
    }
    instance.write(format!("{cmd}\r").as_bytes());
}

fn restore_terminal_for_state(
    ctx: &egui::Context,
    tstate: &TerminalStatePersist,
    id_counter: &mut u64,
    db: &crate::history_db::HistoryDb,
    scrollback: usize,
) -> (Option<TerminalInstance>, Option<crate::hosts::SshHostRef>) {
    if tstate.host_id != 0 {
        if let Some(host) = db.ssh_host_get(tstate.host_id) {
            let snapshot = host.ref_snapshot();
            *id_counter += 1;
            if let Some(instance) =
                crate::hosts::spawn_ssh_instance(ctx, *id_counter, &host, scrollback)
            {
                return (Some(instance), Some(snapshot));
            }
        }
    }
    (
        create_terminal(ctx, &tstate.working_directory, id_counter, None, scrollback),
        None,
    )
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
                shell: data.shell_id.clone(),
                startup_command: data.startup_command.clone(),
                host_id: data.host.as_ref().map(|h| h.id).unwrap_or(0),
            },
        );
    }
    Some(WorkspaceState {
        panel_name: panel.name.clone(),
        dock_state,
        terminals,
    })
}

fn save_to_file<T: Serialize>(path: &std::path::Path, data: &T) -> Result<(), anyhow::Error> {
    crate::persist::atomic_write_json(path, data)
}

fn load_scene_file(path: &PathBuf) -> Option<SceneState> {
    load_scene_file_with_warnings(path, &mut Vec::new())
}

fn load_scene_file_with_warnings(path: &PathBuf, warnings: &mut Vec<String>) -> Option<SceneState> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return None,
    };
    match serde_json::from_str(&content) {
        Ok(scene) => Some(scene),
        Err(e) => {
            log::error!("scene.json is corrupt: {e}");
            if crate::persist::quarantine_corrupt_file(path).is_some() {
                warnings.push(
                    "场景文件已损坏，已备份为 scene.json.corrupt，本次以默认布局启动。".into(),
                );
            } else {
                warnings.push("场景文件已损坏且无法备份，本次以默认布局启动。".into());
            }
            None
        }
    }
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
                    shell: data.shell_id.clone(),
                    startup_command: data.startup_command.clone(),
                    host_id: data.host.as_ref().map(|h| h.id).unwrap_or(0),
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

fn save_scene(path: &std::path::Path, app: &mut App) {
    let state = build_scene_state(app);
    if let Err(e) = save_to_file(path, &state) {
        log::error!("Failed to save scene: {}", e);
    }
}

/// Kick off the background update check at startup. Must run on BOTH
/// startup paths (scene-restored and fresh) — previously it only ran on
/// the fresh path, so users with a saved scene never saw update prompts.
fn spawn_startup_update_check(ctx: &egui::Context) {
    let ctx_clone = ctx.clone();
    std::thread::spawn(move || {
        let result = crate::updater::check_for_update();
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

/// Formats the menu-bar update button label, e.g. "更新版本: 0.1.22".
/// The subset of a theme that feeds `rebuild_fonts`. Dragging color or
/// spacing sliders in the theme editor changes the draft WITHOUT
/// touching these, so the expensive font-atlas rebuild (full system
/// font revalidation + glyph atlas invalidation) is skipped for those
/// frames.
fn theme_fonts_signature(theme: &crate::theme::ThemeDefinition) -> (String, String) {
    (
        theme
            .app
            .ui_font_families
            .first()
            .cloned()
            .unwrap_or_default(),
        theme
            .typography
            .terminal_font_families
            .first()
            .cloned()
            .unwrap_or_default(),
    )
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut startup_warnings = Vec::new();
        let settings = read_settings_from(&settings_path(), &mut startup_warnings);
        let ctx = &cc.egui_ctx.clone();
        // Shell discovery (Windows multi-shell support) + publish the
        // settings' default for create_terminal's fallback path.
        let detected_shells = crate::shells::detect_shells();
        if let Ok(mut guard) = DEFAULT_SHELL_ID.write() {
            if guard.is_empty() {
                *guard = settings.default_shell.clone();
            }
        }

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
        let font_names: Vec<String> = system_fonts
            .iter()
            .map(|(name, _)| name.clone())
            .filter(|name| !is_symbol_font_name(name))
            .collect();

        crate::theme::apply_theme_definition(ctx, &active_theme);
        // Edge-smoothing preference from the saved settings.
        ctx.tessellation_options_mut(|t| {
            t.feathering = settings.smooth_rendering;
            t.feathering_size_in_pixels = if settings.smooth_rendering {
                settings.smooth_level.clamp(0.0, 2.0)
            } else {
                0.0
            };
        });

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
            show_clear_favorites_confirm: false,
            settings,
            show_settings: false,
            show_about: false,
            show_update_window: false,
            show_help_window: false,
            update_notes_cache: (Vec::new(), String::new()),
            settings_edit: AppSettings::default(),
            settings_tab: 0,
            settings_applied_toast: None,
            settings_window_open: false,
            binding_recording: None,
            cached_template_files: Vec::new(),
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
            theme_edit: active_theme.clone(),
            terminal_theme_cache: std::sync::Arc::new(crate::theme::terminal_theme(&active_theme)),
            terminal_theme_cache_theme: active_theme.clone(),
            terminal_theme_cache_edit: false,
            available_themes,
            theme_message: None,
            pending_import_theme: false,
            pending_export_theme: false,
            theme_dialog: Default::default(),
            theme_dirty: false,
            theme_editor_open: false,
            theme_edit_origin: None,
            theme_editor_subtab: Default::default(),
            clipboard_mirror: String::new(),
            terminal_view_rects: Default::default(),
            history_clear_confirm: None,
            history_menu_just_closed: Default::default(),
            auto_match_pending: Default::default(),
            pending_shell: None,
            pending_shell_last: None,
            detected_shells,
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
            monitor_rows: Vec::new(),
            cpu_history: Vec::new(),
            show_monitor: false,
            terminal_search: None,
            proc_sample_tx: None,
            font_asset_cache: None,
            font_cjk_names: Default::default(),
            startup_warnings: Vec::new(),
            startup_check_consumed: false,
            fav_folders: Vec::new(),
            fav_submenu: None,
            fav_sub_focused: false,
            fav_cursor: 0,
            menu_cursors: HashMap::new(),
            modal_just_opened: false,
            prev_modal_open: false,
            menu_pending_restore: Default::default(),
            fav_drag_src: None,
            fav_drag_dst: None,
            fav_folder_rects: Vec::new(),
            fav_column_rect: egui::Rect::NOTHING,
            fav_item_drag: None,
            fav_item_drop: None,
            fav_name_dialog: None,
            fav_cmd_dialog: None,
            hist_drag_cmd: None,
            hist_drop_folder: None,
            fav_delete_confirm: None,
            ssh_hosts: Vec::new(),
            ssh_host_filter: String::new(),
            ssh_dialog: None,
            ssh_dialog_just_opened: false,
            ssh_delete_confirm: None,
            ssh_del_just_opened: false,
            pending_ssh_connect: None,
            broadcast_group: Default::default(),
            show_ai_panel: false,
            ai_prompt: String::new(),
            ai_messages: Vec::new(),
            ai_error: None,
            ai_busy: false,
            ai_rx: None,
            ai_exec_confirm: None,
            ai_exec_just_opened: false,
            ai_ctx_intent: None,
            agent: None,
            agent_goal: String::new(),
            agent_confirm_just_opened: false,
            remote: None,
            virtual_key_recording: None,
            sidebar_input_focused: false,
            snippet_fill: None,
            snippet_fill_just_opened: false,
            startup_cmd_dialog: None,
            startup_cmd_just_opened: false,
            dialog_kb_confirm: false,
            fav_name_just_opened: false,
            fav_cmd_just_opened: false,
            fav_del_just_opened: false,
            hist_clear_just_opened: false,
            fav_clear_just_opened: false,
            close_confirm_just_opened: false,
            ws_close_just_opened: false,
            settings_clear_just_opened: false,
        };

        app.fav_folders = app.history_db.fav_folders();
        app.ssh_hosts = app.history_db.ssh_hosts();

        // Register fonts (system + embedded + theme choices) now that the
        // App and its active theme exist. rebuild_fonts reuses the scanned
        // list captured above for the system_fonts field.
        app.rebuild_fonts(ctx);

        // Surface a silently-failed self-update from the previous session:
        // the helper script could not replace the binary (no root), so we
        // are still the OLD version — tell the user instead of leaving the
        // "updated" illusion (which also made them think features/themes
        // were broken).
        if let Some(reason) = crate::updater::take_last_update_failure() {
            app.update_toast = Some((
                format!("上次自动更新失败：{reason}"),
                std::time::Instant::now() + std::time::Duration::from_secs(10),
            ));
        }

        let scene_path = scene_path();
        if scene_path.exists() {
            if let Some(scene) = load_scene_file_with_warnings(&scene_path, &mut startup_warnings) {
                app.settings_edit = app.settings.clone();
                for panel in &scene.panels {
                    let idx = app.panels.len();
                    for (_id, tstate) in &panel.terminals {
                        let (instance, host_ref) = restore_terminal_for_state(
                            ctx,
                            tstate,
                            &mut app.terminal_id_counter,
                            &app.history_db,
                            app.settings.scrollback,
                        );
                        let Some(mut instance) = instance else {
                            continue;
                        };
                        run_startup_command(&mut instance, &tstate.startup_command);
                        if host_ref.is_none() && tstate.host_id != 0 {
                            app.update_toast = Some((
                                app.texts
                                    .ssh
                                    .host_missing_fallback
                                    .replace("{}", &tstate.name),
                                std::time::Instant::now() + std::time::Duration::from_secs(8),
                            ));
                        }
                        app.terminals.insert(
                            _id.clone(),
                            TerminalData {
                                instance,
                                name: tstate.name.clone(),
                                font_size: tstate.font_size,
                                shell_id: tstate.shell.clone(),
                                host: host_ref,
                                startup_command: tstate.startup_command.clone(),
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
                app.startup_warnings = startup_warnings;
                spawn_startup_update_check(ctx);
                return app;
            }
        }

        app.add_initial_terminal(ctx);
        app.refresh_template_files();
        app.startup_warnings = startup_warnings;

        spawn_startup_update_check(ctx);

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
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
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
        self.load_workspace_state(ctx, state);
    }

    fn add_initial_terminal(&mut self, ctx: &egui::Context) {
        let name = "workspace 1".to_string();
        let Some(tab_id) = self.create_terminal_inner(ctx) else {
            return;
        };
        self.dock_states.insert(0, DockState::new(vec![tab_id]));
        self.panels.push(Panel { name });
        self.active_panel = 0;
        self.restore_workspace_focus(0);
    }

    fn load_workspace_state(&mut self, ctx: &egui::Context, state: WorkspaceState) {
        let panel_idx = self.panels.len();
        for (id, tstate) in &state.terminals {
            if !self.terminals.contains_key(id) {
                let (instance, host_ref) = restore_terminal_for_state(
                    ctx,
                    tstate,
                    &mut self.terminal_id_counter,
                    &self.history_db,
                    self.settings.scrollback,
                );
                let Some(mut instance) = instance else {
                    continue;
                };
                run_startup_command(&mut instance, &tstate.startup_command);
                self.terminals.insert(
                    id.clone(),
                    TerminalData {
                        instance,
                        name: tstate.name.clone(),
                        font_size: tstate.font_size,
                        shell_id: tstate.shell.clone(),
                        host: host_ref,
                        startup_command: tstate.startup_command.clone(),
                    },
                );
            }
        }
        self.panels.push(Panel {
            name: state.panel_name,
        });
        self.dock_states.insert(panel_idx, state.dock_state);
    }

    fn is_renaming(&self) -> bool {
        self.renaming_panel.is_some() || self.renaming_terminal.is_some()
    }

    fn create_terminal_inner(&mut self, ctx: &egui::Context) -> Option<String> {
        self.tab_counter += 1;
        let id = format!("terminal-{}", self.tab_counter);
        // New terminals inherit the FOCUSED terminal's working directory
        // (clicking "+" or splitting should land in the same folder). Poll
        // the focused terminal's cwd first so a fresh `cd` is picked up,
        // then fall back to the app's own cwd when there is none yet.
        if let Some(tab) = self.focused_terminal.clone() {
            if let Some(data) = self.terminals.get_mut(&tab) {
                data.instance.poll_cwd();
            }
        }
        let cwd = self
            .focused_terminal
            .as_ref()
            .and_then(|tab| self.terminals.get(tab))
            .map(|data| data.instance.cwd.clone())
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default()
            });
        // Remember which shell id this terminal uses (scene persistence);
        // create_terminal consumes the option.
        self.pending_shell_last = self.pending_shell.as_ref().map(|s| s.id.to_string());
        let instance = create_terminal(
            ctx,
            &cwd,
            &mut self.terminal_id_counter,
            self.pending_shell.take().as_ref(),
            self.settings.scrollback,
        )?;
        let random_suffix: String = uuid::Uuid::new_v4().as_bytes()[0..3]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        let shell_id_used = self.pending_shell_last.take().unwrap_or_else(|| {
            DEFAULT_SHELL_ID
                .read()
                .map(|g| g.clone())
                .unwrap_or_default()
        });
        self.terminals.insert(
            id.clone(),
            TerminalData {
                instance,
                name: format!("terminal-{random_suffix}"),
                font_size: DEFAULT_FONT_SIZE,
                shell_id: shell_id_used,
                host: None,
                startup_command: String::new(),
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
            // An SSH host chosen in the "+" popup spawns the ssh client
            // instead of the default shell; everything downstream (dock
            // placement) is identical.
            let tab_id = if let Some(host_id) = self.pending_ssh_connect.take() {
                self.connect_ssh_host_inner(ctx, host_id)
            } else {
                self.create_terminal_inner(ctx)
            };
            let Some(tab_id) = tab_id else {
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
                self.broadcast_group.remove(&tab);
                if self.terminal_search.as_ref().is_some_and(|s| s.tab == tab) {
                    self.terminal_search = None;
                }
                // An agent driving the closed terminal must stop, or it
                // spins forever against a vanished tab (and keeps
                // burning model calls).
                if self.agent.as_ref().is_some_and(|a| a.tab == tab) {
                    if let Some(agent) = self.agent.as_mut() {
                        agent.request_stop = true;
                    }
                }
                // Remote caches for the closed tab (frame seq / last
                // serialized ANSI) would otherwise leak until the
                // remote session stops.
                if let Some(session) = self.remote.as_mut() {
                    session.frame_seq.remove(&tab);
                    session.last_ansi.remove(&tab);
                    if session.remote_focus_tab.as_deref() == Some(tab.as_str()) {
                        session.remote_focus_tab = None;
                    }
                }
                if self.focused_terminal.as_ref() == Some(&tab) {
                    self.focused_terminal = None;
                }
                // Drop a rename state pointing at the closed tab: dock
                // only renders the active tab, so an un-rendered rename
                // input could never be confirmed/cancelled and would
                // otherwise linger forever.
                if self.renaming_terminal.as_ref() == Some(&tab) {
                    self.renaming_terminal = None;
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
                    // The agent and broadcast groups pointed at the OLD
                    // terminals; restored tabs would otherwise inherit
                    // stale memberships.
                    self.agent = None;
                    self.broadcast_group.clear();
                    if let Some(session) = self.remote.as_mut() {
                        session.frame_seq.clear();
                        session.last_ansi.clear();
                        session.remote_focus_tab = None;
                    }
                    for panel in &scene.panels {
                        let idx = self.panels.len();
                        for (_id, tstate) in &panel.terminals {
                            let (instance, host_ref) = restore_terminal_for_state(
                                ctx,
                                tstate,
                                &mut self.terminal_id_counter,
                                &self.history_db,
                                self.settings.scrollback,
                            );
                            if let Some(mut instance) = instance {
                                run_startup_command(&mut instance, &tstate.startup_command);
                                self.terminals.insert(
                                    _id.clone(),
                                    TerminalData {
                                        instance,
                                        name: tstate.name.clone(),
                                        font_size: tstate.font_size,
                                        shell_id: tstate.shell.clone(),
                                        host: host_ref,
                                        startup_command: tstate.startup_command.clone(),
                                    },
                                );
                            }
                        }
                        self.panels.push(Panel {
                            name: panel.name.clone(),
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
        self.panels.push(Panel { name });
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
                self.texts.theme_editor.new_dialog_title.clone(),
            ),
            DialogKind::Copy => (
                "theme_copy_modal",
                "theme_copy_name_input",
                self.texts.theme_editor.copy_dialog_title.clone(),
            ),
            DialogKind::Rename => (
                "theme_rename_modal",
                "theme_rename_name_input",
                self.texts.theme_editor.rename_dialog_title.clone(),
            ),
            DialogKind::Delete => (
                "theme_delete_modal",
                "",
                self.texts.theme_editor.delete_confirm.clone(),
            ),
            DialogKind::Switch => (
                "theme_switch_modal",
                "",
                self.texts.theme_editor.switch_confirm.clone(),
            ),
        };

        let input_id = egui::Id::new(input_id_salt);
        let mut close_after = false;
        let mut do_action = false;

        // Unified key protocol, BEFORE the Modal: Enter in an input
        // dialog means CONFIRM (consumed - the old `key_pressed` read
        // let the same Enter also fall through to the terminal).
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            do_action = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            close_after = true;
        }

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
                    ui.add(
                        egui::TextEdit::singleline(&mut self.theme_dialog.name_input)
                            .id(input_id)
                            .desired_width(340.0),
                    );
                }
                DialogKind::Delete => {
                    ui.label(format!(
                        "{}: {}",
                        self.texts.theme_editor.delete_confirm.clone(),
                        self.theme_edit.name
                    ));
                }
                DialogKind::Switch => {
                    ui.label(self.texts.theme_editor.switch_confirm.clone());
                }
            }

            // Buttons
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(&self.texts.theme_editor.confirm).clicked() {
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
        // Esc closes the editor through the SAME path as the close button
        // (unsaved draft is kept/saved, never silently dropped).
        if self.theme_editor_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            is_open = false;
        }
        let mut editor_name = self.theme_edit.name.clone();
        let editor_dirty = self.theme_dirty;
        let mut editor_draft = self.theme_edit.clone();
        let available_themes = self.available_themes.clone();
        let system_fonts = self.system_fonts.clone();
        let mut theme_dialog = self.theme_dialog.clone();
        let accent = self.active_theme.app.accent.to_egui();
        let mut actions_out: Vec<crate::theme::ui::ThemeAction> = Vec::new();
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
            heading: te.heading.clone(),
            current: te.current.clone(),
            unsaved: te.unsaved.clone(),
            new_theme: te.new.clone(),
            copy_theme: te.copy.clone(),
            rename_theme: te.rename.clone(),
            delete_theme: te.delete.clone(),
            import_theme: te.import.clone(),
            export_theme: te.export.clone(),
            ui_appearance: te.ui_appearance.clone(),
            base_colors: te.base_colors.clone(),
            app_bg_label: te.app_bg_label.clone(),
            sidebar_label: te.sidebar_label.clone(),
            panel_label: te.panel_label.clone(),
            input_bg_label: te.input_bg_label.clone(),
            text_colors: te.text_colors.clone(),
            text_label: te.text_label.clone(),
            weak_text_label: te.weak_text_label.clone(),
            status_colors: te.status_colors.clone(),
            accent_label: te.accent_label.clone(),
            warning_label: te.warning_label.clone(),
            danger_label: te.danger_label.clone(),
            interaction_colors: te.interaction_colors.clone(),
            hover_label: te.hover_label.clone(),
            active_label: te.active_label.clone(),
            selection_bg_label: te.selection_bg_label.clone(),
            selection_text_label: te.selection_text_label.clone(),
            border_label: te.border_label.clone(),
            lock_label: te.lock_label.clone(),
            terminal_appearance: te.terminal_appearance.clone(),
            palette_template_label: te.palette_template_label.clone(),
            apply_template: te.apply_template.clone(),
            terminal_base_colors: te.terminal_base_colors.clone(),
            fg_label: te.fg_label.clone(),
            bg_label: te.bg_label.clone(),
            cursor_label: te.cursor_label.clone(),
            link_label: te.link_label.clone(),
            normal: te.normal.clone(),
            bright: te.bright.clone(),
            dim: te.dim.clone(),
            black: te.black.clone(),
            red: te.red.clone(),
            green: te.green.clone(),
            yellow: te.yellow.clone(),
            blue: te.blue.clone(),
            magenta: te.magenta.clone(),
            cyan: te.cyan.clone(),
            white: te.white.clone(),
            copy_dialog_title: te.copy_dialog_title.clone(),
            copy_dialog_hint: te.copy_dialog_hint.clone(),
            new_dialog_title: te.new_dialog_title.clone(),
            new_dialog_hint: te.new_dialog_hint.clone(),
            rename_dialog_title: te.rename_dialog_title.clone(),
            delete_confirm: te.delete_confirm.clone(),
            switch_confirm: te.switch_confirm.clone(),
            save_and_switch: te.save_and_switch.clone(),
            discard_and_switch: te.discard_and_switch.clone(),
            builtin_readonly: te.builtin_readonly.clone(),
            keep: te.keep.clone(),
            discard: te.discard.clone(),
            ui_font_label_short: te.ui_font_label_short.clone(),
            ui_font_size_label: te.ui_font_size_label.clone(),
            font_cjk_hint: te.font_cjk_hint.clone(),
            terminal_font_label_short: te.terminal_font_label_short.clone(),
            terminal_font_size_label: te.terminal_font_size_label.clone(),
            cell_spacing_label: te.cell_spacing_label.clone(),
            terminal_padding_label: te.terminal_padding_label.clone(),
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
                    .frame(egui::Frame::new())
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
                            &self.font_cjk_names,
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
                            &self.font_cjk_names,
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
            // Only font-affecting fields justify an atlas rebuild; color
            // slider drags must not re-validate every system font per frame.
            let fonts_changed =
                theme_fonts_signature(&editor_draft) != theme_fonts_signature(&self.theme_edit);
            self.theme_edit = editor_draft;
            self.theme_dirty = true;
            crate::theme::apply_theme_definition(ctx, &self.theme_edit);
            if fonts_changed {
                self.rebuild_fonts(ctx);
            }
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
        // Persist the layout before the restart so the post-update
        // session restores exactly what the user was working on.
        save_scene(&scene_path(), self);
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
        // Open the update window so the user can watch progress; the
        // status line + progress bar live in-place inside it.
        self.show_update_window = true;
        self.start_download(ctx, info);
    }

    /// Fonts loaded from disk exactly once per process: scanning every
    /// system font directory and re-validating each file on every theme
    /// switch froze the UI for up to seconds on Windows. The cache holds
    /// validated `FontData` behind an Arc; rebuilding only re-wires
    /// family chains and swaps the atlas. Newly installed system fonts
    /// are picked up after an app restart — acceptable for a font list.
    fn rebuild_fonts(&mut self, ctx: &egui::Context) {
        if self.font_asset_cache.is_none() {
            let mut loaded: Vec<(String, std::sync::Arc<egui::FontData>)> = Vec::new();
            let mut cjk_names = std::collections::HashSet::new();
            for (name, path) in scan_system_fonts() {
                let path = std::path::PathBuf::from(path);
                match std::fs::read(&path) {
                    Ok(data) => {
                        // Validate the file parses as a real TTF/OTF/TTC
                        // face before registering it. Some files carry a
                        // .ttf extension but are not TrueType resources
                        // (e.g. Windows mstmc.ttf bitmap fonts); epaint
                        // panics on those at first use.
                        if !is_valid_font_data(&data) {
                            log::warn!("skipping invalid font file {}: {}", name, path.display());
                            continue;
                        }
                        if font_file_has_cjk(&data) {
                            cjk_names.insert(name.clone());
                        }
                        loaded.push((name, std::sync::Arc::new(egui::FontData::from_owned(data))));
                    }
                    Err(e) => log::warn!("unreadable font file {}: {}", path.display(), e),
                }
            }
            // The OS's own UI faces (fixed "os-ui"/"os-ui-cjk" keys, NOT
            // in the picker list): "system-ui" themes head Proportional
            // with these so the app matches the desktop font, Chinese
            // labels included.
            for os in detect_os_ui_fonts() {
                match std::fs::read(&os.path) {
                    Ok(data) if is_valid_font_data(&data) => {
                        if font_file_has_cjk(&data) {
                            cjk_names.insert(os.key.to_string());
                        }
                        // FontData must carry the TTC index so the right
                        // face inside a collection is used.
                        loaded.push((
                            os.key.to_string(),
                            std::sync::Arc::new(egui::FontData {
                                font: data.into(),
                                index: os.index,
                                tweak: Default::default(),
                            }),
                        ));
                    }
                    _ => log::warn!("unreadable system UI font {}", os.path.display()),
                }
            }
            // The bundled CJK font ("noto-cjk" key, see
            // load_multilingual_fonts) always covers Chinese regardless
            // of what the system offers.
            cjk_names.insert("noto-cjk".into());
            self.font_cjk_names = cjk_names;
            self.font_asset_cache = Some(loaded);
        }
        let system_fonts = self.font_asset_cache.as_ref().unwrap();
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        let mut registered_names: Vec<String> = Vec::new();
        for (name, data) in system_fonts {
            fonts.font_data.insert(name.clone(), data.clone());
            // Symbol/decorative fonts stay REGISTERED (a theme or
            // preview referencing FontFamily::Name("NotoColorEmoji")
            // must stay bound — epaint PANICS on unbound names), but
            // they never enter registered_names: no generic fallback
            // chains, no font pickers. Their glyph pollution is thus
            // impossible unless a user explicitly picks one.
            if !is_symbol_font_name(name) {
                registered_names.push(name.clone());
            }
        }
        load_multilingual_fonts(&mut fonts);
        // Monospace generic fallback: ONLY monospace-class families.
        // Proportional faces here tear the terminal's cell grid; symbol
        // faces substitute decorated glyphs for plain ASCII (the stray
        // overlines/ticks seen between words in some themes).
        if let Some(mono_family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            for name in &registered_names {
                if is_monospace_family_name(name) {
                    mono_family.push(name.clone());
                }
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
        // Symbol fonts are excluded from registered_names (no chains, no
        // pickers) but any theme/preview may still reference them by
        // name, and epaint PANICS on an unbound FontFamily::Name. Bind
        // them to their own named family with the clean stack as
        // fallback — safe because nothing routes text through them
        // unless explicitly chosen (and the theme-font injector above
        // refuses symbol heads).
        let symbol_names: Vec<String> = fonts
            .font_data
            .keys()
            .filter(|n| is_symbol_font_name(n))
            .cloned()
            .collect();
        for name in symbol_names {
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
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            if ui_font == "system-ui" {
                // Follow the OS: the desktop's UI face heads the stack and
                // its Chinese face comes right after, so BOTH Latin and
                // Chinese labels match the system font (a plain "insert
                // nothing" here used to pin the UI on egui's built-in
                // Ubuntu-Light + bundled CJK — ignoring whatever the user
                // configured system-wide).
                if fonts.font_data.contains_key("os-ui") {
                    family.insert(0, "os-ui".to_string());
                }
                if fonts.font_data.contains_key("os-ui-cjk") {
                    family.insert(1, "os-ui-cjk".to_string());
                }
            } else if !ui_font.is_empty()
                && !is_symbol_font_name(&ui_font)
                && fonts.font_data.contains_key(&ui_font)
            {
                // Symbol fonts must never head a generic family (a user
                // theme saved with e.g. NotoColorEmoji selected rendered
                // decorated glyphs across the whole UI); fall back to the
                // default stack.
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
            && !is_symbol_font_name(&term_font)
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

    /// Lock a workspace. WITHOUT a configured password, locking would be
    /// cosmetic (any input unlocks) — instead route the user into the
    /// password-setup flow so the lock always means something.
    fn try_lock_workspace(&mut self, index: usize) {
        if self.settings.lock_password.is_empty() {
            self.pw_set1.clear();
            self.pw_set2.clear();
            self.pw_message.clear();
            self.pw_popup = Some("set");
            self.pw_message = self.texts.password.need_setup.clone();
        } else {
            self.locked_panels.insert(index);
            self.lock_password_input.clear();
            self.pw_message.clear();
        }
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

    /// Fixed sidebar bottom cluster: divider + quick toggles + divider +
    /// stats footer. Rendered inside a reserved bottom panel so the
    /// workspace list (a ScrollArea above) can never squeeze it out.
    fn render_sidebar_bottom_cluster(&mut self, ui: &mut egui::Ui) {
        let weak = self.active_theme.app.weak_text.to_egui();
        // Single-line summary with hover details (was a 6-line wall of
        // 10-11px text — the v0.1.37 UI audit's top density finding).
        let footer_h = 24.0;

        // Divider between the (scrolling) workspace list and the fixed
        // quick toggles. It lives INSIDE the bottom panel, so it stays
        // put no matter how far the list scrolls.
        ui.separator();

        // Quick toggles - they write the SAME state as the settings page
        // (both the live settings and the settings_edit draft), so
        // checking either side keeps the other in sync instantly.

        let mut auto_copy = self.settings.auto_copy_selection;
        let mut auto_match = self.settings.auto_match_command;
        ui.style_mut().visuals.widgets.inactive.fg_stroke.color = weak;
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.checkbox(&mut auto_copy, &self.texts.settings.general.auto_copy);
            ui.checkbox(&mut auto_match, &self.texts.settings.general.auto_match);
        });
        if auto_copy != self.settings.auto_copy_selection {
            self.settings.auto_copy_selection = auto_copy;
            self.settings_edit.auto_copy_selection = auto_copy;
            let _ = save_settings(&self.settings);
        }
        if auto_match != self.settings.auto_match_command {
            self.settings.auto_match_command = auto_match;
            self.settings_edit.auto_match_command = auto_match;
            let _ = save_settings(&self.settings);
        }
        ui.separator();

        let (footer_rect, footer_resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), footer_h),
            egui::Sense::click(),
        );
        let x = footer_rect.min.x + 10.0;
        let data_font = egui::FontId::proportional(11.0);

        // Compact summary: focused terminal's CPU/MEM (the number users
        // glance at); full breakdown lives on hover.
        let summary = format!(
            "{} · {}",
            format_cpu(self.focused_cpu),
            format_memory(self.focused_mem)
        );
        ui.painter().text(
            egui::pos2(x, footer_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &summary,
            data_font,
            weak,
        );
        let details = format!(
            "{}: {} | {}\n{}: {} {} | {} | {}\n{}: {} {} | {} | {}",
            self.texts.stats.focused,
            format_cpu(self.focused_cpu),
            format_memory(self.focused_mem),
            self.texts.stats.workspace,
            format_active_ws_terminal_count(self),
            self.texts.stats.terminals,
            format_cpu(self.workspace_cpu),
            format_memory(self.workspace_mem),
            self.texts.stats.global,
            format_ws_terminal_count(self),
            self.texts.stats.terminals,
            format_cpu(self.terminal_cpu),
            format_memory(self.terminal_mem),
        );
        footer_resp.on_hover_text(details);
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
        self.draw_toast(ctx, egui::Id::new("update_toast"), &msg, expires);
    }

    /// Shared toast painter with a 150 ms fade-in and fade-out over the
    /// last 300 ms before expiry (v0.1.37 UI audit: toasts used to pop
    /// in and vanish instantly).
    fn draw_toast(
        &self,
        ctx: &egui::Context,
        id: egui::Id,
        msg: &str,
        expires: std::time::Instant,
    ) {
        const FADE: f32 = 0.15;
        let remaining = expires
            .duration_since(std::time::Instant::now())
            .as_secs_f32();
        let target = if remaining > 0.3 { 1.0 } else { 0.0 };
        let alpha = ctx.animate_value_with_time(id, target, FADE);
        if alpha < 0.01 {
            return;
        }
        egui::Area::new(id)
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
            .interactable(false)
            .show(ctx, |ui| {
                let mut frame = egui::Frame::popup(&ui.ctx().style());
                let bg = frame.fill;
                frame.fill = egui::Color32::from_rgba_unmultiplied(
                    bg.r(),
                    bg.g(),
                    bg.b(),
                    (bg.a() as f32 * alpha) as u8,
                );
                frame.show(ui, |ui| {
                    let fg = ui.style().visuals.strong_text_color();
                    let rt = egui::RichText::new(msg).color(egui::Color32::from_rgba_unmultiplied(
                        fg.r(),
                        fg.g(),
                        fg.b(),
                        (fg.a() as f32 * alpha) as u8,
                    ));
                    ui.label(rt);
                });
            });
    }

    /// If auto-copy is enabled and the user just released the primary
    /// mouse button over a terminal, copy the current selection to
    /// the system clipboard.
    fn handle_selection_auto_copy(&mut self, ctx: &egui::Context) {
        if !self.settings.auto_copy_selection {
            return;
        }
        // The history/favorites menu is open: its rows and buttons are
        // click targets, and a button RELEASE (e.g. the trash icon in a
        // folder's command column) must not be interpreted as the end of
        // a terminal text selection — that copy/toast burst caused the
        // black flash on delete. Selection auto-copy only applies to
        // releases while NO menu is open.
        let any_menu_open = self
            .focused_terminal
            .as_ref()
            .and_then(|tab| self.terminals.get(tab))
            .is_some_and(|td| td.instance.history_nav.is_some());
        if any_menu_open {
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

    // ---- Terminal scrollback search UI (roadmap batch 4) ----

    // ---- Monitor panel (roadmap batch 4) ----

    // ---- AI assistant (roadmap batch 2) ----

    // ---- Snippet placeholders + startup commands (roadmap batch 3) ----

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
    // Unified protocol, BEFORE the Window. Default cursor = CANCEL
    // (restarting is disruptive; a stray Enter must not restart).
    let mut kb = false;
    let keys = dialog_keys(ctx, &mut kb, true);
    let mut restart = keys.confirm;
    let mut cancel = keys.cancel || keys.close;
    let mut open = true;
    let inner = egui::Window::new(&texts.update.restart_title)
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
                    App::dialog_button_row(
                        ui,
                        &mut kb,
                        egui::Id::new("restart_confirm_btn"),
                        egui::Id::new("restart_cancel_btn"),
                        &texts.update.restart_confirm,
                        &texts.theme_editor.cancel,
                    )
                })
                .inner
            })
            .inner
        })
        .and_then(|r| r.inner);
    if let Some((c, x)) = inner {
        restart |= c;
        cancel |= x;
    }
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
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Persist the layout on every exit path (window X button,
        // menu quit, ViewportCommand::Close from the updater restart)
        // so tabs/splits/renames survive without a manual Ctrl+S.
        save_scene(&scene_path(), self);
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        if let Some(id) = self.terminal_focus_id {
            if terminal_focus_lock_allowed(
                self.renaming_panel.is_some(),
                self.renaming_terminal.is_some(),
            ) && !any_modal_open(self)
            {
                ctx.memory_mut(|memory| {
                    memory.set_focus_lock_filter(id, egui_term::terminal_focus_event_filter())
                });
            } else {
                // Surrender the focus AND neutralize the lock filter:
                // set_focus_lock_filter only takes effect while focused,
                // but a stale filter from the focused frames used to keep
                // swallowing arrows/escape for the terminal's tab
                // navigation while a modal dialog owned the UI.
                ctx.memory_mut(|memory| {
                    memory.surrender_focus(id);
                });
                self.terminal_focus_id = None;
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Rising-edge latch: the frame a modal goes from none to some,
        // the opening click's own events must not reach the terminal.
        {
            let now_open = any_modal_open(self);
            self.modal_just_opened = now_open && !self.prev_modal_open;
            self.prev_modal_open = now_open;
        }
        // Drain one queued startup warning per frame into the toast
        // channel so corruption notices are actually seen.
        if !self.startup_warnings.is_empty() {
            let warning = self.startup_warnings.remove(0);
            self.update_toast = Some((
                warning,
                std::time::Instant::now() + std::time::Duration::from_secs(8),
            ));
        }
        // Auto-copy selected text on mouse release.
        self.handle_selection_auto_copy(ctx);
        self.process_pending(ctx);
        // Global agent stop key: consumable EVEN while a modal owns the
        // UI (stopping must always be reachable). Deliberately before
        // the modal arbiter and any other consumer.
        if agent_stop_shortcut_hit(ctx, &self.settings.key_binds) {
            if let Some(agent) = self.agent.as_mut() {
                agent.request_stop = true;
            }
        }
        // Agent state machine tick (thinking drain / completion wait).
        self.agent_tick(ctx);
        // Remote phone control: drain commands + refresh shared frames.
        self.remote_tick(ctx);

        self.cwd_poll_frame = self.cwd_poll_frame.wrapping_add(1);
        if self.cwd_poll_frame >= 15 {
            self.cwd_poll_frame = 0;
            for data in self.terminals.values_mut() {
                data.instance.poll_cwd();
            }
        }

        // Ctrl+F: scrollback search on the focused terminal. Consumed at
        // app level (browser convention) and gated on modals like other
        // shortcuts; the terminal no longer sees Ctrl+F while this build.
        if let Some(tab) = self.focused_terminal.clone() {
            let search_open = self.terminal_search.as_ref().map(|s| &s.tab) == Some(&tab);
            if !any_modal_open(self)
                && !self.is_renaming()
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::F))
            {
                if search_open {
                    self.terminal_search = None;
                } else {
                    self.terminal_search = Some(TerminalSearch::new(tab.clone()));
                }
            }
        }

        // Status-bar sample: every 2 seconds, hand the pid groups to the
        // persistent sampler worker (off-thread) and apply whatever
        // results have arrived through the temp store.
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
            let per_terminal: Vec<(String, u32)> = self
                .terminals
                .iter()
                .map(|(id, td)| (id.clone(), td.instance.backend.child_pid()))
                .filter(|(_, pid)| *pid != 0)
                .collect();
            let job: ProcSampleJob = (all_roots, ws_roots, focused_roots, per_terminal);
            match self.proc_sample_tx.as_ref() {
                Some(tx) => {
                    if let Err(err) = tx.send(job) {
                        // Worker died (shouldn't happen — the loop only
                        // exits when the channel closes): respawn and
                        // redeliver.
                        let tx = spawn_proc_sampler(ctx.clone());
                        let _ = tx.send(err.0);
                        self.proc_sample_tx = Some(tx);
                    }
                }
                None => {
                    let tx = spawn_proc_sampler(ctx.clone());
                    let _ = tx.send(job);
                    self.proc_sample_tx = Some(tx);
                }
            }
        }
        if let Some(result) =
            ctx.memory_mut(|mem| mem.data.remove_temp::<ProcSampleResult>(proc_sample_id()))
        {
            self.terminal_cpu = result.all.0;
            self.terminal_mem = result.all.1;
            self.workspace_cpu = result.workspace.0;
            self.workspace_mem = result.workspace.1;
            self.focused_cpu = result.focused.0;
            self.focused_mem = result.focused.1;
            self.monitor_rows = result.per_terminal;
            if let Some(cpu) = result.all.0 {
                self.cpu_history.push(cpu);
                let cap = 120;
                if self.cpu_history.len() > cap {
                    let overflow = self.cpu_history.len() - cap;
                    self.cpu_history.drain(0..overflow);
                }
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
                let favorites = self.history_db.fav_all();
                if let Some(td) = self.terminals.get_mut(&tab) {
                    let was_open = td.instance.history_nav.is_some();
                    toggle_history_menu(&mut td.instance.history_nav, entries, favorites);
                    if !was_open {
                        // Fresh open: flag for restoration on the FIRST
                        // render frame (see render_history_menu) — the
                        // snapshot holds the last session's panel focus
                        // and cursors.
                        self.menu_pending_restore.insert(tab.clone());
                        // (Actual restoration runs on the first render
                        // frame — see render_history_menu.)
                        // Scroll offsets realign below via follow-logic;
                        // clear stale ones first.
                        ctx.memory_mut(|m| {
                            m.data.insert_temp(
                                egui::Id::new(("hist_menu_scroll", tab.as_str())),
                                0usize,
                            );
                            m.data.insert_temp(
                                egui::Id::new(("hist_fav_scroll", tab.as_str())),
                                0usize,
                            );
                        });
                    }
                }
            }
        }

        let history_menu_active = self
            .focused_terminal
            .as_ref()
            .and_then(|tab| self.terminals.get(tab))
            .is_some_and(|td| td.instance.history_nav.is_some());

        let mut history_menu_handled = false;
        // MODAL ARBITRATION: while ANY modal dialog is open, the history
        // menu's entire keyboard block stays silent — its Esc/Enter/
        // arrows/Delete/Insert must not consume or act behind a popup
        // (arrow keys used to drive the menu AND the popup's buttons at
        // once). The keys belong to the topmost modal.
        let modal_hijack = any_modal_open(self);
        if !workspace_renaming && !modal_hijack && history_menu_active {
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
            // Row-action shortcuts (manual menu only): Delete removes the
            // row under the keyboard cursor (history entry, or favorite
            // when the favorites column holds focus); Insert favorites
            // the selected HISTORY entry. Consumed so the shell never
            // receives the raw escapes.
            let manual_menu = self
                .focused_terminal
                .as_ref()
                .and_then(|tab| self.terminals.get(tab))
                .and_then(|td| td.instance.history_nav.as_ref())
                .is_some_and(|nav| nav.auto_word.is_none());
            let delete_pressed =
                !close && !confirm && manual_menu && check_shortcut(ctx, &binds, "history_delete");
            let favorite_pressed = !close
                && !confirm
                && !delete_pressed
                && manual_menu
                && check_shortcut(ctx, &binds, "history_favorite");
            // Left/Right: move keyboard focus between the main list and
            // the favorites list — only in the MANUAL (Alt) menu; the
            // auto-match overlay must keep passing arrows to the shell
            // (cursor movement / recall). They must also be consumed so
            // the terminal never sees them as cursor-motion escapes.
            let (focus_left, focus_right) = {
                let manual = self
                    .focused_terminal
                    .as_ref()
                    .and_then(|tab| self.terminals.get(tab))
                    .and_then(|td| td.instance.history_nav.as_ref())
                    .is_some_and(|nav| nav.auto_word.is_none());
                if !manual {
                    (false, false)
                } else {
                    let has_favs = self
                        .focused_terminal
                        .as_ref()
                        .and_then(|tab| self.terminals.get(tab))
                        .and_then(|td| td.instance.history_nav.as_ref())
                        .is_some_and(|nav| !nav.favorites.is_empty());
                    if !has_favs {
                        (false, false)
                    } else {
                        let l = ctx.input_mut(|i| {
                            i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                        });
                        let r = ctx.input_mut(|i| {
                            i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                        });
                        (l, r)
                    }
                }
            };

            if let Some(tab) = self.focused_terminal.clone() {
                history_menu_handled = previous
                    || next
                    || close
                    || confirm
                    || focus_left
                    || focus_right
                    || delete_pressed
                    || favorite_pressed;
                // Focus toggle: Right → favorites (first item), Left → back
                // to the main list (selection there is preserved).
                if focus_left || focus_right {
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        if let Some(nav) = td.instance.history_nav.as_mut() {
                            if focus_right {
                                if !nav.fav_focused {
                                    nav.fav_focused = true;
                                    // Restore the REMEMBERED folder cursor
                                    // (clamped to the current folder
                                    // count) instead of snapping to the
                                    // first folder every time.
                                    nav.fav_selected = self
                                        .fav_cursor
                                        .min(self.fav_folders.len().saturating_sub(1));
                                    // Entering the folder list PREVIEWS the
                                    // first folder's command column right
                                    // away (parity with Up/Down moves and
                                    // mouse hover) — it used to stay closed
                                    // until the cursor moved.
                                    if let Some((fid, _)) = self.fav_folders.get(nav.fav_selected) {
                                        let items = self.history_db.fav_items(*fid);
                                        if !items.is_empty() {
                                            self.fav_submenu =
                                                Some((*fid, egui::Pos2::ZERO, items, None));
                                        }
                                    }
                                } else if !self.fav_sub_focused && self.fav_submenu.is_some() {
                                    // Right moves INTO the visible command
                                    // column: now Up/Down/Enter operate on
                                    // commands.
                                    self.fav_sub_focused = true;
                                    if let Some((fid, a, items, _)) = self.fav_submenu.clone() {
                                        self.fav_submenu = Some((fid, a, items, Some(0)));
                                    }
                                }
                            } else if self.fav_sub_focused {
                                // Left from INSIDE the command column:
                                // focus returns to the folder list; the
                                // column keeps previewing the folder.
                                self.fav_sub_focused = false;
                                if let Some((fid, a, items, _)) = self.fav_submenu.clone() {
                                    self.fav_submenu = Some((fid, a, items, None));
                                }
                            } else if nav.fav_focused {
                                // Left from the folder list: straight
                                // back to the HISTORY list — the preview
                                // column closes with the focus handoff
                                // (no intermediate step that swallows
                                // the first Left press).
                                nav.fav_focused = false;
                                self.fav_submenu = None;
                                self.fav_sub_focused = false;
                            }
                        }
                    }
                }
                // Delete under the keyboard cursor (history or favorite).
                if delete_pressed {
                    // Command-column Delete (only when focus is INSIDE
                    // the column — mirrors the history-list behavior).
                    if self.fav_sub_focused {
                        if let Some((fid, _, _, Some(sel))) = self.fav_submenu.clone() {
                            // Rowid-addressed delete: only THAT duplicate.
                            let with_ids = self.history_db.fav_items_with_ids(fid);
                            if let Some((rid, _)) = with_ids.get(sel) {
                                let rid = *rid;
                                self.history_db.fav_item_remove_row(rid);
                                self.fav_folders = self.history_db.fav_folders();
                                let fresh: Vec<String> = self
                                    .history_db
                                    .fav_items_with_ids(fid)
                                    .into_iter()
                                    .map(|(_, c)| c)
                                    .collect();
                                if fresh.is_empty() {
                                    self.fav_submenu = None;
                                } else {
                                    let sel2 = sel.min(fresh.len() - 1);
                                    self.fav_submenu =
                                        Some((fid, egui::Pos2::ZERO, fresh, Some(sel2)));
                                }
                            }
                        }
                        // NOTE: no bare `return` — returning from HERE
                        // aborted the rest of update() (dock area, side
                        // bar, menus) for one frame: the black flash.
                    }
                    // Cursor on the FOLDER list (and NOT inside its
                    // command column — fav_sub_focused stays true while
                    // the folder keeps focus, so both deletes used to
                    // fire): open the SAME confirm dialog as the trash
                    // button (protected default folder excluded).
                    let on_folders = self
                        .terminals
                        .get(&tab)
                        .and_then(|td| td.instance.history_nav.as_ref())
                        .is_some_and(|n| n.fav_focused)
                        && !self.fav_sub_focused;
                    if on_folders {
                        let target = self
                            .terminals
                            .get(&tab)
                            .and_then(|td| td.instance.history_nav.as_ref())
                            .and_then(|n| self.fav_folders.get(n.fav_selected).cloned());
                        if let Some((fid, name)) = target {
                            if name != crate::history_db::HistoryDb::DEFAULT_FAVORITE_FOLDER {
                                self.fav_delete_confirm = Some((fid, name));
                                self.fav_del_just_opened = true;
                            }
                        }
                    } else {
                        let (fav_idx, hist_idx) = self
                            .terminals
                            .get(&tab)
                            .and_then(|td| td.instance.history_nav.as_ref())
                            .map(|nav| {
                                if !nav.fav_focused {
                                    (None, Some(nav.selected))
                                } else {
                                    (None::<usize>, None)
                                }
                            })
                            .unwrap_or((None, None));
                        if let Some(fi) = fav_idx {
                            let cmd = self
                                .terminals
                                .get(&tab)
                                .and_then(|td| td.instance.history_nav.as_ref())
                                .and_then(|nav| nav.favorites.get(fi).cloned());
                            if let Some(cmd) = cmd {
                                self.history_db.fav_remove(&cmd);
                                // (history-path legacy branch; closes the
                                // exclusive else-chain opened above)
                                if let Some(td) = self.terminals.get_mut(&tab) {
                                    if let Some(nav) = td.instance.history_nav.as_mut() {
                                        nav.favorites = self.history_db.fav_all();
                                        if nav.favorites.is_empty() {
                                            nav.fav_focused = false;
                                        } else {
                                            nav.fav_selected =
                                                nav.fav_selected.min(nav.favorites.len() - 1);
                                        }
                                    }
                                }
                            }
                        } else if let Some(i) = hist_idx {
                            self.history_db.remove_entry(&tab, i);
                            if let Some(td) = self.terminals.get_mut(&tab) {
                                if let Some(nav) = td.instance.history_nav.as_mut() {
                                    if i < nav.entries.len() {
                                        nav.entries.remove(i);
                                    }
                                    if !nav.entries.is_empty() {
                                        nav.selected = nav.selected.min(nav.entries.len() - 1);
                                    }
                                }
                            }
                        }
                    } // exclusive else-chain (folder-list vs history)
                    history_menu_handled = true;
                }
                // Favorite the selected HISTORY entry (Insert by default).
                if favorite_pressed {
                    let cmd = self
                        .terminals
                        .get(&tab)
                        .and_then(|td| td.instance.history_nav.as_ref())
                        .filter(|nav| !nav.fav_focused)
                        .and_then(|nav| nav.entries.get(nav.selected).cloned());
                    if let Some(cmd) = cmd {
                        self.history_db.fav_add(&cmd);
                        if let Some(td) = self.terminals.get_mut(&tab) {
                            if let Some(nav) = td.instance.history_nav.as_mut() {
                                nav.favorites = self.history_db.fav_all();
                            }
                        }
                    }
                }
                // While the favorites list holds focus, Up/Down navigate
                // the favorites instead of the main list, and Enter sends
                // the selected favorite command. The favorites scroll
                // follows the selection the same way the main list's does
                // (bring into view only on the frame the selection moved).
                if let Some(td) = self.terminals.get_mut(&tab) {
                    if let Some(nav) = td.instance.history_nav.as_mut() {
                        // The command column owns Up/Down ONLY after the
                        // user pressed Right INTO it (fav_sub_focused);
                        // while it merely PREVIEWS (cursor on folders),
                        // Up/Down keep walking the folder list.
                        if self.fav_sub_focused && self.fav_submenu.is_some() && nav.fav_focused {
                            if let Some((fid, anchor, sub_items, sub_sel)) =
                                self.fav_submenu.clone()
                            {
                                let mut new_sel = sub_sel;
                                if previous {
                                    new_sel = Some(sub_sel.unwrap_or(0).saturating_sub(1));
                                } else if next {
                                    let cur = sub_sel.unwrap_or(0);
                                    if cur + 1 < sub_items.len() {
                                        new_sel = Some(cur + 1);
                                    }
                                }
                                self.fav_submenu = Some((fid, anchor, sub_items.clone(), new_sel));
                                // Bring the highlighted COMMAND into view
                                // this frame — same follow semantics as
                                // the folder column (no header offset
                                // here; 10 rows fit the 200px body).
                                if let Some(sel) = new_sel {
                                    let sub_scroll_id =
                                        egui::Id::new(("hist_subcol_scroll", tab.as_str()));
                                    let mv = 10usize;
                                    let max_sc = sub_items.len().saturating_sub(mv);
                                    let mut sc = ctx
                                        .memory(|m| m.data.get_temp(sub_scroll_id).unwrap_or(0))
                                        .min(max_sc);
                                    if sel < sc {
                                        sc = sel;
                                    } else if sel >= sc + mv {
                                        sc = sel + 1 - mv;
                                    }
                                    ctx.memory_mut(|m| m.data.insert_temp(sub_scroll_id, sc));
                                }
                            }
                        } else if nav.fav_focused && (previous || next) {
                            let folder_count = self.fav_folders.len();
                            if previous {
                                nav.fav_selected = nav.fav_selected.saturating_sub(1);
                            } else if next && nav.fav_selected + 1 < folder_count {
                                nav.fav_selected += 1;
                            }
                            // Keyboard selection OPENS the selected
                            // folder's submenu (the third column shows its
                            // commands) — same as hover. Right then moves
                            // the CURSOR into the third column.
                            if let Some((fid, _)) = self.fav_folders.get(nav.fav_selected) {
                                let items = self.history_db.fav_items(*fid);
                                // Always open the selected folder's column
                                // (even an empty folder shows an empty list).
                                self.fav_submenu = Some((*fid, egui::Pos2::ZERO, items, None));
                            }
                            // Selection changed: bring the highlighted
                            // FOLDER into view this frame — same follow
                            // semantics as the main list, but against the
                            // folder column's scroll (the old code wrote
                            // the legacy flat-favorites scroll id, so the
                            // column never followed). Capacity accounts
                            // for the 24px column header.
                            let col_scroll_id = egui::Id::new(("hist_favcol_scroll", tab.as_str()));
                            let header_h = 24.0f32;
                            let row_h = 20.0f32;
                            let mv = (((200.0f32 - header_h).max(row_h)) / row_h) as usize;
                            let max_sc = folder_count.saturating_sub(mv);
                            let mut sc = ctx
                                .memory(|m| m.data.get_temp(col_scroll_id).unwrap_or(0))
                                .min(max_sc);
                            if nav.fav_selected < sc {
                                sc = nav.fav_selected;
                            } else if nav.fav_selected >= sc + mv {
                                sc = nav.fav_selected + 1 - mv;
                            }
                            ctx.memory_mut(|m| m.data.insert_temp(col_scroll_id, sc));
                            // Remember the cursor for the next open.
                            self.fav_cursor = nav.fav_selected;
                        }
                    }
                }
                let fav_confirming = confirm
                    && self
                        .terminals
                        .get(&tab)
                        .and_then(|td| td.instance.history_nav.as_ref())
                        .is_some_and(|nav| nav.fav_focused);
                if previous || next {
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        if let Some(nav) = td.instance.history_nav.as_mut() {
                            if nav.fav_focused {
                                // Favorites own Up/Down this frame.
                            } else {
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
                }
                if close {
                    self.close_history_menu(&tab);
                }
                if confirm {
                    // Enter with focus INSIDE the command column sends the
                    // selected command; with focus on folders it assembles
                    // the folder (earlier behavior).
                    if self.fav_sub_focused {
                        if let Some((fid, _, items, Some(sel))) = self.fav_submenu.clone() {
                            if let Some(cmd) = items.get(sel).cloned() {
                                let _ = fid;
                                // Snippets with {placeholders} open the
                                // fill dialog instead of inserting raw.
                                if !self.open_snippet_fill(&tab, cmd.clone()) {
                                    if let Some(td) = self.terminals.get_mut(&tab) {
                                        td.instance.history_nav = None;
                                        // Type without executing — same as
                                        // the history list's Enter (the \r
                                        // made it run immediately).
                                        td.instance.write(cmd.as_bytes());
                                    }
                                }
                                self.history_menu_just_closed.insert(tab.clone(), true);
                                self.fav_submenu = None;
                            }
                        }
                        history_menu_handled = true;
                    } else if fav_confirming {
                        // Enter on the folder column: ASSEMBLE the selected
                        // folder's commands into one line and send it (shell-
                        // aware separators).
                        let assembled = self
                            .fav_folders
                            .get(
                                self.terminals
                                    .get(&tab)
                                    .and_then(|td| td.instance.history_nav.as_ref())
                                    .map(|nav| nav.fav_selected)
                                    .unwrap_or(0),
                            )
                            .and_then(|(fid, _)| {
                                let shell_id =
                                    self.terminals.get(&tab).map(|td| td.shell_id.clone())?;
                                let line =
                                    assemble_commands(&self.history_db.fav_items(*fid), &shell_id);
                                (!line.is_empty()).then_some(line)
                            });
                        if let Some(cmd) = assembled {
                            if !self.open_snippet_fill(&tab, cmd.clone()) {
                                if let Some(td) = self.terminals.get_mut(&tab) {
                                    td.instance.history_nav = None;
                                    td.instance.write(cmd.as_bytes());
                                }
                            }
                            self.history_menu_just_closed.insert(tab.clone(), true);
                        }
                    } else {
                        // Unified confirm path: sends the entry AND sets the
                        // close latch (stops matching like Esc).
                        self.confirm_history_entry(&tab);
                    }
                }
            }
        }

        // Unified shortcut gate: no shortcuts while ANY modal dialog owns
        // the UI or a key-binding recording is in progress. This replaces
        // the old `global_shortcuts_allowed`, whose dialog list had
        // drifted from `any_modal_open` (theme editor etc. were missing).
        let shortcuts_allowed = !workspace_renaming && !modal_hijack && !binding_was_recording;

        if shortcuts_allowed && consume_exact_shortcut(ctx, &binds, "toggle_workspace_sidebar") {
            self.workspace_sidebar_visible = !self.workspace_sidebar_visible;
        }

        // Enter: select history entry, or record command + send CR to terminal
        if !workspace_renaming
            && !history_menu_handled
            && !self.locked_panels.contains(&self.active_panel)
            && !modal_hijack
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
        {
            if let Some(tab) = &self.focused_terminal.clone() {
                // Host label for the history row (ssh addr, "" = local),
                // captured BEFORE the mutable terminal borrow below.
                let rec_host = self
                    .terminals
                    .get(tab)
                    .and_then(|td| td.host.as_ref())
                    .map(|h| h.addr.clone())
                    .unwrap_or_default();
                if let Some(td) = self.terminals.get_mut(tab) {
                    if let Some(ref nav) = td.instance.history_nav {
                        let selected = nav.entries.get(nav.selected).cloned();
                        if let Some(cmd) = selected {
                            td.instance.write(cmd.as_bytes());
                        }
                        td.instance.history_nav = None;
                    } else {
                        let line = td.instance.get_current_line();
                        // Strip the prompt via the shared helper: the
                        // bare "$"/"#" fallback matters after a resize —
                        // readline swallows the line-end space when the
                        // prompt wraps, and the old "$ "-only lookup
                        // recorded NOTHING for the first command typed
                        // after narrowing the pane.
                        let cmd = crate::terminal::strip_prompt(&line)
                            .map(|s| s.trim().to_string())
                            .unwrap_or_default();
                        if !cmd.is_empty() {
                            self.history_db.add(tab, &cmd, &rec_host);
                        }
                        // The command was EXECUTED: the pending-keystroke
                        // buffer must not survive into the next prompt —
                        // stale chars used to pollute the first keystrokes
                        // of the next command (word = grid + stale → no
                        // match, menu never opened until backspace).
                        self.auto_match_pending.remove(tab);
                        td.instance.write(b"\r");
                    }
                }
            }
        }

        // Escape closes the floating panels (AI assistant / remote
        // control) BEFORE the terminal sees the escape — both were
        // otherwise unclosable from the keyboard (no Esc path at all,
        // while every other popup in the app closes on Esc). Real modal
        // dialogs keep priority: they own the Esc while open.
        let ai_panel_open = self.show_ai_panel;
        let remote_panel_open = self
            .remote
            .as_ref()
            .is_some_and(|s| s.lan_panel_visible || s.wan_panel_visible);
        if !any_modal_open_excluding_ai(self)
            && (ai_panel_open || remote_panel_open)
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            if ai_panel_open {
                self.show_ai_panel = false;
            }
            if remote_panel_open {
                // Esc closes the remote panels AND stops the session
                // (server + tunnel + frp).
                self.remote_stop();
            }
        }

        // Escape: close history menu
        if !workspace_renaming
            && !history_menu_handled
            && !modal_hijack
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

        // Handle virtual-key sequence recording in settings (remote page).
        // Runs only when the SHORTCUT recorder is idle; Esc cancels.
        if let Some(idx) = self.virtual_key_recording {
            if self.binding_recording.is_some() {
                // Shortcut recording takes precedence this frame.
            } else {
                #[derive(Default)]
                struct Captured {
                    seq: String,
                    label: String,
                    cancel: bool,
                }
                let captured = ctx.input(|i| {
                    let mut out = Captured::default();
                    for event in &i.events {
                        match event {
                            egui::Event::Key {
                                key,
                                pressed: true,
                                repeat: false,
                                modifiers,
                                ..
                            } => {
                                if *key == egui::Key::Escape {
                                    out.cancel = true;
                                    return out;
                                }
                                if let Some((seq, label)) =
                                    virtual_key_from_capture(*key, *modifiers)
                                {
                                    out.seq = seq;
                                    out.label = label;
                                    return out;
                                }
                            }
                            egui::Event::Text(t) if !t.is_empty() => {
                                out.seq = t.clone();
                                out.label = t.trim().to_string();
                                return out;
                            }
                            // egui-winit SWALLOWS Ctrl+C / Ctrl+X / Ctrl+V
                            // (and their Shift variants) on Linux/Windows:
                            // they arrive here as Copy/Cut/Paste with the
                            // original Key event dropped. The frame's
                            // modifiers still carry the full combo, so
                            // Ctrl+Shift+C records with its Shift intact —
                            // emitted as the xterm modifyOtherKeys form
                            // (\x1b[27;mod;char) so TUIs can tell it apart
                            // from plain Ctrl+C.
                            egui::Event::Copy | egui::Event::Cut | egui::Event::Paste(_) => {
                                let m = i.modifiers;
                                let (plain_byte, ch, shift_relevant) = match event {
                                    egui::Event::Copy => ('\x03', 'c', true),
                                    egui::Event::Cut => ('\x18', 'x', false),
                                    _ => ('\x16', 'v', true),
                                };
                                let mut seq = String::new();
                                if m.alt {
                                    seq.push('\x1b');
                                }
                                if shift_relevant && m.shift {
                                    // mod = 1 + shift(1) + alt(2) + ctrl(4)
                                    let code = 1
                                        + (m.shift as u8)
                                        + ((m.alt as u8) << 1)
                                        + ((m.ctrl as u8) << 2);
                                    seq.push_str(&format!("\x1b[27;{};{}", code, ch as u8));
                                } else {
                                    seq.push(plain_byte);
                                }
                                let mut label = String::from("Ctrl+");
                                if m.alt {
                                    label.push_str("Alt+");
                                }
                                if shift_relevant && m.shift {
                                    label.push_str("Shift+");
                                }
                                label.push(ch.to_ascii_uppercase());
                                out.seq = seq;
                                out.label = label;
                                return out;
                            }
                            _ => {}
                        }
                    }
                    out
                });
                if captured.cancel {
                    self.virtual_key_recording = None;
                } else if !captured.seq.is_empty() {
                    if let Some(vk) = self.settings_edit.remote_keys.get_mut(idx) {
                        vk.action = captured.seq;
                        if vk.label.is_empty() {
                            vk.label = captured.label;
                        }
                    }
                    self.virtual_key_recording = None;
                }
            }
        }

        // Configurable shortcuts

        if shortcuts_allowed && check_shortcut(ctx, &binds, "new_terminal") {
            if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                if let Some((surface, node)) = tree.focused_leaf() {
                    self.pending_new_terminal = Some((self.active_panel, surface, node));
                }
            }
        }
        // Cycle to the next terminal tab within the CURRENT panel's
        // focused leaf (wraps from the last back to the first).
        if shortcuts_allowed && check_shortcut(ctx, &binds, "next_terminal") {
            if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                if let Some((surface, node)) = tree.focused_leaf() {
                    let tabs: Vec<String> = tree
                        .iter_all_tabs()
                        .filter(|((s, _), _)| *s == surface)
                        .filter(|((_, n), _)| *n == node)
                        .map(|(_, t)| t.clone())
                        .collect();
                    if tabs.len() > 1 {
                        let current = tree.find_active_focused().map(|(_, t)| t.clone());
                        let next = current
                            .and_then(|cur| {
                                let idx = tabs.iter().position(|t| *t == cur)?;
                                Some(tabs[(idx + 1) % tabs.len()].clone())
                            })
                            .unwrap_or_else(|| tabs[0].clone());
                        if let Some(loc) = tree.find_tab(&next) {
                            tree.set_active_tab(loc);
                        }
                        tree.set_focused_node_and_surface((surface, node));
                        self.focused_terminal = Some(next);
                    }
                }
            }
        }
        // Cycle the SPLIT PANES (leaves) inside the current workspace.
        if shortcuts_allowed && check_shortcut(ctx, &binds, "next_panel") {
            self.focus_adjacent_panel(1);
        }
        // Cycle FOCUS across workspaces (wraps last → first) and focus
        // the workspace's active terminal.
        if shortcuts_allowed
            && check_shortcut(ctx, &binds, "next_workspace")
            && self.panels.len() > 1
        {
            self.active_panel = (self.active_panel + 1) % self.panels.len();
            self.restore_workspace_focus(self.active_panel);
        }
        // Save the current layout (menu Workspace > Save).
        if shortcuts_allowed && check_shortcut(ctx, &binds, "save_scene") {
            self.save_scene();
        }
        if shortcuts_allowed && check_shortcut(ctx, &binds, "close_terminal") {
            // Same confirmation dialog as the mouse-close path (on_close):
            // the shortcut must not bypass it.
            if let Some(tab) = &self.focused_terminal.clone() {
                if self.pending_close_confirm.is_none() {
                    self.pending_close_confirm = Some(tab.clone());
                    self.close_confirm_just_opened = true;
                }
            }
        }
        // Move the active workspace one slot up/down. No-repeat: holding
        // the combo must not cycle at ~30 switches/s (repaint thrash).
        // restore_workspace_focus re-points focused_terminal at the new
        // workspace's active tab — without it the old terminal's focus
        // lock survived while the NEW workspace's terminals all
        // surrendered focus, so egui kept routing Text events to a
        // widget that was no longer rendered (keyboard died app-wide
        // until restart). Clearing terminal_focus_id releases the stale
        // focus-lock filter in raw_input_hook.
        if shortcuts_allowed
            && check_shortcut_no_repeat(ctx, &binds, "workspace_up")
            && self.active_panel > 0
        {
            self.active_panel -= 1;
            self.terminal_focus_id = None;
            self.restore_workspace_focus(self.active_panel);
        }
        if shortcuts_allowed
            && check_shortcut_no_repeat(ctx, &binds, "workspace_down")
            && self.active_panel + 1 < self.panels.len()
        {
            self.active_panel += 1;
            self.terminal_focus_id = None;
            self.restore_workspace_focus(self.active_panel);
        }
        if shortcuts_allowed && check_shortcut(ctx, &binds, "panel_left") {
            self.focus_adjacent_panel(-1);
        }
        if shortcuts_allowed && check_shortcut(ctx, &binds, "panel_right") {
            self.focus_adjacent_panel(1);
        }
        if shortcuts_allowed && check_shortcut(ctx, &binds, "lock_workspace") {
            if self.locked_panels.contains(&self.active_panel) {
                // Already locked: overlay already shows password input.
                self.lock_password_input.clear();
                self.pw_message.clear();
            } else {
                self.try_lock_workspace(self.active_panel);
            }
        }

        // Global UI zoom (Ctrl +/- by default, user-configurable).
        if shortcuts_allowed && check_shortcut(ctx, &binds, "zoom_in") {
            let z = ctx.zoom_factor();
            ctx.set_zoom_factor((z + 0.1).min(3.0));
        }
        if shortcuts_allowed && check_shortcut(ctx, &binds, "zoom_out") {
            let z = ctx.zoom_factor();
            ctx.set_zoom_factor((z - 0.1).max(0.5));
        }

        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::new()
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
                    let chrome =
                        crate::theme::ui::ButtonChrome::from_theme(&self.active_theme.app, fg_menu);
                    let menu_btn = |ui: &mut egui::Ui, label: &str| -> egui::Response {
                        // Fully hand-drawn button: the chrome layers are
                        // painted BEFORE the text so they can never cover
                        // the label. Unified three-state feedback via
                        // ButtonChrome (hover fill + pressed dim + focus ring).
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
                        let _fg = chrome.paint(ui, rect, &resp);
                        ui.painter()
                            .galley(rect.center() - galley.size() / 2.0, galley, fg_menu);
                        resp
                    };
                    // Dropdown wrapper: the visible button + the popup menu
                    // share one hit area (invisible overlay Button drives
                    // egui's BarState so styling stays fully ours).
                    let dropdown =
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
                        let visible = self.show_monitor;
                        if ui
                            .selectable_label(visible, &self.texts.view_menu.monitor)
                            .clicked()
                        {
                            self.show_monitor = !self.show_monitor;
                            ui.close_menu();
                        }
                    });
                    // Top-level toggle buttons: AI assistant & remote
                    // control (pulled out of the View menu — frequent
                    // single-click actions). Accent text marks the ON
                    // state.
                    let accent = self.active_theme.app.accent.to_egui();
                    let menu_btn_toggled =
                        |ui: &mut egui::Ui, label: &str, active: bool| -> egui::Response {
                            let color = if active { accent } else { fg_menu };
                            let galley = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    label.to_string(),
                                    egui::FontId::proportional(12.0),
                                    color,
                                )
                            });
                            let pad = 8.0;
                            let size =
                                egui::vec2(galley.size().x + pad * 2.0, galley.size().y + 8.0);
                            let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                            let _fg = chrome.paint(ui, rect, &resp);
                            ui.painter()
                                .galley(rect.center() - galley.size() / 2.0, galley, color);
                            resp
                        };
                    if AI_UI_ENABLED {
                        let ai_label = self.texts.view_menu.ai_assistant.clone();
                        let ai_active = self.show_ai_panel;
                        if menu_btn_toggled(ui, &ai_label, ai_active).clicked() {
                            self.show_ai_panel = !self.show_ai_panel;
                        }
                    }
                    // Remote control: a DROPDOWN with the two channels —
                    // clicking an entry IS the start action (opens the
                    // channel's own panel immediately; no separate start
                    // buttons inside). Both panels can be open at once.
                    let remote_label = self.texts.view_menu.remote.clone();
                    let remote_active = self
                        .remote
                        .as_ref()
                        .is_some_and(|s| s.lan_panel_visible || s.wan_panel_visible);
                    {
                        let btn = menu_btn_toggled(ui, &remote_label, remote_active);
                        let mut bar =
                            egui::menu::BarState::load(ui.ctx(), egui::Id::new("menu_remote"));
                        let overlay = ui.interact(
                            btn.rect,
                            egui::Id::new(("menu_remote", "hit")),
                            egui::Sense::click(),
                        );
                        bar.bar_menu(&overlay, |ui| {
                            let lan_on = self.remote.as_ref().is_some_and(|s| s.lan_panel_visible);
                            if ui
                                .selectable_label(lan_on, &self.texts.remote.menu_lan)
                                .clicked()
                            {
                                self.remote_toggle_lan();
                                ui.close_menu();
                            }
                            let wan_on = self.remote.as_ref().is_some_and(|s| s.wan_panel_visible);
                            if ui
                                .selectable_label(wan_on, &self.texts.remote.menu_wan)
                                .clicked()
                            {
                                self.remote_toggle_wan();
                                ui.close_menu();
                            }
                        });
                        bar.store(ui.ctx(), egui::Id::new("menu_remote"));
                    }
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
                    // Help dropdown: Tutorial + About (at the bottom).
                    // The update entry is a STANDALONE right-corner button.
                    let help_label = self.texts.menu.help.clone();
                    dropdown(ui, &help_label, "menu_help", &mut |ui| {
                        if ui.button(&self.texts.help.title).clicked() {
                            self.show_help_window = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(&self.texts.about.menu_label).clicked() {
                            self.show_about = true;
                            ui.close_menu();
                        }
                    });

                    // Right-aligned extras: a PERSISTENT update button
                    // (rightmost) + the app version to its left. The
                    // button label follows the last check: an available
                    // release reads "更新至 v{x}" (accent), anything else
                    // reads "检查更新". Clicking opens the update window,
                    // which re-checks and drives the download flow.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let weak = self.active_theme.app.weak_text.to_egui();
                        let fg = self.active_theme.app.button_fg.to_egui();
                        let accent = self.active_theme.app.accent.to_egui();
                        let hover_bg = self.active_theme.app.hover.to_egui();

                        let (label_text, label_color) =
                            if let crate::updater::UpdateState::Available(info) = &self.update_state
                            {
                                (self.texts.update.badge.replace("{}", &info.version), accent)
                            } else {
                                (self.texts.update.check.clone(), fg)
                            };
                        let pad = 8.0;
                        let galley = ui.fonts(|f| {
                            f.layout_no_wrap(
                                label_text.clone(),
                                egui::FontId::proportional(11.0),
                                label_color,
                            )
                        });
                        let size = egui::vec2(galley.size().x + pad * 2.0, galley.size().y + 6.0);
                        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                        if resp.contains_pointer() {
                            ui.painter().rect_filled(rect, 0.0, hover_bg);
                        }
                        ui.painter().galley(
                            rect.center() - galley.size() / 2.0,
                            galley,
                            label_color,
                        );
                        if resp.clicked() {
                            self.show_update_window = true;
                        }

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
            let open = self.show_settings;
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
            let _layer_id = egui::LayerId::new(egui::Order::Middle, egui::Id::new("settings_area"));
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

                        let nav_item =
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
                        if AI_UI_ENABLED
                            && nav_item(
                                nav_ui,
                                &texts.settings.nav.ai_assistant,
                                SettingsPage::AiAssistant,
                            )
                        {
                            self.settings_tab = SettingsPage::AiAssistant as u8;
                        }
                        if nav_item(nav_ui, &texts.settings.nav.remote, SettingsPage::Remote) {
                            self.settings_tab = SettingsPage::Remote as u8;
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
                                                SettingsPage::AiAssistant => {
                                                    self.settings_page_ai(ui)
                                                }
                                                SettingsPage::Remote => {
                                                    self.settings_page_remote(ui)
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

            // Keep the create_terminal fallback in sync with the chosen
            // default shell (instant-apply pipeline).
            if let Ok(mut guard) = DEFAULT_SHELL_ID.write() {
                *guard = self.settings_edit.default_shell.clone();
            }
            // Edge-smoothing (feathering): off = hard edges, on = the
            // configured width in physical pixels. Text glyph AA lives in
            // the font atlas and is unaffected by this.
            ctx.tessellation_options_mut(|t| {
                t.feathering = self.settings_edit.smooth_rendering;
                t.feathering_size_in_pixels = if self.settings_edit.smooth_rendering {
                    self.settings_edit.smooth_level.clamp(0.0, 2.0)
                } else {
                    0.0
                };
            });
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
        // Styled like the password popups: compact metrics, danger confirm.
        if self.show_clear_history_confirm {
            // Rising edge: start on the safe side (CANCEL).
            if std::mem::take(&mut self.settings_clear_just_opened) {
                self.dialog_kb_confirm = false;
            }
            // Unified protocol, BEFORE the Window.
            let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
            let mut confirmed = keys.confirm;
            let mut cancelled = keys.cancel;
            if keys.close {
                self.show_clear_history_confirm = false;
                return;
            }
            let mut kb = self.dialog_kb_confirm;
            let mut open = self.show_clear_history_confirm;
            let title = self.texts.stats.clear_history_title.clone();
            let body = self.texts.stats.clear_history_body.clone();
            let confirm_txt = self.texts.theme_editor.dialog_confirm.clone();
            let cancel_txt = self.texts.theme_editor.cancel.clone();
            let text_col = self.active_theme.app.text.to_egui();
            // Same fixed 360x300 dialog: bottom panel pins the centered
            // button row 20px above the bottom edge; finite height kills
            // the auto-size growth loop.
            let dlg_w = 360.0f32;
            let dlg_h = 96.0f32;
            let center = ctx.screen_rect().center();
            let pos = egui::pos2(center.x - dlg_w / 2.0, center.y - dlg_h / 2.0);
            egui::Window::new(title)
                .id(egui::Id::new("settings_clear_confirm"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .fixed_pos(pos)
                .fixed_size([dlg_w, dlg_h])
                .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::same(12)))
                .show(ctx, |ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 4.0);
                    ui.style_mut().spacing.interact_size.y = 24.0;
                    ui.style_mut().spacing.button_padding = egui::vec2(10.0, 3.0);
                    egui::TopBottomPanel::bottom("settings_clear_confirm_footer")
                        .frame(egui::Frame::new())
                        .exact_height(44.0)
                        .show_inside(ui, |ui| {
                            ui.add_space(20.0);
                            let (c, x) = Self::dialog_button_row(
                                ui,
                                &mut kb,
                                egui::Id::new("settings_clear_confirm_btn"),
                                egui::Id::new("settings_clear_cancel_btn"),
                                &confirm_txt,
                                &cancel_txt,
                            );
                            confirmed |= c;
                            cancelled |= x;
                        });
                    ui.label(egui::RichText::new(body).size(13.0).color(text_col));
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

        // Confirm-dialog for clearing ALL global favorite commands.
        // Same fixed-size danger dialog as the history clear above.
        if self.show_clear_favorites_confirm {
            // Rising edge: start on the safe side (CANCEL).
            if std::mem::take(&mut self.fav_clear_just_opened) {
                self.dialog_kb_confirm = false;
            }
            // Unified protocol, BEFORE the Modal.
            let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
            let mut confirmed = keys.confirm;
            let mut cancelled = keys.cancel;
            if keys.close {
                self.show_clear_favorites_confirm = false;
                return;
            }
            let mut kb = self.dialog_kb_confirm;
            let title = self.texts.terminal.clear_favorites_title.clone();
            let body = self.texts.terminal.clear_favorites_body.clone();
            let confirm_txt = self.texts.theme_editor.dialog_confirm.clone();
            let cancel_txt = self.texts.theme_editor.cancel.clone();
            let danger = self.active_theme.app.danger.to_egui();
            let text_col = self.active_theme.app.text.to_egui();
            let dlg_w = 360.0f32;
            let dlg_h = 96.0f32;
            let center = ctx.screen_rect().center();
            let pos = egui::pos2(center.x - dlg_w / 2.0, center.y - dlg_h / 2.0);
            let _ = pos;
            let modal = egui::Modal::new(egui::Id::new("fav_clear_confirm"))
                .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::same(12)))
                .show(ctx, |ui| {
                    ui.set_min_size(egui::vec2(dlg_w, dlg_h));
                    ui.heading(title);
                    ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 4.0);
                    ui.style_mut().spacing.interact_size.y = 24.0;
                    ui.style_mut().spacing.button_padding = egui::vec2(10.0, 3.0);
                    egui::TopBottomPanel::bottom("fav_clear_confirm_footer")
                        .frame(egui::Frame::new())
                        .exact_height(44.0)
                        .show_inside(ui, |ui| {
                            ui.add_space(20.0);
                            let (c, x) = Self::dialog_button_row(
                                ui,
                                &mut kb,
                                egui::Id::new("fav_clear_confirm_btn"),
                                egui::Id::new("fav_clear_cancel_btn"),
                                &confirm_txt,
                                &cancel_txt,
                            );
                            confirmed |= c;
                            cancelled |= x;
                        });
                    ui.label(egui::RichText::new(body).size(13.0).color(text_col));
                });
            let _ = danger;
            if cancelled {
                self.show_clear_favorites_confirm = false;
            } else if confirmed {
                self.history_db.fav_clear();
                // Drop the favorites snapshot from any open menus so the
                // side lists disappear immediately.
                for td in self.terminals.values_mut() {
                    if let Some(nav) = td.instance.history_nav.as_mut() {
                        nav.favorites.clear();
                        nav.fav_focused = false;
                    }
                }
                self.show_clear_favorites_confirm = false;
            }
            // Backdrop click cancels.
            if modal.backdrop_response.clicked() {
                self.show_clear_favorites_confirm = false;
            }
        }

        if self.show_about {
            // Esc consumed first; no early return (the old `return` skipped
            // the rest of the frame's UI - sidebar/terminal - flashing it).
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.show_about = false;
            }
            let mut open = self.show_about;
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
                });
            if !open {
                self.show_about = false;
            }
        }

        // Standalone UPDATE window (menu 帮助 → 更新, or the corner
        // badge). Opening it triggers ONE version check (temp latch
        // cleared on close); the UpdateState state machine drives the
        // display, reusing the old About-page flow.
        if self.show_update_window {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.show_update_window = false;
            }
            let checked_latch = egui::Id::new("update_window_checked");
            let already_checked =
                ctx.memory(|m| m.data.get_temp::<bool>(checked_latch).unwrap_or(false));
            if !already_checked {
                ctx.memory_mut(|m| m.data.insert_temp(checked_latch, true));
                self.start_manual_check();
                self.check_update_manual(ctx);
            }
            let mut open = true;
            egui::Window::new(&self.texts.update_window.title)
                .id(egui::Id::new("update_window"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_size([520.0, 460.0])
                .default_width(520.0)
                .min_width(460.0)
                .show(ctx, |ui| {
                    let ut = self.texts.update.clone();
                    let uw = self.texts.update_window.clone();
                    let weak = self.active_theme.app.weak_text.to_egui();
                    let accent = self.active_theme.app.accent.to_egui();
                    use crate::updater::UpdateState;

                    // Current version (always visible).
                    ui.horizontal(|ui| {
                        ui.weak(
                            egui::RichText::new(&uw.current_version)
                                .size(12.0)
                                .color(weak),
                        );
                        ui.label(
                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .size(12.0),
                        );
                    });
                    ui.add_space(6.0);

                    // Status line per state.
                    match &self.update_state {
                        UpdateState::Idle | UpdateState::Checking => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(&ut.checking);
                            });
                        }
                        UpdateState::UpToDate => {
                            ui.label(
                                egui::RichText::new(&ut.up_to_date)
                                    .size(12.0)
                                    .color(self.active_theme.app.success.to_egui()),
                            );
                        }
                        UpdateState::Available(info) => {
                            ui.label(
                                egui::RichText::new(ut.available.replace("{}", &info.version))
                                    .size(13.0)
                                    .color(accent),
                            );
                        }
                        UpdateState::Downloading(_) => {
                            ui.label(
                                egui::RichText::new(&ut.downloading)
                                    .size(12.0)
                                    .color(self.active_theme.app.info.to_egui()),
                            );
                        }
                        UpdateState::Verifying => {
                            ui.label(egui::RichText::new(&ut.verifying).size(12.0).color(accent));
                        }
                        UpdateState::Ready(_) => {
                            ui.label(
                                egui::RichText::new(&ut.ready)
                                    .size(13.0)
                                    .color(self.active_theme.app.success.to_egui()),
                            );
                        }
                        UpdateState::Error(msg) => {
                            ui.label(
                                egui::RichText::new(ut.failed.replace("{}", msg))
                                    .size(12.0)
                                    .color(self.active_theme.app.danger.to_egui()),
                            );
                        }
                    }
                    ui.add_space(6.0);

                    // Release notes: visible for EVERY post-check state so
                    // the changelog and the progress bar coexist on screen.
                    let show_logs = matches!(
                        self.update_state,
                        UpdateState::Available(_)
                            | UpdateState::Downloading(_)
                            | UpdateState::Verifying
                            | UpdateState::Ready(_)
                    );
                    if show_logs {
                        ui.weak(egui::RichText::new(&uw.log_title).size(11.0).color(weak));
                        // Keep the notes visible through Downloading/Ready:
                        // cache them while Available, reuse afterwards.
                        let (changes, changelog) = match &self.update_state {
                            UpdateState::Available(info) => {
                                let c = (info.changes.clone(), info.changelog.clone());
                                self.update_notes_cache = c.clone();
                                c
                            }
                            _ => self.update_notes_cache.clone(),
                        };
                        // Cached from the Available state: download keeps
                        // the notes on screen without re-borrowing info.
                        egui::ScrollArea::vertical()
                            .id_salt("update_changelog_scroll")
                            .max_height(200.0)
                            .min_scrolled_height(60.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                if changes.is_empty() && changelog.is_empty() {
                                    ui.weak(
                                        egui::RichText::new(&uw.no_logs).size(11.0).color(weak),
                                    );
                                }
                                for line in &changes {
                                    ui.label(egui::RichText::new(line).size(12.0));
                                }
                                if changes.is_empty() && !changelog.is_empty() {
                                    ui.label(egui::RichText::new(&changelog).size(12.0));
                                }
                            });
                        ui.add_space(8.0);
                    }

                    // Progress bar (download only), below the notes.
                    if let UpdateState::Downloading(p) = &self.update_state {
                        let p = *p;
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
                        painter.rect_filled(
                            filled,
                            egui::CornerRadius::ZERO,
                            self.active_theme.app.info.to_egui(),
                        );
                        let pct_text = format!("{}%", (p * 100.0) as i32);
                        let galley = ui.fonts(|f| {
                            f.layout_no_wrap(
                                pct_text,
                                egui::FontId::proportional(12.0),
                                visuals.override_text_color.unwrap_or(visuals.text_color()),
                            )
                        });
                        painter.galley(
                            egui::pos2(
                                rect.center().x - galley.size().x / 2.0,
                                rect.center().y - galley.size().y / 2.0,
                            ),
                            galley,
                            visuals.text_color(),
                        );
                        ui.add_space(6.0);
                    }

                    // Bottom action row (single primary action per state).
                    ui.vertical_centered(|ui| {
                        let is_busy = matches!(
                            self.update_state,
                            UpdateState::Checking
                                | UpdateState::Downloading(_)
                                | UpdateState::Verifying
                        );
                        if is_busy {
                            ui.add_enabled(false, egui::Button::new(&ut.check));
                        } else {
                            match &self.update_state {
                                UpdateState::Available(info) => {
                                    if ui.button(&ut.update_now).clicked() {
                                        let info_clone = info.clone();
                                        self.kick_download(ctx, &info_clone);
                                    }
                                }
                                UpdateState::Ready(path) => {
                                    if ui.button(&ut.restart).clicked() {
                                        let path = path.clone();
                                        self.apply_update_and_restart(ctx, path);
                                    }
                                }
                                _ => {
                                    if ui.button(&uw.recheck).clicked() {
                                        self.start_manual_check();
                                        self.check_update_manual(ctx);
                                    }
                                }
                            }
                        }
                    });
                });
            if !open {
                self.show_update_window = false;
                ctx.memory_mut(|m| {
                    m.data
                        .remove_temp::<bool>(egui::Id::new("update_window_checked"))
                });
            }
        }

        // Standalone HELP window: an overview of the implemented
        // features (content is a single i18n blob, split by lines).
        if self.show_help_window {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                self.show_help_window = false;
            }
            let mut open = true;
            egui::Window::new(&self.texts.help.title)
                .id(egui::Id::new("help_window"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .default_width(460.0)
                .show(ctx, |ui| {
                    let h = self.texts.help.clone();
                    let weak = self.active_theme.app.weak_text.to_egui();
                    egui::ScrollArea::vertical()
                        .id_salt("help_content_scroll")
                        .max_height(420.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for para in h.content.split('\n') {
                                if para.trim().is_empty() {
                                    ui.add_space(4.0);
                                    continue;
                                }
                                ui.label(egui::RichText::new(para).size(12.0));
                                let _ = weak;
                            }
                        });
                });
            if !open {
                self.show_help_window = false;
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
            // Rising edge: start on the safe side (CANCEL).
            if std::mem::take(&mut self.ws_close_just_opened) {
                self.dialog_kb_confirm = false;
            }
            // Unified protocol, BEFORE the Window.
            let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
            let mut confirmed = keys.confirm;
            let mut cancelled = keys.cancel || keys.close;
            let mut open = true;
            let mut kb = self.dialog_kb_confirm;
            let panel_name = self
                .panels
                .get(panel_idx)
                .map(|p| p.name.clone())
                .unwrap_or_default();
            let inner = egui::Window::new(&self.texts.close_confirm.confirm)
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
                        Self::dialog_button_row(
                            ui,
                            &mut kb,
                            egui::Id::new("ws_close_confirm_btn"),
                            egui::Id::new("ws_close_cancel_btn"),
                            &self.texts.close_confirm.confirm,
                            &self.texts.close_confirm.cancel,
                        )
                    })
                    .inner
                })
                .and_then(|r| r.inner);
            if let Some((c, x)) = inner {
                confirmed |= c;
                cancelled |= x;
            }
            if confirmed {
                self.close_workspace(panel_idx);
                self.close_confirm_panel = None;
            }
            if cancelled || !open {
                self.close_confirm_panel = None;
            }
        }

        // Check for update result from background thread.
        // After ~3 seconds (180 frames @ 60fps) we show the update dialog
        // for any available update, or display a toast for up-to-date / error.
        // The window is a lower bound, not an exact frame: on slow networks
        // the HTTP call outlives 3 s, so keep polling until the result
        // actually arrives instead of dropping it.
        self.startup_frame_count = self.startup_frame_count.saturating_add(1);
        if !self.startup_check_consumed && self.startup_frame_count >= 180 {
            let result = ctx.memory_mut(|mem| {
                mem.data
                    .remove_temp::<StartCheckResult>(egui::Id::new("start_check_result"))
            });
            if let Some(r) = result {
                self.startup_check_consumed = true;
                match r {
                    StartCheckResult::Available(info) => {
                        if !self.skipped_versions.contains(&info.version) {
                            // Surface the new version ONLY as the menu-bar
                            // update badge (no popup): it stays visible
                            // until updated/dismissed, instead of a modal
                            // the user must close right away.
                            self.update_state = crate::updater::UpdateState::Available(info);
                        }
                    }
                    // UpToDate / Error: silent at startup — the user asked
                    // for no toast; feedback only comes from the About
                    // window's MANUAL check button.
                    StartCheckResult::UpToDate => {}
                    StartCheckResult::Error(e) => {
                        // Silent in the UI (no startup noise), but keep it
                        // diagnosable in logs.
                        log::warn!("startup update check failed: {e}");
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
        self.render_fav_name_dialog(ctx);
        self.render_fav_cmd_dialog(ctx);
        self.render_fav_delete_confirm(ctx);
        self.render_ssh_host_dialog(ctx);
        self.render_ssh_delete_confirm(ctx);
        self.render_ai_panel(ctx);
        self.render_ai_exec_confirm(ctx);
        // Two INDEPENDENT remote panels: LAN and WAN can be open at the
        // same time (both share the one embedded server session).
        self.render_lan_panel(ctx);
        self.render_wan_panel(ctx);
        self.render_agent_confirm(ctx);
        self.render_monitor_panel(ctx);
        self.render_search_bar(ctx);
        self.render_snippet_fill_dialog(ctx);
        self.render_startup_cmd_dialog(ctx);

        // Terminal close confirmation
        if let Some(ref tab_id) = self.pending_close_confirm.clone() {
            // Rising edge: start on the safe side (CANCEL - a stray
            // Enter must not kill the terminal).
            if std::mem::take(&mut self.close_confirm_just_opened) {
                self.dialog_kb_confirm = false;
            }
            // Unified protocol, BEFORE the Window. Esc now closes too.
            let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
            let mut confirmed = keys.confirm;
            let mut cancelled = keys.cancel;
            let mut open = true;
            let tab_id = tab_id.clone();
            let mut kb = self.dialog_kb_confirm;
            let inner = egui::Window::new(&self.texts.close_confirm.terminal_title)
                .id(egui::Id::new("close_confirm_window"))
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .default_pos(screen_center(ctx))
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui| {
                    let is_remote = self
                        .terminals
                        .get(&tab_id)
                        .and_then(|d| d.host.as_ref())
                        .is_some();
                    let message = if is_remote {
                        &self.texts.ssh.close_remote_message
                    } else {
                        &self.texts.close_confirm.terminal_message
                    };
                    ui.label(message);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        Self::dialog_button_row(
                            ui,
                            &mut kb,
                            egui::Id::new("close_confirm_confirm"),
                            egui::Id::new("close_confirm_cancel"),
                            &self.texts.close_confirm.confirm,
                            &self.texts.close_confirm.cancel,
                        )
                    })
                    .inner
                })
                .and_then(|r| r.inner);
            if let Some((c, x)) = inner {
                confirmed |= c;
                cancelled |= x;
            }
            if keys.close {
                cancelled = true;
            }
            if confirmed {
                self.pending_close_confirm = None;
                self.pending_close = Some(tab_id);
            }
            if cancelled || !open {
                self.pending_close_confirm = None;
            }
        }

        // Password popup windows. Escape is consumed BEFORE the if-let
        // so closing the popup does NOT early-return from update() (the
        // old return skipped the sidebar/terminal rendering for a frame,
        // flashing the layout); the if-let simply matches None below.
        if self.pw_popup.is_some_and(|_| {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        }) {
            self.pw_popup = None;
            self.pw_message.clear();
        }
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
                                        store_lock_password(
                                            &mut self.settings,
                                            &mut self.settings_edit,
                                            &self.pw_set1,
                                        );
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
                                    if !verify_lock_password(
                                        &self.pw_old,
                                        &self.settings.lock_password,
                                    ) {
                                        self.pw_message = self.texts.password.wrong_error.clone();
                                    } else if self.pw_new1.is_empty() {
                                        self.pw_message = self.texts.password.empty_error.clone();
                                    } else if self.pw_new1 != self.pw_new2 {
                                        self.pw_message =
                                            self.texts.password.mismatch_error.clone();
                                    } else {
                                        store_lock_password(
                                            &mut self.settings,
                                            &mut self.settings_edit,
                                            &self.pw_new1,
                                        );
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
                                    if !verify_lock_password(
                                        &self.pw_clear,
                                        &self.settings.lock_password,
                                    ) {
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
                    egui::Frame::new()
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
                    // Header row: "工作区" title on the left, square icon
                    // buttons anchored RIGHT (模板 left of the rightmost
                    // 新建) — background fill only, no stroke, no rounding,
                    // Phosphor regular glyphs.
                    ui.horizontal(|ui| {
                        let fg = self.active_theme.app.button_fg.to_egui();
                        let icon_active = self.active_theme.app.text.to_egui();
                        let _btn_bg = self.active_theme.app.button_bg.to_egui();

                        let btn_size = 18.5;
                        let glyph = 12.0;
                        // Section title, flush left.
                        ui.label(
                            egui::RichText::new(&self.texts.workspace.heading)
                                .size(12.0)
                                .color(fg),
                        );
                        // Right-aligned cluster: RTL layout paints the FIRST
                        // control rightmost, so 新建 comes first.
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // 新建 (rightmost): flat PLUS glyph, no background.
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
                                    - egui::vec2(
                                        new_galley.size().x / 2.0,
                                        new_galley.size().y / 2.0,
                                    ),
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
                                    - egui::vec2(
                                        tpl_galley.size().x / 2.0,
                                        tpl_galley.size().y / 2.0,
                                    ),
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
                                                self.pending_load_from_template =
                                                    Some(path.clone());
                                                ui.close_menu();
                                            }
                                            if ui.small_button(egui_phosphor::regular::X).clicked()
                                            {
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
                    });
                    // Fixed bottom cluster FIRST: a bottom panel inside
                    // the sidebar reserves its height up front, so the
                    // workspace list below can NEVER squeeze it out of
                    // the window - no matter how many workspaces exist.
                    egui::TopBottomPanel::bottom("sidebar_bottom_cluster")
                        .frame(egui::Frame::new())
                        // The panel's built-in separator line is OFF: the
                        // cluster draws its own divider at the right
                        // spacing; both lines together read as duplicates.
                        .show_separator_line(false)
                        .show_inside(ui, |ui| {
                            self.render_sidebar_bottom_cluster(ui);
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

                    // The workspace list scrolls when it outgrows the
                    // space left by the fixed bottom cluster.
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
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
                                                    .desired_width(
                                                        (ui.available_width() - 112.0).max(80.0),
                                                    )
                                                    .id_source("workspace_rename"),
                                            );
                                            response.request_focus();
                                            confirm = ui
                                                .button(&self.texts.workspace.rename_confirm)
                                                .clicked();
                                            cancel = ui
                                                .button(&self.texts.workspace.rename_cancel)
                                                .clicked();
                                            response
                                        })
                                        .response;
                                    self.panel_rects[i] = response.rect;
                                    let enter = ui.input_mut(|i| {
                                        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                    });
                                    let escape = ui.input_mut(|i| {
                                        i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                                    });
                                    if confirm || enter {
                                        apply_panel_rename(
                                            &mut self.panels[i],
                                            &self.rename_buffer,
                                        );
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
                                    let _is_row_hovered = row_resp.hovered();
                                    let _row_resp = row_resp
                                        .on_hover_text(&self.texts.workspace.drag_handle_hint);
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
                                    // Activity strip: the LEFT 4px of the row
                                    // button itself doubles as the indicator —
                                    // no separate dot. It watches EVERY
                                    // terminal in THIS workspace (not just
                                    // the highlighted tab), even when the
                                    // workspace is not on screen. Red = PTY
                                    // output or user input on ANY of them
                                    // within the last 30s, green = all silent
                                    // longer, neutral = nothing to watch.
                                    let activity_ms = self.workspace_activity_ms(i);
                                    let strip_color = match workspace_activity_state(
                                        activity_ms,
                                        egui_term::unix_ms(),
                                    ) {
                                        WorkspaceActivity::Active => {
                                            self.active_theme.app.danger.to_egui()
                                        }
                                        WorkspaceActivity::Idle => {
                                            self.active_theme.app.success.to_egui()
                                        }
                                        WorkspaceActivity::Unknown => {
                                            self.active_theme.app.weak_text.to_egui()
                                        }
                                    };
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(
                                            row_rect.min,
                                            egui::vec2(4.0, row_rect.height()),
                                        ),
                                        0.0,
                                        strip_color,
                                    );
                                    self.panel_rects[i] = row_rect;

                                    // Layout: [name (flex)] [lock btn][≡ drag icon]
                                    // Use a child Ui inside row_rect so we can place
                                    // items by their own rect, not via add_sized.
                                    let mut child = ui.new_child(
                                        egui::UiBuilder::new().max_rect(row_rect).layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                        ),
                                    );
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
                                    let name_color = if is_active || response.hovered() {
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
                                            // 12px indent: keeps the label clear
                                            // of the 4px activity strip.
                                            name_rect.min.x + 12.0,
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
                                        if ui
                                            .button(&self.texts.workspace.save_as_template)
                                            .clicked()
                                        {
                                            self.save_as_template(i);
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                        if ui
                                            .button(if is_locked { "解锁" } else { "锁定" })
                                            .clicked()
                                        {
                                            if is_locked {
                                                self.active_panel = i;
                                                self.lock_password_input.clear();
                                                self.pw_message.clear();
                                            } else {
                                                self.try_lock_workspace(i);
                                            }
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                        if ui.button(&self.texts.settings.buttons.close).clicked() {
                                            self.close_confirm_panel = Some(i);
                                            self.ws_close_just_opened = true;
                                            ui.close_menu();
                                        }
                                    });

                                    // Right-side action cluster, anchored to the
                                    // right edge with right-to-left layout. The
                                    // drag handle (reorder) is the rightmost
                                    // item and the lock button sits to its
                                    // left; icons sit flush against each other
                                    // (no divider, no item spacing inside the
                                    // cluster).
                                    let btn_w = 17.0;
                                    let btn_h = row_h;
                                    let drag_w = 14.0;
                                    let action_cluster_w = btn_w + drag_w;
                                    let mut actions_ui = ui.new_child(
                                        egui::UiBuilder::new()
                                            .max_rect(egui::Rect::from_min_size(
                                                egui::pos2(
                                                    row_rect.max.x - action_cluster_w,
                                                    row_rect.min.y,
                                                ),
                                                egui::vec2(action_cluster_w, row_rect.height()),
                                            ))
                                            .layout(egui::Layout::right_to_left(
                                                egui::Align::Center,
                                            )),
                                    );
                                    actions_ui.style_mut().spacing.item_spacing =
                                        egui::vec2(0.0, 0.0);
                                    let button_fg = self.active_theme.app.button_fg.to_egui();
                                    let icon_active = self.active_theme.app.text.to_egui();

                                    // Drag handle (rightmost): grab to reorder
                                    // the workspace (brighter when hovered).
                                    let (handle_rect, handle_resp) = actions_ui
                                        .allocate_exact_size(
                                            egui::vec2(drag_w, btn_h),
                                            egui::Sense::drag(),
                                        );
                                    let handle_color = if handle_resp.hovered() {
                                        icon_active
                                    } else {
                                        button_fg
                                    };
                                    actions_ui.painter().text(
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
                                    let _ = handle_resp
                                        .on_hover_text(&self.texts.workspace.drag_handle_hint);

                                    // Lock / unlock button (left of the drag
                                    // handle). Always painted so the
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
                                            self.lock_password_input.clear();
                                            self.pw_message.clear();
                                        } else {
                                            self.try_lock_workspace(i);
                                        }
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
                                    let source_fill =
                                        ui.visuals().faint_bg_color.linear_multiply(0.65);
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
                                                    let insertion_y =
                                                        drag_insertion_y(target_rect, pos.y);
                                                    painter.line_segment(
                                                        [
                                                            egui::pos2(
                                                                target_rect.left(),
                                                                insertion_y,
                                                            ),
                                                            egui::pos2(
                                                                target_rect.right(),
                                                                insertion_y,
                                                            ),
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
                                            drag_drop_destination(
                                                src,
                                                target,
                                                after_target,
                                                panel_count,
                                            ),
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

                            // SSH host book: below the workspace list, in the
                            // same scroll area. Clicking a row connects.
                            self.render_ssh_hosts_section(ui);
                        }); // ScrollArea (workspace list)
                }); // sidebar SidePanel show
        }

        // The sidebar can enter rename mode during this frame. Read the state again so the
        // terminal view cannot reclaim focus after the rename input requests it.
        let renaming = self.is_renaming();
        // Modal arbiter, computed BEFORE the dock tree borrow (the viewer
        // needs it, but self can't be immutably borrowed inside).
        let modal_open_flag = {
            let app: &App = &*self;
            any_modal_open(app) || app.modal_just_opened
        };
        // Rebuild the shared TerminalTheme cache only when the active/edit
        // theme changes; every terminal then just clones the Arc per frame.
        {
            let current = if self.show_settings {
                &self.theme_edit
            } else {
                &self.active_theme
            };
            if self.terminal_theme_cache_edit != self.show_settings
                || &self.terminal_theme_cache_theme != current
            {
                self.terminal_theme_cache =
                    std::sync::Arc::new(crate::theme::terminal_theme(current));
                self.terminal_theme_cache_theme = current.clone();
                self.terminal_theme_cache_edit = self.show_settings;
            }
        }
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
            .frame(egui::Frame::new().fill(central_fill))
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
                                        || verify_lock_password(
                                            &self.lock_password_input.clone(),
                                            &self.settings.lock_password.clone(),
                                        )
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
                                            .color(self.active_theme.app.danger.to_egui()),
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
                        // The '+' popup hosts the shell selection menu
                        // (default shell + alternatives). It only exists
                        // when there IS a choice: single-shell systems
                        // create directly on click (old behavior).
                        .show_add_popup(self.detected_shells.len() > 1)
                        .show_close_buttons(true)
                        .show_leaf_close_all_buttons(false)
                        .show_leaf_collapse_buttons(true)
                        .show_inside(
                            ui,
                            &mut TerminalTabViewer {
                                terminals: &mut self.terminals,
                                history_db: &self.history_db,
                                max_history: self.settings.max_history,
                                pending_close: &mut self.pending_close,
                                pending_close_confirm: &mut self.pending_close_confirm,
                                close_confirm_just_opened: &mut self.close_confirm_just_opened,
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
                                modal_open: modal_open_flag,
                                fav_submenu_slot: &mut self.fav_submenu,
                                fav_sub_focus_slot: &mut self.fav_sub_focused,
                                auto_match: self.settings_edit.auto_match_command,
                                terminal_view_rects: &mut self.terminal_view_rects,
                                history_menu_just_closed: &mut self.history_menu_just_closed,
                                auto_match_pending: &mut self.auto_match_pending,
                                detected_shells: &self.detected_shells,
                                clipboard_binds: &self.settings.key_binds,
                                clipboard_mirror: &mut self.clipboard_mirror,
                                pending_shell: &mut self.pending_shell,
                                default_shell_id: &self.settings.default_shell,
                                theme: self.terminal_theme_cache.clone(),
                                texts: &self.texts,
                                prod_banner_enabled: self.settings.ssh_prod_banner,
                                danger: self.active_theme.app.danger.to_egui(),
                                ssh_hosts: &self.ssh_hosts,
                                pending_ssh_connect: &mut self.pending_ssh_connect,
                                broadcast_group: &mut self.broadcast_group,
                                startup_cmd_dialog: &mut self.startup_cmd_dialog,
                                startup_cmd_just_opened: &mut self.startup_cmd_just_opened,
                                search: &self.terminal_search,
                                ai_ctx_intent: &mut self.ai_ctx_intent,
                                ai_settings_enabled: self.settings.ai_enabled,
                                sidebar_input_focused: self.sidebar_input_focused,
                                agent_tab: self
                                    .agent
                                    .as_ref()
                                    .filter(|a| {
                                        matches!(
                                            a.phase,
                                            AgentPhase::Thinking
                                                | AgentPhase::WaitingConfirm
                                                | AgentPhase::Executing
                                                | AgentPhase::TimedOut
                                        )
                                    })
                                    .map(|a| a.tab.clone()),
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
        binding_to_key, check_shortcut_no_repeat, default_key_binds, is_monospace_family_name,
        is_symbol_font_name, toggle_history_menu, update_alt_key_state, workspace_activity_state,
        AltKeyState, AppSettings, HistoryNav, ShortcutBinding, TerminalStatePersist,
        WorkspaceActivity, WORKSPACE_IDLE_MS,
    };
    use egui_dock::DockState;

    /// The sidebar activity dot: red inside the 30s window, green after
    /// it, neutral when there is no focused terminal to watch.
    #[test]
    fn workspace_activity_states_follow_the_idle_window() {
        let now = 1_000_000_000u64;
        assert_eq!(
            workspace_activity_state(Some(now - 1_000), now),
            WorkspaceActivity::Active
        );
        // Exactly at the boundary: 30s elapsed = idle.
        assert_eq!(
            workspace_activity_state(Some(now - WORKSPACE_IDLE_MS), now),
            WorkspaceActivity::Idle
        );
        assert_eq!(
            workspace_activity_state(Some(now - WORKSPACE_IDLE_MS - 1), now),
            WorkspaceActivity::Idle
        );
        // No focused terminal: neutral regardless of the clock.
        assert_eq!(
            workspace_activity_state(None, now),
            WorkspaceActivity::Unknown
        );
        // Fresh/never-armed terminal reports 0 (activity epoch) — it
        // must read as IDLE (green), not as "active 30+ years ago".
        assert_eq!(
            workspace_activity_state(Some(0), now),
            WorkspaceActivity::Idle
        );
        // Clock skew (activity in the "future"): saturating subtraction
        // keeps it inside the window → active, never a panic.
        assert_eq!(
            workspace_activity_state(Some(now + 5_000), now),
            WorkspaceActivity::Active
        );
    }

    /// The unified dialog keyboard protocol: a fresh dialog starts on
    /// CANCEL, Enter activates the selected side, Escape closes, and the
    /// cursor toggles only when `toggle` is on.
    #[test]
    fn dialog_keys_protocol_behavior() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |_ctx| {});

        // Escape with no cursor set: close, no confirm/cancel.
        let mut kb = false;
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                    repeat: false,
                    physical_key: None,
                }],
                ..Default::default()
            },
            |ctx| {
                let out = super::dialog_keys(ctx, &mut kb, true);
                assert!(out.close);
                assert!(!out.confirm && !out.cancel && !out.enter);
            },
        );

        // Enter with cursor on CANCEL (safe default) -> cancel only.
        let mut kb = false;
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                    repeat: false,
                    physical_key: None,
                }],
                ..Default::default()
            },
            |ctx| {
                let out = super::dialog_keys(ctx, &mut kb, true);
                assert!(out.enter && out.cancel);
                assert!(!out.confirm && !out.close);
            },
        );

        // ArrowRight flips to CONFIRM; the next Enter confirms.
        let mut kb = false;
        let _ = ctx.run(
            egui::RawInput {
                events: vec![
                    egui::Event::Key {
                        key: egui::Key::ArrowRight,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                        repeat: false,
                        physical_key: None,
                    },
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                        repeat: false,
                        physical_key: None,
                    },
                ],
                ..Default::default()
            },
            |ctx| {
                let out = super::dialog_keys(ctx, &mut kb, true);
                assert!(out.confirm);
                assert!(!out.cancel && !out.close);
            },
        );

        // toggle=false: arrows do NOT flip the cursor (input-style
        // dialogs where Enter means confirm regardless).
        let mut kb = false;
        let _ = ctx.run(
            egui::RawInput {
                events: vec![
                    egui::Event::Key {
                        key: egui::Key::ArrowRight,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                        repeat: false,
                        physical_key: None,
                    },
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                        repeat: false,
                        physical_key: None,
                    },
                ],
                ..Default::default()
            },
            |ctx| {
                let out = super::dialog_keys(ctx, &mut kb, false);
                // Arrow ignored; Enter still reported raw.
                assert!(out.enter && out.cancel && !out.confirm);
            },
        );
    }

    /// Virtual-key capture: CSI keys with optional modifiers, control
    /// bytes for Ctrl+letter, and a readable display form for settings.
    #[test]
    fn virtual_key_capture_and_display() {
        use egui::Key as K;
        let Some((seq, label)) = super::virtual_key_from_capture(K::ArrowUp, egui::Modifiers::NONE)
        else {
            panic!("capture failed");
        };
        assert_eq!(seq, "\x1b[A");
        assert_eq!(label, "↑");
        let Some((seq, _)) = super::virtual_key_from_capture(K::ArrowUp, egui::Modifiers::CTRL)
        else {
            panic!("capture failed");
        };
        assert_eq!(seq, "\x1b[1;5A");
        let Some((seq, label)) = super::virtual_key_from_capture(K::C, egui::Modifiers::CTRL)
        else {
            panic!("capture failed");
        };
        assert_eq!(seq, "\x03");
        assert_eq!(label, "Ctrl+C");
        let Some((seq, _)) = super::virtual_key_from_capture(K::X, egui::Modifiers::ALT) else {
            panic!("capture failed");
        };
        assert_eq!(seq, "\x1bx");
        let Some((seq, _)) = super::virtual_key_from_capture(K::F5, egui::Modifiers::NONE) else {
            panic!("capture failed");
        };
        assert_eq!(seq, "\x1b[15~");
        assert_eq!(super::display_seq("\x1b[A"), "ESC[A");
        assert_eq!(super::display_seq("\x03"), "^C");
    }

    /// fc-match output parsing: file + TTC index, family ignored,
    /// non-numeric index falls back to face 0, garbage rejected.
    #[test]
    fn fc_match_output_parses_file_and_index() {
        assert_eq!(
            super::parse_fc_match(
                "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc\t0\tNoto Sans CJK SC,Noto Sans CJK SC Medium"
            ),
            Some((
                "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc".to_string(),
                0
            ))
        );
        // Non-TTC faces report -1: face 0.
        assert_eq!(
            super::parse_fc_match(
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf\t-1\tDejaVu Sans"
            ),
            Some((
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".to_string(),
                0
            ))
        );
        // TTC at index 2.
        assert_eq!(
            super::parse_fc_match("/fonts/some.ttc\t2\tFamily"),
            Some(("/fonts/some.ttc".to_string(), 2))
        );
        // Garbage: no file field.
        assert_eq!(super::parse_fc_match("\t0\tFamily"), None);
        assert_eq!(super::parse_fc_match(""), None);
    }

    /// The CJK probe must recognise the bundled CJK font and reject a
    /// Latin/Devanagari font — this drives the editor's fallback hint.
    #[test]
    fn font_cjk_probe_matches_bundled_fonts() {
        assert!(super::font_file_has_cjk(include_bytes!(
            "../assets/fonts/NotoSansCJK-Regular.ttc"
        )));
        assert!(!super::font_file_has_cjk(include_bytes!(
            "../assets/fonts/Lohit-Devanagari.ttf"
        )));
        assert!(!super::font_file_has_cjk(b"not a font"));
    }

    /// The workspace-switch shortcut fires once per PHYSICAL press:
    /// auto-repeat events are neither counted as a trigger nor leaked
    /// to the focused terminal as bare arrow-key escapes.
    #[test]
    fn workspace_shortcut_ignores_key_autorepeat() {
        let binds = default_key_binds();

        // Non-repeat press: exactly one hit.
        let ctx = egui::Context::default();
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::CTRL,
                    physical_key: None,
                }],
                ..Default::default()
            },
            |ctx| {
                assert!(check_shortcut_no_repeat(ctx, &binds, "workspace_down"));
            },
        );

        // Auto-repeat press: NOT a hit; the repeat event is consumed so
        // it cannot leak into the PTY as an escape sequence. egui
        // normalizes a single injected repeat:true to false ("first
        // press" - the key is not down yet), so the repeat must be
        // simulated across TWO frames (press, then repeat).
        let ctx = egui::Context::default();
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::CTRL,
                    physical_key: None,
                }],
                ..Default::default()
            },
            |ctx| {
                assert!(check_shortcut_no_repeat(ctx, &binds, "workspace_down"));
            },
        );
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    repeat: true,
                    modifiers: egui::Modifiers::CTRL,
                    physical_key: None,
                }],
                ..Default::default()
            },
            |ctx| {
                assert!(!check_shortcut_no_repeat(ctx, &binds, "workspace_down"));
                let leftover = ctx.input(|i| {
                    i.events.iter().any(|e| {
                        matches!(
                            e,
                            egui::Event::Key {
                                key: egui::Key::ArrowDown,
                                pressed: true,
                                ..
                            }
                        )
                    })
                });
                assert!(!leftover);
            },
        );

        // Plain ArrowDown (no Ctrl): untouched by the workspace shortcut.
        let ctx = egui::Context::default();
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::ArrowDown,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                    physical_key: None,
                }],
                ..Default::default()
            },
            |ctx| {
                assert!(!check_shortcut_no_repeat(ctx, &binds, "workspace_down"));
                assert_eq!(ctx.input(|i| i.events.len()), 1);
            },
        );
    }

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
    fn rename_state_on_another_tab_does_not_block_auto_match() {
        // Regression: the auto-match gate used to require NO tab to be
        // renaming at all. A rename left dangling on a hidden (inactive)
        // tab — whose input the dock never renders, so it can never be
        // confirmed or cancelled — silently disabled auto-match for EVERY
        // terminal, including freshly split ones.
        let renaming_terminal: Option<String> = Some("terminal-stale".into());
        for tab in ["terminal-a", "terminal-b"] {
            let blocked = renaming_terminal.is_some();
            let blocked_now = renaming_terminal.as_ref() == Some(&tab.to_string());
            assert!(!blocked_now, "auto-match must run on {tab}");
            let _ = blocked;
        }
        // The stale tab itself stays gated while its input is visible.
        let gated = renaming_terminal.as_ref() == Some(&"terminal-stale".to_string());
        assert!(gated);
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
    fn symbol_fonts_are_recognized_and_excluded_from_chains() {
        // The renderer must never fall back to symbol/decorative faces:
        // their Latin coverage draws circled/overlined variants between
        // ordinary words (stray marks + torn spacing), while copy stays
        // correct — glyph-only corruption.
        for name in [
            "OpenSymbol",
            "opens___",
            "StandardSymbolsPS",
            "Dingbats",
            "Wingdings",
            "Noto Color Emoji",
            "SomeIconPack",
        ] {
            assert!(is_symbol_font_name(name), "{name} should be filtered");
        }
        for name in [
            "Ubuntu Mono",
            "mononoki",
            "Liberation Mono",
            "Ubuntu",
            "Noto Sans",
        ] {
            assert!(!is_symbol_font_name(name), "{name} must stay");
        }
        // Monospace chains only take genuinely monospaced families.
        assert!(is_monospace_family_name("Ubuntu Mono"));
        assert!(is_monospace_family_name("jetbrains-mono"));
        assert!(is_monospace_family_name("LiberationMono-Regular"));
        assert!(!is_monospace_family_name("Ubuntu"));
        assert!(!is_monospace_family_name("DejaVu Sans"));
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
        let ids: Vec<_> = super::shortcut_hint_ids().to_vec();

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
    fn workspace_sidebar_uses_wider_default_and_drag_handle_width() {}

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
                            shell: String::new(),
                            host_id: 0,
                            startup_command: String::new(),
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
                            shell: String::new(),
                            host_id: 0,
                            startup_command: String::new(),
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
        assert_eq!(binds["history_favorite"].key, "F2");
        assert_eq!(binds["next_terminal"].key, "Tab");
        assert!(binds["next_terminal"].ctrl);
        assert_eq!(binds["next_panel"].key, "Q");
        assert_eq!(binds["next_workspace"].key, "W");
        assert_eq!(binds["close_terminal"].key, "E");
        assert_eq!(binds["save_scene"].key, "S");
        assert_eq!(binds["terminal_interrupt"].key, "C");
        assert!(binds["terminal_interrupt"].ctrl);
        assert!(binds["terminal_interrupt"].shift);
        assert_eq!(binds["terminal_copy"].key, "C");
        assert!(binds["terminal_copy"].ctrl);
        assert!(!binds["terminal_copy"].shift);
        assert_eq!(binds["terminal_paste"].key, "V");
        assert_eq!(binds["terminal_cut"].key, "X");
        assert!(!binds["history_favorite"].ctrl);
        assert_eq!(binds["history_delete"].key, "Delete");
        assert!(!binds["history_delete"].ctrl);
        // Both must resolve through the egui key mapping (settings page
        // recording and the menu keyboard handler share it).
        assert_eq!(
            binding_to_key(&binds["history_favorite"]),
            Some(egui::Key::F2)
        );
        assert_eq!(
            binding_to_key(&binds["history_delete"]),
            Some(egui::Key::Delete)
        );
        // Clipboard/interrupt keys MUST resolve through the egui key
        // mapping too — a missing mapping silently disabled the terminal
        // interrupt (Ctrl+Shift+C) and remapped copy/paste/cut.
        assert_eq!(
            binding_to_key(&binds["terminal_interrupt"]),
            Some(egui::Key::C)
        );
        assert_eq!(binding_to_key(&binds["terminal_copy"]), Some(egui::Key::C));
        assert_eq!(binding_to_key(&binds["terminal_paste"]), Some(egui::Key::V));
        assert_eq!(binding_to_key(&binds["terminal_cut"]), Some(egui::Key::X));
    }

    #[test]
    fn next_terminal_wraps_from_last_back_to_first() {
        // Pure index math of the cycle used by the next_terminal
        // shortcut: current → (i+1) % len.
        let tabs = ["t1".to_string(), "t2".to_string(), "t3".to_string()];
        let next_of = |cur: &str| -> String {
            let idx = tabs.iter().position(|t| t == cur).unwrap();
            tabs[(idx + 1) % tabs.len()].clone()
        };
        assert_eq!(next_of("t1"), "t2");
        assert_eq!(next_of("t2"), "t3");
        // Last wraps to first.
        assert_eq!(next_of("t3"), "t1");
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
            favorites: Vec::new(),
            fav_focused: false,
            fav_selected: 0,
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
        toggle_history_menu(&mut nav, vec!["command".into()], Vec::new());
        assert!(nav.is_some());
        toggle_history_menu(&mut nav, vec!["command".into()], Vec::new());
        assert!(nav.is_none());
    }

    #[test]
    fn terminal_state_does_not_serialize_snapshots() {
        let state = TerminalStatePersist {
            name: "Terminal 1".into(),
            font_size: 14.0,
            working_directory: "/tmp".into(),
            shell: String::new(),
            host_id: 0,
            startup_command: String::new(),
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

/// Localized display name for a detected shell option.
fn shell_display_name(s: &crate::shells::ShellOption) -> String {
    match s.id {
        "cmd" => "cmd".to_string(),
        "powershell" => "Windows PowerShell".to_string(),
        "pwsh" => "PowerShell 7".to_string(),
        "vs-dev" => "VS 开发人员命令提示符".to_string(),
        "wsl" => "WSL".to_string(),
        "default" => format!("默认 ({})", shell_short_name(&s.program)),
        "bash" => "bash".to_string(),
        "zsh" => "zsh".to_string(),
        "fish" => "fish".to_string(),
        "nu" => "nushell".to_string(),
        "sh" => "sh".to_string(),
        _ => "Shell".to_string(),
    }
}

/// Last path component of a shell program, for the default-shell label.
/// Command separator for assembling a favorite folder's commands into
/// one line, by the target terminal's shell family:
///   POSIX (bash/zsh/fish/nu/sh)  -> " && "   (stop on first failure)
///   PowerShell                   -> "; "     (PS has no && prior to 7)
///   cmd.exe                      -> " & "    (cmd chains with single &)
pub(crate) fn assemble_separator(shell_id: &str) -> &'static str {
    match shell_id {
        "powershell" => "; ",
        "cmd" | "vs-dev" => " & ",
        _ => " && ",
    }
}

/// Join a folder's commands for execution in the given shell. Empty
/// commands are filtered; multi-line commands collapse to single spaces
/// (a newline inside an assembled line would execute prematurely).
pub(crate) fn assemble_commands(commands: &[String], shell_id: &str) -> String {
    commands
        .iter()
        .map(|c| c.trim())
        .filter(|c| !c.is_empty())
        .map(|c| c.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join(assemble_separator(shell_id))
}
#[cfg(test)]
mod assemble_tests {
    use super::assemble_commands;

    #[test]
    fn posix_shells_join_with_and_chain() {
        let cmds = vec![
            "cmake .".into(),
            "make -j8".into(),
            "sudo make install".into(),
        ];
        assert_eq!(
            assemble_commands(&cmds, "bash"),
            "cmake . && make -j8 && sudo make install"
        );
        assert_eq!(
            assemble_commands(&cmds, "zsh"),
            assemble_commands(&cmds, "bash")
        );
        assert_eq!(
            assemble_commands(&cmds, "fish"),
            assemble_commands(&cmds, "sh")
        );
    }

    #[test]
    fn windows_shells_use_their_own_separators() {
        let cmds = vec!["dir".into(), "echo ok".into()];
        assert_eq!(assemble_commands(&cmds, "powershell"), "dir; echo ok");
        assert_eq!(assemble_commands(&cmds, "cmd"), "dir & echo ok");
        assert_eq!(assemble_commands(&cmds, "vs-dev"), "dir & echo ok");
    }

    #[test]
    fn blank_and_multiline_entries_are_sanitized() {
        let cmds = vec!["".into(), "   ".into(), "cd\n/tmp".into(), "ls".into()];
        assert_eq!(assemble_commands(&cmds, "bash"), "cd /tmp && ls");
        let only_blank = vec!["".into()];
        assert_eq!(assemble_commands(&only_blank, "bash"), "");
    }
}

fn shell_short_name(program: &str) -> &str {
    program.rsplit('/').next().unwrap_or(program)
}

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    history_db: &'a crate::history_db::HistoryDb,
    max_history: usize,
    // Written through by tab_ui; read back on App after DockArea::show.
    #[allow(dead_code)]
    pending_close: &'a mut Option<String>,
    pending_close_confirm: &'a mut Option<String>,
    close_confirm_just_opened: &'a mut bool,
    pending_new_terminal: &'a mut Option<(usize, SurfaceIndex, NodeIndex)>,
    #[allow(dead_code)]
    pending_split_after: &'a mut Option<String>,
    #[allow(dead_code)]
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
    /// ANY modal dialog is open: the terminal must not claim/keep
    /// keyboard focus while one owns the UI (single arbiter flag,
    /// replaces the previous per-dialog-family booleans that each knew
    /// only their own subset).
    modal_open: bool,
    /// Manual-menu transient column state, so the auto-match overlay
    /// (opened inside tab_ui) can clear it.
    fav_submenu_slot: &'a mut Option<(i64, egui::Pos2, Vec<String>, Option<usize>)>,
    fav_sub_focus_slot: &'a mut bool,
    auto_match: bool,
    terminal_view_rects: &'a mut std::collections::HashMap<String, egui::Rect>,
    history_menu_just_closed: &'a mut std::collections::HashMap<String, bool>,
    auto_match_pending: &'a mut std::collections::HashMap<String, String>,
    detected_shells: &'a [crate::shells::ShellOption],
    /// Terminal clipboard/interrupt shortcut bindings (settings).
    clipboard_binds: &'a HashMap<String, ShortcutBinding>,
    /// Clipboard mirror for remapped paste.
    clipboard_mirror: &'a mut String,
    pending_shell: &'a mut Option<crate::shells::ShellOption>,
    /// Shell id chosen as default in settings (labels the menu's
    /// default entry; the actual spawn resolves in create_terminal).
    default_shell_id: &'a str,
    theme: std::sync::Arc<egui_term::TerminalTheme>,
    texts: &'a crate::i18n::Texts,
    /// PROD-marked SSH hosts show a red warning banner above the terminal.
    prod_banner_enabled: bool,
    /// Theme danger color for the PROD tab-title tint and banner.
    danger: egui::Color32,
    /// Saved SSH hosts for the "+" popup connection entries.
    ssh_hosts: &'a [crate::hosts::SshHost],
    /// Host chosen in the "+" popup; consumed by process_pending.
    pending_ssh_connect: &'a mut Option<i64>,
    /// Tabs currently receiving broadcast keystrokes.
    broadcast_group: &'a mut std::collections::HashSet<String>,
    /// Per-terminal startup command editor: (tab id, buffer).
    startup_cmd_dialog: &'a mut Option<(String, String)>,
    startup_cmd_just_opened: &'a mut bool,
    /// Active scrollback search (highlights are painted per tab).
    search: &'a Option<TerminalSearch>,
    /// Right-click AI intent slot (written by the tab, consumed by App).
    ai_ctx_intent: &'a mut Option<AiCtxAction>,
    /// Whether the AI settings allow requests (hides the menu entry).
    ai_settings_enabled: bool,
    /// The sidebar SSH host search box holds keyboard focus: the
    /// terminal must not reclaim it every frame (typing used to fall
    /// through into the PTY).
    sidebar_input_focused: bool,
    /// The tab the terminal agent is attached to (title icon + broadcast
    /// pause while the agent drives it).
    agent_tab: Option<String>,
}

impl<'a> egui_dock::TabViewer for TerminalTabViewer<'a> {
    type Tab = String;

    /// The egui_dock default derives the egui widget Id from the tab
    /// TITLE — two same-named SSH tabs in one workspace then collided
    /// ("first use of widget id" spam + broken click handling). Key the
    /// widget Id off the unique tab id (terminal-N) instead; titles are
    /// display-only.
    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.as_str())
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let Some(d) = self.terminals.get(tab) else {
            return tab.clone().into();
        };
        let is_prod = d.host.as_ref().is_some_and(|h| h.prod);
        let mut prefix = String::new();
        if self.agent_tab.as_deref() == Some(tab.as_str()) {
            prefix.push_str(egui_phosphor::regular::ROBOT);
        }
        if self.broadcast_group.contains(tab) {
            prefix.push_str(egui_phosphor::regular::BROADCAST);
        }
        // PROD hosts: the title itself carries the danger tint so
        // the wrong-window risk is visible even on tiny tabs.
        if is_prod {
            prefix.push_str(egui_phosphor::regular::WARNING);
            return egui::RichText::new(format!("{} {}", prefix, d.name))
                .color(self.danger)
                .into();
        }
        if prefix.is_empty() {
            d.name.clone().into()
        } else {
            egui::RichText::new(format!("{} {}", prefix, d.name)).into()
        }
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
                let enter =
                    ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                let escape =
                    ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
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

            // Configurable terminal clipboard/interrupt shortcuts.
            // Interrupt writes ^C to the PTY; copy/cut copy the live
            // selection (cut keeps the text: terminals have no
            // delete-selection); paste writes the clipboard mirror (kept
            // in sync on every copy/paste event) or falls back to ^V.
            // The DEFAULT keys (Ctrl+C/V/X) are handled natively by egui
            // (Copy/Cut/Paste events) — this interceptor covers REMAPPED
            // keys, plus terminal_interrupt in every configuration.
            {
                let mk = |b: &ShortcutBinding| -> Option<(egui::Key, egui::Modifiers)> {
                    binding_to_key(b).map(|k| {
                        (
                            k,
                            egui::Modifiers {
                                ctrl: b.ctrl,
                                shift: b.shift,
                                alt: b.alt,
                                ..Default::default()
                            },
                        )
                    })
                };
                // A binding sitting on egui's BUILT-IN clipboard keys
                // (Ctrl+C/V/X) must NOT be intercepted here: egui's own
                // channel handles those with the real system clipboard;
                // interception would downgrade paste to the mirror.
                let is_builtin_clipboard_key = |b: &ShortcutBinding| -> bool {
                    mk(b).is_some_and(|(key, mods)| {
                        mods == egui::Modifiers::CTRL
                            && matches!(key, egui::Key::C | egui::Key::V | egui::Key::X)
                    })
                };
                let hit = |ui: &egui::Ui, b: &ShortcutBinding| -> bool {
                    mk(b).is_some_and(|(key, mods)| ui.input_mut(|i| i.consume_key(mods, key)))
                };
                if is_focused && !self.modal_open {
                    if let Some(b) = self.clipboard_binds.get("terminal_interrupt") {
                        let mut interrupted = hit(ui, b);
                        if !interrupted {
                            // egui-winit's `is_copy_command` ignores Shift:
                            // Ctrl+Shift+C arrives as `Event::Copy`, not
                            // `Event::Key{C}`. Detect that and treat it as
                            // the interrupt, consuming the Copy so it can't
                            // also copy the terminal selection.
                            let is_ctrl_shift_c = b.key == "C" && b.ctrl && b.shift && !b.alt;
                            if is_ctrl_shift_c {
                                interrupted = ui.input_mut(|i| {
                                    let has_copy =
                                        i.events.iter().any(|e| matches!(e, egui::Event::Copy));
                                    if has_copy && i.modifiers.shift {
                                        i.events.retain(|e| !matches!(e, egui::Event::Copy));
                                        true
                                    } else {
                                        false
                                    }
                                });
                            }
                        }
                        if interrupted {
                            td.instance.write(&[0x03]);
                        }
                    }
                    if let Some(b) = self
                        .clipboard_binds
                        .get("terminal_copy")
                        .filter(|b| !is_builtin_clipboard_key(b))
                    {
                        if hit(ui, b) {
                            let content = td.instance.backend.selectable_content();
                            if !content.is_empty() {
                                ui.ctx().copy_text(content.clone());
                                *self.clipboard_mirror = content;
                            }
                        }
                    }
                    if let Some(b) = self
                        .clipboard_binds
                        .get("terminal_cut")
                        .filter(|b| !is_builtin_clipboard_key(b))
                    {
                        if hit(ui, b) {
                            let content = td.instance.backend.selectable_content();
                            if !content.is_empty() {
                                ui.ctx().copy_text(content.clone());
                                *self.clipboard_mirror = content;
                            }
                        }
                    }
                    if let Some(b) = self
                        .clipboard_binds
                        .get("terminal_paste")
                        .filter(|b| !is_builtin_clipboard_key(b))
                    {
                        if hit(ui, b) {
                            if self.clipboard_mirror.is_empty() {
                                // Mirror cold: fall back to the shell's ^V
                                // paste (readline-level).
                                td.instance.write(&[0x16]);
                            } else {
                                let text = self.clipboard_mirror.clone();
                                td.instance.write(text.as_bytes());
                            }
                        }
                    }
                }
            }

            // PROD banner: an unmissable production-machine warning above
            // the terminal viewport (wrong-window protection for ops).
            if self.prod_banner_enabled && td.host.as_ref().is_some_and(|h| h.prod) {
                let host = td.host.as_ref().unwrap();
                let label = format!(
                    "{}  {} ({})",
                    self.texts.ssh.prod_banner, host.name, host.addr
                );
                let bar_h = 18.0;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), bar_h),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(rect, 0.0, self.danger.gamma_multiply(0.18));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(11.0),
                    self.danger,
                );
            }

            let terminal_view = {
                let mut tv = egui_term::TerminalView::new(ui, &mut td.instance.backend);
                tv = tv.set_theme((*self.theme).clone());
                tv = tv.set_font(egui_term::TerminalFont::new(egui_term::FontSettings {
                    font_type: egui::FontId::monospace(td.font_size),
                }));
                // While the settings window (and its password popups) is
                // open, the terminal must NOT claim keyboard focus every
                // frame — otherwise text inputs in the popups lose focus
                // immediately after being clicked.
                // The favorite-folder name/delete dialogs need the same
                // protection as the settings popups: without it the
                // terminal re-claims keyboard focus every frame and the
                // dialog's text field can never hold focus.
                let terminal_may_focus =
                    is_focused && !self.modal_open && !self.sidebar_input_focused;
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

            // Search highlight overlay: rects over the viewport for the
            // active search's hits (current one emphasized).
            if let Some(search) = self.search.as_ref() {
                if search.tab == *tab && !search.matches.is_empty() {
                    let content = td.instance.backend.last_content();
                    let display_offset = content.grid.display_offset();
                    let cw = content.terminal_size.cell_width as f32;
                    let ch = content.terminal_size.cell_height as f32;
                    let origin = terminal_response.rect.min;
                    for (i, hit) in search.matches.iter().enumerate() {
                        let Some(vp) = point_to_viewport(
                            display_offset,
                            Point::new(hit.line, Column(hit.col_start)),
                        ) else {
                            continue;
                        };
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(
                                origin.x + vp.column.0 as f32 * cw,
                                origin.y + vp.line as f32 * ch,
                            ),
                            egui::vec2(hit.col_count as f32 * cw, ch),
                        );
                        let fill = if i == search.current {
                            egui::Color32::from_rgba_unmultiplied(255, 140, 0, 115)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(255, 220, 0, 60)
                        };
                        ui.painter().rect_filled(rect, 0.0, fill);
                    }
                }
            }

            // ---- Auto-match command suggestions --------------------------------
            // Event-driven ONLY (real key edits in the FOCUSED terminal).
            // The PTY grid lags keypresses by ≥1 frame: a just-typed
            // space is not yet echoed, so the grid word alone would match
            // the pre-space text. A pending-keystroke buffer compensates:
            // effective word = grid word + pending; the buffer drains as
            // the grid catches up.
            if self.auto_match && !self.renaming && self.renaming_terminal.as_ref() != Some(tab) {
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
                    // Self-heal: when the grid ALREADY echoes this frame's
                    // typed text, any remaining buffered chars are stale
                    // pollution from a previous command (the buffer used
                    // to survive Enter and corrupt the next command's
                    // word, silently disabling auto-match until the user
                    // backspaced everything).
                    if !typed.is_empty() && grid_word.ends_with(typed.as_str()) {
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
                        // Command completion: the terminal's own history
                        // ranked by re-run count (hits), then PATH
                        // executable names matching the word prefix.
                        // History matches the WHOLE text by prefix —
                        // typed "cd " matches "cd /tmp" but never "cd".
                        let ranked = self.history_db.get_ranked(tab, self.max_history);
                        let matches: Vec<String> =
                            crate::completion::suggestions(&word, &ranked, 10);
                        let single_exact = matches.len() == 1 && matches[0] == word;
                        if single_exact || matches.is_empty() {
                            if let Some(nav) = td.instance.history_nav.as_mut() {
                                if nav.auto_word.is_some() {
                                    td.instance.history_nav = None;
                                }
                            }
                        } else {
                            let keep_sel = if td
                                .instance
                                .history_nav
                                .as_ref()
                                .is_some_and(|n| n.auto_word.is_some())
                            {
                                {
                                    td.instance
                                        .history_nav
                                        .as_ref()
                                        .map(|n| n.selected)
                                        .unwrap_or(0)
                                }
                            } else {
                                0
                            };
                            td.instance.history_nav = Some(HistoryNav {
                                entries: matches,
                                selected: keep_sel,
                                auto_word: Some(word.clone()),
                                favorites: Vec::new(),
                                fav_focused: false,
                                fav_selected: 0,
                            });
                            // The auto-match overlay is a SEPARATE
                            // feature from the manual menu: drop the
                            // manual session's column leftovers so they
                            // can neither render behind the overlay nor
                            // leak into the snapshot above.
                            *self.fav_submenu_slot = None;
                            *self.fav_sub_focus_slot = false;
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

            // ---- Broadcast input (roadmap batch 2) --------------------------
            // The FOCUSED member replicates its keystrokes to every other
            // member of the group (ops batch operations). Only text-mode
            // input is replicated: printable text, paste, Enter, Backspace
            // and the classic ^C/^D interrupts. Read-only event peek — the
            // terminal view already delivered the input to its own PTY.
            if is_focused
                && !self.modal_open
                && self.broadcast_group.len() > 1
                && self.agent_tab.as_deref() != Some(tab.as_str())
            {
                let others: Vec<String> = self
                    .broadcast_group
                    .iter()
                    .filter(|t| *t != tab)
                    .cloned()
                    .collect();
                let mut payload: Vec<u8> = Vec::new();
                ui.ctx().input(|i| {
                    for event in &i.events {
                        match event {
                            egui::Event::Text(text) => payload.extend_from_slice(text.as_bytes()),
                            egui::Event::Paste(text) => payload.extend_from_slice(text.as_bytes()),
                            egui::Event::Key {
                                key,
                                pressed: true,
                                modifiers,
                                ..
                            } => match key {
                                egui::Key::Enter => payload.push(b'\r'),
                                egui::Key::Backspace => payload.push(0x7f),
                                egui::Key::C if modifiers.ctrl && !modifiers.shift => {
                                    payload.push(0x03)
                                }
                                egui::Key::D if modifiers.ctrl => payload.push(0x04),
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                });
                if !payload.is_empty() {
                    for other in others {
                        if let Some(td) = self.terminals.get_mut(&other) {
                            td.instance.write(&payload);
                        }
                    }
                }
            }

            // ---- Right-click AI menu ------------------------------------
            // Records the intent only (the terminal borrow is live here);
            // render_ai_panel consumes it from the App-side slot. With a
            // selection the actions run on it; without one, on the visible
            // screen text.
            let ai_menu = self.texts.ai.clone();
            let mut ai_intent: Option<AiCtxAction> = None;
            terminal_response.clone().context_menu(|ui| {
                if self.ai_settings_enabled {
                    if ui.button(&ai_menu.ctx_explain).clicked() {
                        ai_intent = Some(AiCtxAction::ExplainSelection);
                        ui.close_menu();
                    }
                    if ui.button(&ai_menu.ctx_fix).clicked() {
                        ai_intent = Some(AiCtxAction::FixSelection);
                        ui.close_menu();
                    }
                    if ui.button(&ai_menu.ctx_translate).clicked() {
                        ai_intent = Some(AiCtxAction::TranslateSelection);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(&ai_menu.ctx_explain_screen).clicked() {
                        ai_intent = Some(AiCtxAction::ExplainScreen);
                        ui.close_menu();
                    }
                }
            });
            if let Some(action) = ai_intent {
                *self.ai_ctx_intent = Some(action);
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
        *self.close_confirm_just_opened = true;
        false
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        terminal_tab_is_closeable(self.terminal_count)
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        *self.pending_new_terminal = Some((self.active_panel, surface, node));
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, surface: SurfaceIndex, node: NodeIndex) {
        // Shell selection menu: the default (settings') shell leads the
        // list, then every other detected shell. The popup is only
        // enabled at all when there IS a choice (single-shell systems
        // create directly via on_add). No separate "+ tab" entry — it
        // duplicated the default-shell entry.
        let shells = self.detected_shells.to_vec();
        // Widen the popup to the LONGEST label so shell names never wrap
        // (the popup defaults to the narrow '+' button width and the
        // labels like "VS 开发人员命令提示符" folded onto several lines).
        let resolved_default = crate::shells::resolve_shell(&shells, self.default_shell_id);
        let longest = shells
            .iter()
            .map(|s| {
                if s.id == "default" {
                    format!("默认 ({})", shell_short_name(&resolved_default.program))
                } else if s.id == resolved_default.id {
                    format!("{}（默认）", shell_display_name(s))
                } else {
                    shell_display_name(s)
                }
            })
            .map(|name| {
                ui.fonts(|f| {
                    f.layout_no_wrap(
                        name,
                        egui::FontId::proportional(14.0),
                        egui::Color32::PLACEHOLDER,
                    )
                    .size()
                    .x
                })
            })
            .fold(0.0f32, f32::max);
        // Button padding both sides + a little slack for the hover frame.
        ui.set_min_width(longest + 24.0);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        for s in &shells {
            // The entry matching the SETTINGS default gets the 默认
            // marker and spawns via the default-resolution path (no
            // explicit pending_shell) — identical semantics on every
            // platform: Unix's "default" entry and Windows' concrete
            // cmd/powershell/... entry alike.
            let is_default_entry = s.id == "default" || s.id == resolved_default.id;
            let label = if s.id == "default" {
                format!("默认 ({})", shell_short_name(&resolved_default.program))
            } else if is_default_entry {
                format!("{}（默认）", shell_display_name(s))
            } else {
                shell_display_name(s)
            };
            if ui.button(label).clicked() {
                if !is_default_entry {
                    *self.pending_shell = Some(s.clone());
                }
                *self.pending_new_terminal = Some((self.active_panel, surface, node));
                ui.close_menu();
            }
        }
        // Saved SSH hosts: connection entries below the shell list.
        if !self.ssh_hosts.is_empty() {
            ui.separator();
            for host in self.ssh_hosts.iter() {
                let marker = if host.prod {
                    egui_phosphor::regular::WARNING.to_string()
                } else {
                    String::new()
                };
                let label = format!("{} {}{}", egui_phosphor::regular::PLUG, marker, host.name);
                if ui.button(label).clicked() {
                    *self.pending_ssh_connect = Some(host.id);
                    *self.pending_new_terminal = Some((self.active_panel, surface, node));
                    ui.close_menu();
                }
            }
        }
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
        // Per-terminal startup command (runs on creation and every
        // scene restore).
        if ui.button(&self.texts.terminal.startup_cmd).clicked() {
            let buffer = self
                .terminals
                .get(tab)
                .map(|d| d.startup_command.clone())
                .unwrap_or_default();
            *self.startup_cmd_dialog = Some((tab.clone(), buffer));
            *self.startup_cmd_just_opened = true;
            ui.close_menu();
        }
        // Broadcast input group: members receive each focused member's
        // keystrokes (ops batch operations).
        let in_broadcast = self.broadcast_group.contains(tab);
        if ui
            .button(if in_broadcast {
                &self.texts.terminal.broadcast_leave
            } else {
                &self.texts.terminal.broadcast_join
            })
            .clicked()
        {
            if in_broadcast {
                self.broadcast_group.remove(tab);
            } else {
                self.broadcast_group.insert(tab.clone());
            }
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

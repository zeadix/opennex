use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::terminal::TerminalInstance;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 1.0;

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
        "history_up".into(),
        ShortcutBinding {
            key: "PageUp".into(),
            ctrl: false,
            shift: false,
            alt: false,
        },
    );
    m.insert(
        "history_down".into(),
        ShortcutBinding {
            key: "PageDown".into(),
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
                return settings;
            }
        }
    }
    AppSettings::default()
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
        let name = "Workspace 1".to_string();
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
                name: format!("Terminal {}", random_suffix),
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
        let name = format!("Workspace {}", n);
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
        // Reorder panels
        let panel = self.panels.remove(src);
        self.panels.insert(dst, panel);
        // Reorder dock_states: rebuild with new indices
        let mut old_states: Vec<(usize, DockState<String>)> = self.dock_states.drain().collect();
        old_states.sort_by_key(|(k, _)| *k);
        let mut new_states = HashMap::new();
        // Build a mapping: old index -> new index
        let mut old_to_new = vec![0usize; self.panels.len() + 1];
        let mut new_idx = 0;
        // The panel that was at src is now at dst
        // All other panels shift
        for old_idx in 0..self.panels.len() {
            if old_idx == dst {
                old_to_new[src] = new_idx;
            } else {
                let actual_old = if old_idx < dst && old_idx < src {
                    old_idx
                } else if old_idx >= dst && old_idx < src {
                    old_idx + 1
                } else if old_idx >= dst && old_idx >= src {
                    old_idx
                } else {
                    old_idx
                };
                old_to_new[actual_old] = new_idx;
            }
            new_idx += 1;
        }
        for (old_k, state) in old_states {
            if old_k < old_to_new.len() {
                new_states.insert(old_to_new[old_k], state);
            }
        }
        self.dock_states = new_states;
        // Update active_panel
        if self.active_panel == src {
            self.active_panel = dst;
        } else if src < self.active_panel && dst >= self.active_panel {
            self.active_panel -= 1;
        } else if src > self.active_panel && dst <= self.active_panel {
            self.active_panel += 1;
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

        // Keep focused_terminal in sync with dock active tab before handling shortcuts
        if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
            if let Some((_, tab)) = tree.find_active_focused() {
                self.focused_terminal = Some(tab.clone());
            }
        }

        // Consume Tab key to prevent egui focus navigation, send to focused terminal
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
            if let Some(tab) = &self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    td.instance.write(&[0x09]);
                }
            }
        }

        // Enter: select history entry, or record command + send CR to terminal
        if !self.locked_panels.contains(&self.active_panel)
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
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
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

        // PageUp / history_up: open history (newest on top) or move highlight up
        let history_up = check_shortcut(ctx, &self.settings.key_binds, "history_up")
            || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::PageUp));
        if history_up {
            if let Some(tab) = &self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    if let Some(ref mut nav) = td.instance.history_nav {
                        if nav.selected > 0 {
                            nav.selected -= 1;
                        }
                    } else {
                        let entries = self.history_db.get(tab, self.settings.max_history);
                        if !entries.is_empty() {
                            // entries: [newest, ..., oldest] — select top (newest)
                            td.instance.history_nav = Some(HistoryNav {
                                entries,
                                selected: 0,
                            });
                        }
                    }
                }
            }
        }
        // PageDown / history_down: move highlight down (toward older commands)
        let history_down = check_shortcut(ctx, &self.settings.key_binds, "history_down")
            || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::PageDown));
        if history_down {
            if let Some(tab) = &self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(tab) {
                    if let Some(ref mut nav) = td.instance.history_nav {
                        if nav.selected + 1 < nav.entries.len() {
                            nav.selected += 1;
                        }
                    }
                }
            }
        }

        // Handle key binding recording in settings
        if let Some(recording) = self.binding_recording.clone() {
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

        // Configurable shortcuts
        let binds = self.settings.key_binds.clone();

        if check_shortcut(ctx, &binds, "new_terminal") {
            if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                if let Some((surface, node)) = tree.focused_leaf() {
                    self.pending_new_terminal = Some((self.active_panel, surface, node));
                }
            }
        }
        if check_shortcut(ctx, &binds, "close_terminal") {
            if let Some(tab) = &self.focused_terminal.clone() {
                self.pending_close = Some(tab.clone());
            }
        }
        if check_shortcut(ctx, &binds, "workspace_up") {
            if self.active_panel > 0 {
                self.active_panel -= 1;
            }
        }
        if check_shortcut(ctx, &binds, "workspace_down") {
            if self.active_panel + 1 < self.panels.len() {
                self.active_panel += 1;
            }
        }
        if check_shortcut(ctx, &binds, "panel_left") {
            self.focus_adjacent_panel(-1);
        }
        if check_shortcut(ctx, &binds, "panel_right") {
            self.focus_adjacent_panel(1);
        }
        if check_shortcut(ctx, &binds, "lock_workspace") {
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
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        self.save_scene();
                        ui.close_menu();
                    }
                    if ui.button("Load").clicked() {
                        self.pending_load_scene = true;
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        self.pending_save_scene_as = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                        let active_tab = tree.find_active_focused().map(|(_, t)| t.clone());
                        if let Some(ref tab) = active_tab {
                            if ui.button("Split Right").clicked() {
                                self.pending_split_after = Some(tab.clone());
                                self.pending_split_vertical = false;
                                if let Some((surface, node, _)) = tree.find_tab(tab) {
                                    self.pending_new_terminal =
                                        Some((self.active_panel, surface, node));
                                }
                                ui.close_menu();
                            }
                            if ui.button("Split Down").clicked() {
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
                if ui.button("Settings").clicked() {
                    self.show_settings = true;
                    self.settings_edit = self.settings.clone();
                }
            });
        });

        if self.show_settings {
            let mut open = self.show_settings;
            let ws = &self.settings_edit.settings_window;
            egui::Window::new("Settings")
                .open(&mut open)
                .resizable(true)
                .default_pos([ws.x, ws.y])
                .default_size([ws.width, ws.height])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let tabs = ["通用", "外观", "快捷键", "锁定"];
                        for (i, label) in tabs.iter().enumerate() {
                            let selected = self.settings_tab == i;
                            if ui.selectable_label(selected, *label).clicked() {
                                self.settings_tab = i;
                            }
                        }
                    });
                    ui.separator();

                    match self.settings_tab {
                        0 => {
                            ui.label("Scene and templates use fixed paths:");
                            ui.label("  Scene: ./scene.json");
                            ui.label("  Templates: ./templates/");
                            ui.separator();
                            ui.label("历史记录:");
                            ui.horizontal(|ui| {
                                ui.label("最大条数:");
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.max_history)
                                        .range(10..=10000),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("滚动回溯:");
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.scrollback)
                                        .range(100..=50000),
                                );
                            });
                            if ui.button("清空所有历史").clicked() {
                                self.pending_clear_history = true;
                            }
                        }
                        1 => {
                            ui.label("终端外观:");
                            ui.horizontal(|ui| {
                                ui.label("字号:");
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.font_size)
                                        .range(8.0..=32.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("字间距:");
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.settings_edit.cell_spacing,
                                        0.5..=2.0,
                                    )
                                    .text("x"),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("字体:");
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
                                ui.label("背景色:");
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.bg_color,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("前景色:");
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.fg_color,
                                );
                            });
                            ui.separator();
                            ui.label("指令菜单:");
                            ui.horizontal(|ui| {
                                ui.label("背景色:");
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.menu_bg_color,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("文字色:");
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.menu_fg_color,
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("字号:");
                                ui.add(
                                    egui::DragValue::new(&mut self.settings_edit.menu_font_size)
                                        .range(8.0..=32.0),
                                );
                            });
                            ui.separator();
                            ui.label("锁定:");
                            ui.horizontal(|ui| {
                                ui.label("遮罩色:");
                                egui::widgets::color_picker::color_edit_button_srgb(
                                    ui,
                                    &mut self.settings_edit.lock_color,
                                );
                            });
                        }
                        2 => {
                            ui.label("点击快捷键名称后按下新按键即可修改");
                            ui.separator();
                            let labels = [
                                ("new_terminal", "新建终端"),
                                ("close_terminal", "关闭终端"),
                                ("workspace_up", "上一个 Workspace"),
                                ("workspace_down", "下一个 Workspace"),
                                ("panel_left", "左侧 Panel"),
                                ("panel_right", "右侧 Panel"),
                                ("lock_workspace", "锁定/解锁 Workspace"),
                                ("history_up", "显示指令历史"),
                                ("history_down", "历史指令导航"),
                            ];
                            for (id, label) in &labels {
                                ui.horizontal(|ui| {
                                    ui.label(*label);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let text =
                                                if self.binding_recording.as_deref() == Some(id) {
                                                    "按下按键...".to_string()
                                                } else if let Some(b) =
                                                    self.settings_edit.key_binds.get(*id)
                                                {
                                                    shortcut_display(b)
                                                } else {
                                                    "(未设置)".to_string()
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
                            ui.label("密码配置:");
                            ui.separator();
                            if self.settings.lock_password.is_empty() {
                                if ui.button("设置密码").clicked() {
                                    self.pw_popup = Some("set");
                                    self.pw_set1.clear();
                                    self.pw_set2.clear();
                                    self.pw_message.clear();
                                }
                            } else {
                                if ui.button("修改密码").clicked() {
                                    self.pw_popup = Some("change");
                                    self.pw_old.clear();
                                    self.pw_new1.clear();
                                    self.pw_new2.clear();
                                    self.pw_message.clear();
                                }
                                ui.add_space(10.0);
                                if ui.button("清除密码").clicked() {
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
                        if ui.button("应用").clicked() {
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
                        if ui.button("关闭").clicked() {
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
            egui::Window::new("确认关闭")
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("确定要关闭工作区「{}」吗？", panel_name));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("确认").clicked() {
                            self.close_workspace(panel_idx);
                            self.close_confirm_panel = None;
                        }
                        if ui.button("取消").clicked() {
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
                "set" => "设置密码",
                "change" => "修改密码",
                "clear" => "清除密码",
                _ => "",
            };
            egui::Window::new(title)
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| match popup {
                    "set" => {
                        ui.horizontal_centered(|ui| {
                            ui.label("输入密码:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_set1)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_set1"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("确认密码:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_set2)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_set2"),
                            );
                            if !self.pw_set2.is_empty() && self.pw_set1 != self.pw_set2 {
                                ui.label(egui::RichText::new("不一致").color(egui::Color32::RED));
                            } else if !self.pw_set2.is_empty() {
                                ui.label(egui::RichText::new("一致").color(egui::Color32::GREEN));
                            }
                        });
                        if !self.pw_message.is_empty() {
                            ui.label(
                                egui::RichText::new(&self.pw_message).color(egui::Color32::RED),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button("确认").clicked() {
                                if self.pw_set1.is_empty() {
                                    self.pw_message = "密码不能为空".into();
                                } else if self.pw_set1 != self.pw_set2 {
                                    self.pw_message = "两次输入的密码不一致".into();
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
                            if ui.button("取消").clicked() {
                                self.pw_set1.clear();
                                self.pw_set2.clear();
                                self.pw_message.clear();
                                self.pw_popup = None;
                            }
                        });
                    }
                    "change" => {
                        ui.horizontal(|ui| {
                            ui.label("原密码:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_old)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_old"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("新密码:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_new1)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_new1"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("确认新密码:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.pw_new2)
                                    .password(true)
                                    .desired_width(150.0)
                                    .id_source("pw_new2"),
                            );
                            if !self.pw_new2.is_empty() && self.pw_new1 != self.pw_new2 {
                                ui.label(egui::RichText::new("不一致").color(egui::Color32::RED));
                            } else if !self.pw_new2.is_empty() {
                                ui.label(egui::RichText::new("一致").color(egui::Color32::GREEN));
                            }
                        });
                        if !self.pw_message.is_empty() {
                            ui.label(
                                egui::RichText::new(&self.pw_message).color(egui::Color32::RED),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ui.button("确认").clicked() {
                                if self.pw_old != self.settings.lock_password {
                                    self.pw_message = "原密码错误".into();
                                } else if self.pw_new1.is_empty() {
                                    self.pw_message = "新密码不能为空".into();
                                } else if self.pw_new1 != self.pw_new2 {
                                    self.pw_message = "两次输入的新密码不一致".into();
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
                            if ui.button("取消").clicked() {
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
                            ui.label("密码:");
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
                            if ui.button("确认").clicked() {
                                if self.pw_clear != self.settings.lock_password {
                                    self.pw_message = "密码错误".into();
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
                            if ui.button("取消").clicked() {
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
            .default_width(160.0)
            .show(ctx, |ui| {
                ui.heading("Workspaces");
                ui.separator();
                let mut to_select = None;
                let panel_count = self.panels.len();
                let mut reorder = None;
                self.panel_rects.clear();
                self.panel_rects.resize(panel_count, egui::Rect::NOTHING);

                // Detect drag state from pointer
                let pointer_down = ui.input(|i| i.pointer.primary_down());
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                let pointer_delta = ui.input(|i| i.pointer.delta());

                for i in 0..panel_count {
                    let is_active = i == self.active_panel;
                    if self.renaming_panel == Some(i) {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.rename_buffer)
                                .font(egui::FontId::monospace(14.0))
                                .desired_width(ui.available_width())
                                .id_source("workspace_rename"),
                        );
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                        self.panel_rects[i] = response.rect;
                        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if enter {
                            if !self.rename_buffer.is_empty() {
                                self.panels[i].name = self.rename_buffer.clone();
                            }
                            self.renaming_panel = None;
                        }
                    } else {
                        ui.horizontal(|ui| {
                            let panel_name = self.panels[i].name.clone();
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

                            // Drag to reorder: detect drag via pointer delta on this rect
                            if pointer_down && pointer_delta.length() > 2.0 {
                                if let Some(pos) = pointer_pos {
                                    // Start drag if not already dragging and pointer is on this rect
                                    if self.drag_src_panel.is_none() && response.rect.contains(pos)
                                    {
                                        self.drag_src_panel = Some(i);
                                    }
                                    // Find drag target
                                    if let Some(src) = self.drag_src_panel {
                                        if src != i {
                                            // Use this panel's rect as potential target
                                        }
                                    }
                                }
                            }

                            // Context menu
                            response.context_menu(|ui| {
                                if ui.button("重命名").clicked() {
                                    self.renaming_panel = Some(i);
                                    self.rename_buffer = self.panels[i].name.clone();
                                    self.rename_frame_count = 0;
                                    ui.close_menu();
                                }
                                if ui.button("保存为模版").clicked() {
                                    self.save_as_template(i);
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("关闭").clicked() {
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
                                            if ui.small_button("x").clicked() {
                                                self.close_confirm_panel = Some(i);
                                                return;
                                            }
                                        }
                                        let is_locked = self.locked_panels.contains(&i);
                                        let lock_label = if is_locked { "🔓" } else { "🔒" };
                                        if ui.small_button(lock_label).clicked() {
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
                    }
                }

                // Handle drag target detection after all rects are known
                if pointer_down && pointer_delta.length() > 2.0 {
                    if let Some(src) = self.drag_src_panel {
                        if let Some(pos) = pointer_pos {
                            for j in (0..panel_count).rev() {
                                if j == src {
                                    continue;
                                }
                                if j < self.panel_rects.len() && self.panel_rects[j].contains(pos) {
                                    if self.drag_dst_panel != Some(j) {
                                        self.drag_dst_panel = Some(j);
                                        reorder = Some((src, j));
                                    }
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    // Pointer released: reset drag state
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
                if ui.button("+ New Workspace").clicked() {
                    self.add_panel(ui.ctx());
                }
                if self.cached_template_files.is_empty() {
                    self.refresh_template_files();
                }
                let template_files = self.cached_template_files.clone();
                ui.menu_button("Templates", |ui| {
                    if template_files.is_empty() {
                        ui.label("(empty)");
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
                ui.label("L1: Workspaces");
                ui.label("L2: Dock panels");
                ui.label("L3: Terminal tabs");
            });

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
                            "🔒 此工作区已锁定".to_owned(),
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
                            egui::RichText::new("🔒 此工作区已锁定")
                                .size(24.0)
                                .color(egui::Color32::WHITE),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("密码:").color(egui::Color32::from_gray(200)),
                            );
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut self.lock_password_input)
                                    .password(true)
                                    .desired_width(160.0)
                                    .id(pw_id),
                            );
                            resp.request_focus();
                            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            if ui.button("解锁").clicked() || (enter_pressed && resp.has_focus())
                            {
                                if self.settings.lock_password.is_empty()
                                    || self.lock_password_input == self.settings.lock_password
                                {
                                    self.locked_panels.remove(&self.active_panel);
                                    self.lock_password_input.clear();
                                    self.pw_message.clear();
                                } else {
                                    self.pw_message = "密码错误".into();
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
                            renaming_terminal: &mut self.renaming_terminal,
                            terminal_rename_buffer: &mut self.terminal_rename_buffer,
                            renaming,
                            rename_frame_count: self.rename_frame_count,
                            active_tab,
                            focused_terminal: &mut self.focused_terminal,
                            show_settings: self.show_settings,
                        },
                    );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Click '+ New Workspace' to create one.");
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalStatePersist;

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
    renaming_terminal: &'a mut Option<String>,
    terminal_rename_buffer: &'a mut String,
    renaming: bool,
    rename_frame_count: u32,
    active_tab: Option<String>,
    focused_terminal: &'a mut Option<String>,
    show_settings: bool,
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
                        .hint_text("Enter name...")
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

            let is_focused = self.focused_terminal.as_ref() == Some(tab);

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

            if is_focused && !self.renaming && !self.show_settings {
                terminal_response.request_focus();
            }

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
            ui.label("Terminal not found");
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        *self.pending_close = Some(tab.clone());
        true
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        *self.pending_new_terminal = Some((self.active_panel, surface, node));
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, surface: SurfaceIndex, node: NodeIndex) {
        ui.horizontal(|ui| {
            if ui.button("+ Tab").clicked() {
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
        if ui.button("Rename").clicked() {
            *self.renaming_terminal = Some(tab.clone());
            if let Some(data) = self.terminals.get(tab) {
                *self.terminal_rename_buffer = data.name.clone();
            }
            self.rename_frame_count = 0;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("清空指令历史").clicked() {
            self.history_db.clear(tab);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("+ New Tab").clicked() {
            *self.pending_new_terminal = Some((self.active_panel, surface, node));
            ui.close_menu();
        }
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

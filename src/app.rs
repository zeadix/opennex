use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use egui_term::{PtyEvent, TerminalBackend, TerminalView, TerminalFont, FontSettings, Binding, BindingAction, InputKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 1.0;

// ── Settings ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    #[serde(default = "default_max_history")]
    max_history: usize,
    #[serde(default)]
    settings_window: SettingsWindowState,
}

fn default_max_history() -> usize {
    300
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
        SettingsWindowState { x: 200.0, y: 150.0, width: 500.0, height: 350.0 }
    }
}

fn settings_path() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("settings.json")
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
    let path = settings_path();
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            max_history: default_max_history(),
            settings_window: SettingsWindowState::default(),
        }
    }
}

fn scene_path() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("scene.json")
}

// ── Persistence structs (single panel per file) ──────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceState {
    panel_name: String,
    dock_state: DockState<String>,
    terminals: HashMap<String, TerminalState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TerminalState {
    name: String,
    font_size: f32,
    working_directory: String,
    #[serde(default)]
    snapshot: Option<crate::snapshot::state::TerminalSnapshot>,
    #[serde(default)]
    process_info: Option<crate::snapshot::state::ProcessInfo>,
}

// ── Scene (global workspace data) ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SceneState {
    panels: Vec<ScenePanel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenePanel {
    name: String,
    dock_state: DockState<String>,
    terminals: HashMap<String, TerminalState>,
}

struct Panel {
    name: String,
    bound_file: Option<PathBuf>,
}

struct HistoryNav {
    entries: Vec<String>,
    selected: usize,
}

pub struct App {
    panels: Vec<Panel>,
    active_panel: usize,
    dock_states: HashMap<usize, DockState<String>>,
    terminals: HashMap<String, TerminalData>,
    tab_counter: u32,
    pending_new_terminal: Option<(usize, SurfaceIndex, NodeIndex)>,
    pending_close: Option<String>,
    pending_split_after: Option<String>,
    pending_split_vertical: bool,
    renaming_panel: Option<usize>,
    rename_buffer: String,
    renaming_terminal: Option<String>,
    terminal_rename_buffer: String,
    rename_frame_count: u32,
    pending_load_workspace: bool,
    pending_load_from_template: Option<PathBuf>,
    pending_delete_template: Option<PathBuf>,
    pending_load_scene: bool,
    pending_save_scene_as: bool,
    pending_clear_history: bool,
    settings: AppSettings,
    show_settings: bool,
    settings_edit: AppSettings,
    cached_template_files: Vec<(String, PathBuf)>,
    completion: crate::completion::CompletionEngine,
    history_db: crate::history_db::HistoryDb,
    history_nav: Option<HistoryNav>,
}

struct TerminalData {
    backend: TerminalBackend,
    receiver: Receiver<(u64, PtyEvent)>,
    name: String,
    font_size: f32,
    working_directory: String,
    cwd_file: std::path::PathBuf,
    restored_snapshot: Option<crate::snapshot::state::TerminalSnapshot>,
}

impl App {
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
        let Some(state) = build_panel_state(self, panel_idx) else { return };
        let dir = self.templates_dir();
        let _ = std::fs::create_dir_all(&dir);
        let name = state.panel_name.replace(['/', '\\', ':'], "_");
        let path = dir.join(format!("{}.json", name));
        if let Err(e) = save_to_file(&path, &state) {
            log::error!("Failed to save template: {}", e);
        }
        self.refresh_template_files();
    }
}

// ── Save / Load ──────────────────────────────────────────────────

fn build_panel_state(app: &App, panel_idx: usize) -> Option<WorkspaceState> {
    let panel = app.panels.get(panel_idx)?;
    let dock_state = app.dock_states.get(&panel_idx)?.clone();
    let mut terminals = HashMap::new();
    for (id, data) in &app.terminals {
        terminals.insert(id.clone(), TerminalState {
            name: data.name.clone(),
            font_size: data.font_size,
            working_directory: data.working_directory.clone(),
            snapshot: None,
            process_info: None,
        });
    }
    Some(WorkspaceState {
        panel_name: panel.name.clone(),
        dock_state,
        terminals,
    })
}

fn save_to_file(path: &PathBuf, state: &WorkspaceState) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn load_from_file(path: &PathBuf) -> Result<WorkspaceState, anyhow::Error> {
    let json = std::fs::read_to_string(path)?;
    let state: WorkspaceState = serde_json::from_str(&json)?;
    Ok(state)
}

fn update_tab_id_in_dock(dock: &mut DockState<String>, old_id: &str, new_id: &str) {
    for (_surface, node) in dock.iter_all_nodes_mut() {
        if let egui_dock::Node::Leaf { tabs, .. } = node {
            for tab in tabs.iter_mut() {
                if tab == old_id {
                    *tab = new_id.to_string();
                }
            }
        }
    }
}

// ── App impl ─────────────────────────────────────────────────────

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        let settings = load_settings();

        // Try to load scene file from root
        let sp = scene_path();
        if sp.exists() {
            if let Ok(scene) = load_scene_file(&sp) {
                let mut app = App::empty();
                app.settings = settings.clone();
                app.settings_edit = settings;
                for panel in &scene.panels {
                    let idx = app.panels.len();
                 for (_id, tstate) in &panel.terminals {
                    let Some((backend, receiver)) = create_terminal(ctx, &tstate.working_directory) else { continue };
                    let cwd_file = std::path::PathBuf::from(format!("/tmp/openzoo_cwd_{}", _id));
                    app.terminals.insert(_id.clone(), TerminalData {
                            backend, receiver,
                            name: tstate.name.clone(),
                            font_size: tstate.font_size,
                            working_directory: tstate.working_directory.clone(),

                            cwd_file,
                            restored_snapshot: tstate.snapshot.clone(),
                        });
                    if let Some(n) = _id.strip_prefix("terminal-").and_then(|s| s.parse::<u32>().ok()) {
                        app.tab_counter = app.tab_counter.max(n + 1);
                    }
                    }
                    app.panels.push(Panel { name: panel.name.clone(), bound_file: None });
                    app.dock_states.insert(idx, panel.dock_state.clone());
                }
                if app.panels.is_empty() {
                    app.add_initial_terminal(ctx);
                }
                app.active_panel = 0;
                return app;
            }
        }

        // No scene file — create default
        let mut app = App::empty();
        app.settings = settings.clone();
        app.settings_edit = settings;
        app.add_initial_terminal(ctx);

        // Save default scene
        let scene = build_scene(&app);
        let _ = save_scene_file(&sp, &scene);

        app
    }

    fn empty() -> Self {
        let db_path = std::env::current_dir().unwrap_or_default().join("history.db");
        App {
            panels: Vec::new(),
            active_panel: 0,
            dock_states: HashMap::new(),
            terminals: HashMap::new(),
            tab_counter: 0,
            pending_new_terminal: None,
            pending_close: None,
            pending_split_after: None,
            pending_split_vertical: false,
            renaming_panel: None,
            rename_buffer: String::new(),
            renaming_terminal: None,
            terminal_rename_buffer: String::new(),
            rename_frame_count: 0,
            pending_load_workspace: false,
            pending_load_from_template: None,
            pending_delete_template: None,
            pending_load_scene: false,
            pending_save_scene_as: false,
            pending_clear_history: false,
            settings: AppSettings::default(),
            show_settings: false,
            settings_edit: AppSettings::default(),
            cached_template_files: Vec::new(),
            completion: crate::completion::CompletionEngine::new(),
            history_db: crate::history_db::HistoryDb::new(&db_path, default_max_history()),
            history_nav: None,
        }
    }

    fn add_initial_terminal(&mut self, ctx: &egui::Context) {
        let name = "Workspace 1".to_string();
        let Some(tab_id) = self.create_terminal_inner(ctx) else { return };
        self.dock_states.insert(0, DockState::new(vec![tab_id]));
        self.panels.push(Panel { name, bound_file: None });
        self.active_panel = 0;
    }

    fn load_workspace_state(&mut self, ctx: &egui::Context, state: WorkspaceState, file: Option<PathBuf>) {
        let panel_idx = self.panels.len();

        for (id, tstate) in &state.terminals {
            if !self.terminals.contains_key(id) {
                let Some((backend, receiver)) = create_terminal(ctx, &tstate.working_directory) else { continue };
                self.terminals.insert(id.clone(), TerminalData {
                    backend,
                    receiver,
                    name: tstate.name.clone(),
                    font_size: tstate.font_size,
                    working_directory: tstate.working_directory.clone(),
                    cwd_file: std::path::PathBuf::from(format!("/tmp/openzoo_cwd_{}", id)),
                    restored_snapshot: tstate.snapshot.clone(),
                });
            }
        }

        self.panels.push(Panel { name: state.panel_name, bound_file: file });
        self.dock_states.insert(panel_idx, state.dock_state);
    }

    fn is_renaming(&self) -> bool {
        self.renaming_panel.is_some() || self.renaming_terminal.is_some()
    }

    fn create_terminal_inner(&mut self, ctx: &egui::Context) -> Option<String> {
        self.tab_counter += 1;
        let id = format!("terminal-{}", self.tab_counter);
        let cwd = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let (backend, receiver) = create_terminal(ctx, &cwd)?;
        let random_suffix: String = uuid::Uuid::new_v4().as_bytes()[0..3]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        self.terminals.insert(id.clone(), TerminalData {
            backend, receiver,
            name: format!("Terminal {}", random_suffix),
            font_size: DEFAULT_FONT_SIZE,
            working_directory: cwd.clone(),
            cwd_file: std::path::PathBuf::from(format!("/tmp/openzoo_cwd_{}", id)),
            restored_snapshot: None,
        });
        Some(id)
    }

    fn create_terminal(&mut self, ctx: &egui::Context) -> Option<String> {
        self.create_terminal_inner(ctx)
    }

    fn process_pending(&mut self, ctx: &egui::Context) {
        if let Some((panel_idx, surface_idx, node_idx)) = self.pending_new_terminal.take() {
            let Some(tab_id) = self.create_terminal(ctx) else { return };
            if let Some(dock) = self.dock_states.get_mut(&panel_idx) {
                if let Some(ref after_tab) = self.pending_split_after.clone() {
                    if let Some((_surface, split_node_idx, _)) = dock.find_tab(after_tab) {
                        if self.pending_split_vertical {
                            dock.main_surface_mut().split_below(split_node_idx, 0.5, vec![tab_id]);
                        } else {
                            dock.main_surface_mut().split_right(split_node_idx, 0.5, vec![tab_id]);
                        }
                    }
                    self.pending_split_after = None;
                } else {
                    // 精确定位到用户点击的 surface/node
                    dock[surface_idx][node_idx].append_tab(tab_id);
                }
            } else {
                let mut dock = DockState::new(vec![]);
                dock.main_surface_mut().push_to_first_leaf(tab_id);
                self.dock_states.insert(panel_idx, dock);
            }
        }

        if let Some(tab_id) = self.pending_close.take() {
            self.terminals.remove(&tab_id);
        }
        // Handle pending load (deferred to avoid blocking UI)
        if self.pending_load_workspace {
            self.pending_load_workspace = false;
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Load Workspace")
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                if let Ok(state) = load_from_file(&path) {
                    self.load_workspace_state(ctx, state, Some(path));
                    self.active_panel = self.panels.len() - 1;
                }
            }
        }

        // Handle pending load from template (no binding)
        if let Some(path) = self.pending_load_from_template.take() {
            if let Ok(mut state) = load_from_file(&path) {
                // Add 6-digit random hex suffix to workspace name
                let random_suffix: String = uuid::Uuid::new_v4().as_bytes()[0..3]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                state.panel_name = format!("{} {}", state.panel_name, random_suffix);
                // Remap terminal IDs with new counter
                let mut new_terminals = HashMap::new();
                let mut new_dock_state = state.dock_state.clone();
                for (old_id, tstate) in &state.terminals {
                    self.tab_counter += 1;
                    let new_id = format!("terminal-{}", self.tab_counter);
                    // Update dock state references
                    update_tab_id_in_dock(&mut new_dock_state, old_id, &new_id);
                    new_terminals.insert(new_id, tstate.clone());
                }
                state.dock_state = new_dock_state;
                state.terminals = new_terminals;
                self.load_workspace_state(ctx, state, None);
                self.active_panel = self.panels.len() - 1;
            }
        }

        // Handle pending delete template
        if let Some(path) = self.pending_delete_template.take() {
            let confirmed = rfd::MessageDialog::new()
                .set_title("确认删除")
                .set_description("确定要删除这个模版吗？")
                .set_buttons(rfd::MessageButtons::OkCancel)
                .show()
                == rfd::MessageDialogResult::Ok;
            if confirmed {
                let _ = std::fs::remove_file(&path);
                self.refresh_template_files();
            }
        }

        // Handle pending clear history
        if self.pending_clear_history {
            self.pending_clear_history = false;
            let confirmed = rfd::MessageDialog::new()
                .set_title("确认清空")
                .set_description("确定要清空所有命令历史吗？")
                .set_buttons(rfd::MessageButtons::OkCancel)
                .show()
                == rfd::MessageDialogResult::Ok;
            if confirmed {
                self.history_db.clear_all();
            }
        }

        // Handle pending scene load
        if self.pending_load_scene {
            self.pending_load_scene = false;
            self.load_scene(ctx);
        }

        // Handle pending scene save as
        if self.pending_save_scene_as {
            self.pending_save_scene_as = false;
            self.save_scene_as();
        }
    }

    fn add_panel(&mut self, ctx: &egui::Context) {
        let idx = self.panels.len();
        let name = format!("Workspace {}", idx + 1);
        let Some(tab_id) = self.create_terminal(ctx) else { return };
        self.dock_states.insert(idx, DockState::new(vec![tab_id]));
        self.panels.push(Panel { name, bound_file: None });
        self.active_panel = idx;
    }

    fn save_workspace(&mut self, panel_idx: usize) {
        if let Some(state) = build_panel_state(self, panel_idx) {
            let path = if let Some(ref file) = self.panels[panel_idx].bound_file {
                file.clone()
            } else {
                match rfd::FileDialog::new()
                    .set_title("Save Workspace")
                    .add_filter("JSON", &["json"])
                    .set_file_name(&format!("{}.json", self.panels[panel_idx].name))
                    .save_file()
                {
                    Some(p) => {
                        self.panels[panel_idx].bound_file = Some(p.clone());
                        p
                    }
                    None => return,
                }
            };
            if let Err(e) = save_to_file(&path, &state) {
                log::error!("Failed to save: {}", e);
            }
        }
    }

    fn save_workspace_as(&mut self, panel_idx: usize) {
        if let Some(panel) = self.panels.get(panel_idx) {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Save Workspace As")
                .add_filter("JSON", &["json"])
                .set_file_name(&format!("{}.json", panel.name))
                .save_file()
            {
                if let Some(state) = build_panel_state(self, panel_idx) {
                    if let Err(e) = save_to_file(&path, &state) {
                        log::error!("Failed to save: {}", e);
                    } else if let Some(p) = self.panels.get_mut(panel_idx) {
                        p.bound_file = Some(path);
                    }
                }
            }
        }
    }

    fn save_scene(&mut self) {
        let sp = scene_path();
        let scene = build_scene(self);
        if let Err(e) = save_scene_file(&sp, &scene) {
            log::error!("Failed to save scene: {}", e);
        }
    }

    fn save_scene_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Scene As")
            .add_filter("JSON", &["json"])
            .set_file_name("scene.json")
            .save_file()
        {
            let scene = build_scene(self);
            let _ = save_scene_file(&path, &scene);
        }
    }

    fn load_scene(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Load Scene")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            if let Ok(scene) = load_scene_file(&path) {
                self.apply_scene(ctx, scene);
            }
        }
    }

    fn apply_scene(&mut self, ctx: &egui::Context, scene: SceneState) {
        self.panels.clear();
        self.dock_states.clear();
        self.terminals.clear();
        self.tab_counter = 0;
        self.active_panel = 0;
        for panel in &scene.panels {
            let idx = self.panels.len();
            for (_id, tstate) in &panel.terminals {
                let Some((backend, receiver)) = create_terminal(ctx, &tstate.working_directory) else { continue };
                let cwd_file = std::path::PathBuf::from(format!("/tmp/openzoo_cwd_{}", _id));
                self.terminals.insert(_id.clone(), TerminalData {
                    backend, receiver,
                    name: tstate.name.clone(),
                    font_size: tstate.font_size,
                    working_directory: tstate.working_directory.clone(),
                    cwd_file,
                    restored_snapshot: tstate.snapshot.clone(),
                });
                if let Some(n) = _id.strip_prefix("terminal-").and_then(|s| s.parse::<u32>().ok()) {
                    self.tab_counter = self.tab_counter.max(n + 1);
                }
            }
            self.panels.push(Panel { name: panel.name.clone(), bound_file: None });
            self.dock_states.insert(idx, panel.dock_state.clone());
        }
        if self.panels.is_empty() {
            self.add_initial_terminal(ctx);
        }
    }

    fn close_workspace(&mut self, idx: usize) {
        if self.panels.len() <= 1 {
            return;
        }
        // 收集该 panel 的所有终端 ID，从 terminals 中移除（释放 PTY fd）
        if let Some(dock_state) = self.dock_states.get(&idx) {
            let tab_ids: Vec<String> = dock_state.iter_all_tabs()
                .map(|((_, _), tab)| tab.clone())
                .collect();
            for id in &tab_ids {
                self.terminals.remove(id);
            }
        }
        self.panels.remove(idx);
        let old_dock = self.dock_states.clone();
        self.dock_states.clear();
        for old_idx in 0..old_dock.len() {
            if old_idx == idx { continue; }
            let new_idx = if old_idx > idx { old_idx - 1 } else { old_idx };
            if let Some(state) = old_dock.get(&old_idx) {
                self.dock_states.insert(new_idx, state.clone());
            }
        }
        if self.active_panel >= self.panels.len() {
            self.active_panel = self.panels.len().saturating_sub(1);
        }
    }
}

// ── Terminal creation ────────────────────────────────────────────

fn load_scene_file(path: &PathBuf) -> Result<SceneState, anyhow::Error> {
    let json = std::fs::read_to_string(path)?;
    let state: SceneState = serde_json::from_str(&json)?;
    Ok(state)
}

fn save_scene_file(path: &PathBuf, state: &SceneState) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn build_scene(app: &App) -> SceneState {
    let mut panels = Vec::new();
    for (i, panel) in app.panels.iter().enumerate() {
        if let Some(dock_state) = app.dock_states.get(&i) {
            let mut terminals = HashMap::new();
            for (id, data) in &app.terminals {
                let cwd = std::fs::read_to_string(&data.cwd_file)
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| data.working_directory.clone());

                terminals.insert(id.clone(), TerminalState {
                    name: data.name.clone(),
                    font_size: data.font_size,
                    working_directory: cwd,
                    snapshot: None,
                    process_info: None,
                });
            }
            panels.push(ScenePanel {
                name: panel.name.clone(),
                dock_state: dock_state.clone(),
                terminals,
            });
        }
    }
    SceneState { panels }
}

fn create_terminal(ctx: &egui::Context, working_dir: &str) -> Option<(TerminalBackend, Receiver<(u64, PtyEvent)>)> {
    #[cfg(unix)]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    #[cfg(not(unix))]
    let shell = "cmd.exe".to_string();

    let cwd = std::path::PathBuf::from(working_dir);
    let cwd_str = if cwd.exists() { working_dir.to_string() } else { std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default() };

    // Use random u64 for unique widget ID
    let uuid_bytes = uuid::Uuid::new_v4();
    let backend_id = u64::from_be_bytes(uuid_bytes.as_bytes()[0..8].try_into().unwrap());

    let (sender, receiver) = std::sync::mpsc::channel();
    let backend = TerminalBackend::new(backend_id, ctx.clone(), sender, egui_term::BackendSettings {
        shell,
        working_directory: Some(std::path::PathBuf::from(&cwd_str)),
        ..Default::default()
    }).ok()?;

    Some((backend, receiver))
}

// ── eframe::App ──────────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending(ctx);
        let renaming = self.is_renaming();
        if renaming { self.rename_frame_count += 1; }

        // Menu bar
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
                        // Also set pending_new_terminal with current active surface/node
                        if let Some((surface, node, _)) = tree.find_tab(tab) {
                            self.pending_new_terminal = Some((self.active_panel, surface, node));
                        }
                        ui.close_menu();
                    }
                    if ui.button("Split Down").clicked() {
                        self.pending_split_after = Some(tab.clone());
                        self.pending_split_vertical = true;
                        if let Some((surface, node, _)) = tree.find_tab(tab) {
                            self.pending_new_terminal = Some((self.active_panel, surface, node));
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

        // Settings window
        if self.show_settings {
            let mut open = self.show_settings;
            let ws_x = self.settings_edit.settings_window.x;
            let ws_y = self.settings_edit.settings_window.y;
            let ws_w = self.settings_edit.settings_window.width;
            let ws_h = self.settings_edit.settings_window.height;

            let resp = egui::Window::new("Settings")
                .open(&mut open)
                .resizable(true)
                .default_pos([ws_x, ws_y])
                .default_size([ws_w, ws_h])
                .max_width(600.0)
                .show(ctx, |ui| {
                    ui.heading("Settings");
                    ui.separator();
                    ui.label("Scene and templates use fixed paths:");
                    ui.label("  Scene: ./scene.json");
                    ui.label("  Templates: ./templates/");

                    ui.separator();
                    ui.label("History:");
                    ui.horizontal(|ui| {
                        ui.label("Max entries:");
                        ui.add(egui::DragValue::new(&mut self.settings_edit.max_history)
                            .range(10..=10000));
                    });
                    if ui.button("Clear All History").clicked() {
                        self.pending_clear_history = true;
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.settings = self.settings_edit.clone();
                            self.history_db.set_max_entries(self.settings.max_history);
                            let _ = save_settings(&self.settings);
                        }
                        if ui.button("Cancel").clicked() {
                            self.settings_edit = self.settings.clone();
                        }
                    });
                });

            // Save window position and size
            if let Some(inner) = resp {
                let rect = inner.response.rect;
                self.settings_edit.settings_window.x = rect.min.x;
                self.settings_edit.settings_window.y = rect.min.y;
                self.settings_edit.settings_window.width = rect.width();
                self.settings_edit.settings_window.height = rect.height();
            }

            self.show_settings = open;
            if !open {
                self.settings.settings_window = self.settings_edit.settings_window.clone();
                let _ = save_settings(&self.settings);
            }
        }

        // Left panel
        egui::SidePanel::left("navigation").default_width(160.0).show(ctx, |ui| {
            ui.heading("Workspaces");
            ui.separator();

            let mut to_select = None;
            let panel_count = self.panels.len();
            for i in 0..panel_count {
                let is_active = i == self.active_panel;

                if self.renaming_panel == Some(i) {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.rename_buffer)
                            .font(egui::FontId::monospace(14.0))
                            .desired_width(ui.available_width()),
                    );
                    if !response.has_focus() { response.request_focus(); }

                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let can_exit = self.rename_frame_count > 1;
                    let pointer = ui.input(|i| i.pointer.clone());
                    let clicked_outside = can_exit && pointer.any_click()
                        && !response.rect.contains(pointer.interact_pos().unwrap_or_default());

                    if enter || clicked_outside {
                        if !self.rename_buffer.is_empty() {
                            self.panels[i].name = self.rename_buffer.clone();
                        }
                        self.renaming_panel = None;
                    }
                } else {
                    ui.horizontal(|ui| {
                        let panel_name = self.panels[i].name.clone();
                        let response = ui.selectable_label(is_active, &panel_name);
                        if response.clicked() && !renaming { to_select = Some(i); }
                        if response.double_clicked() && !renaming {
                            self.renaming_panel = Some(i);
                            self.rename_buffer = panel_name;
                            self.rename_frame_count = 0;
                        }

                        response.context_menu(|ui| {
                            if ui.button("保存为模版").clicked() {
                                self.save_as_template(i);
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("关闭").clicked() {
                                self.close_workspace(i);
                                ui.close_menu();
                            }
                        });

                        if self.panels.len() > 1 {
                            if ui.small_button("x").clicked() {
                                self.close_workspace(i);
                                return;
                            }
                        }
                    });
                }
            }

            if let Some(i) = to_select { self.active_panel = i; }

            ui.separator();
            if ui.button("+ New Workspace").clicked() {
                self.add_panel(ui.ctx());
            }

            // Template button — show cached workspace files
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

        // Right panel
        let active_tab = self.dock_states.get_mut(&self.active_panel)
            .and_then(|t| t.find_active_focused().map(|(_, t)| t.clone()));
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                DockArea::new(tree)
                    .style(Style::from_egui(ui.style().as_ref()))
                    .show_add_buttons(true)
                    .show_add_popup(false)
                    .show_inside(ui, &mut TerminalTabViewer {
                        terminals: &mut self.terminals,
                        completion: &self.completion,
                        history_db: &self.history_db,
                        history_nav: &mut self.history_nav,
                        max_history: self.settings.max_history,
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
                        last_tab_time: None,
                    });
            } else {
                ui.centered_and_justified(|ui| { ui.label("Click '+ New Workspace' to create one."); });
            }
        });
    }
}

// ── TerminalTabViewer ────────────────────────────────────────────

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    completion: &'a crate::completion::CompletionEngine,
    history_db: &'a crate::history_db::HistoryDb,
    history_nav: &'a mut Option<HistoryNav>,
    max_history: usize,
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
    last_tab_time: Option<std::time::Instant>,
}

impl<'a> egui_dock::TabViewer for TerminalTabViewer<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.terminals.get(tab).map(|d| d.name.clone().into()).unwrap_or_else(|| tab.clone().into())
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // Inline rename
        if self.renaming_terminal.as_ref() == Some(tab) {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(self.terminal_rename_buffer)
                        .font(egui::FontId::monospace(14.0))
                        .desired_width(200.0)
                        .hint_text("Enter name..."),
                );
                if !response.has_focus() { response.request_focus(); }

                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let can_exit = self.rename_frame_count > 1;
                let pointer = ui.input(|i| i.pointer.clone());
                let clicked_outside = can_exit && pointer.any_click()
                    && !response.rect.contains(pointer.interact_pos().unwrap_or_default());

                if enter || clicked_outside {
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

        // Terminal
        if let Some(terminal_data) = self.terminals.get_mut(tab) {
            if let Ok((_, PtyEvent::Exit)) = terminal_data.receiver.try_recv() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }



            // Check if there's a restored snapshot to display
            let has_snapshot = terminal_data.restored_snapshot.is_some();
            if has_snapshot {
                // Show snapshot as overlay until user interacts
                let snapshot = terminal_data.restored_snapshot.as_ref().unwrap();
                let font = TerminalFont::new(FontSettings {
                    font_type: egui::FontId::monospace(terminal_data.font_size),
                });
                
                // Render snapshot grid
                let size = ui.available_size();
                let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
                let rect = response.rect;
                
                if let Some(first_row) = snapshot.grid.first() {
                    let cell_w = size.x / first_row.len() as f32;
                    let cell_h = size.y / snapshot.grid.len() as f32;
                    
                    // Draw background
                    painter.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::BLACK);
                    
                    // Draw cells
                    for (row_idx, row) in snapshot.grid.iter().enumerate() {
                        for (col_idx, cell) in row.iter().enumerate() {
                            let x = rect.min.x + col_idx as f32 * cell_w;
                            let y = rect.min.y + row_idx as f32 * cell_h;
                            
                            let fg = egui::Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2]);
                            let bg = egui::Color32::from_rgb(cell.bg[0], cell.bg[1], cell.bg[2]);
                            
                            if bg != egui::Color32::BLACK {
                                painter.rect_filled(
                                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_w, cell_h)),
                                    egui::CornerRadius::ZERO,
                                    bg,
                                );
                            }
                            
                            let text = egui::RichText::new(cell.ch.to_string())
                                .color(fg)
                                .font(font.font_type());
                            ui.painter().text(
                                egui::pos2(x, y),
                                egui::Align2::LEFT_TOP,
                                text.text(),
                                font.font_type(),
                                fg,
                            );
                        }
                    }
                    
                    // Draw cursor
                    let (cx, cy) = snapshot.cursor;
                    let cursor_x = rect.min.x + cx as f32 * cell_w;
                    let cursor_y = rect.min.y + cy as f32 * cell_h;
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(cursor_x, cursor_y), egui::vec2(cell_w, cell_h)),
                        egui::CornerRadius::ZERO,
                        egui::Color32::WHITE,
                    );
                }
                
                // Clear snapshot on any interaction
                if response.clicked() || response.hovered() {
                    terminal_data.restored_snapshot = None;
                }
                
                // Show hint
                ui.label("Click to restore terminal");
            } else {
                // Normal terminal rendering
                // Ctrl+scroll zoom (mouse over this terminal only)
                let mouse_over = ui.rect_contains_pointer(ui.clip_rect());
                if mouse_over {
                    let scroll: f32 = ui.input(|i| i.events.iter().filter_map(|e| {
                        if let egui::Event::MouseWheel { delta, modifiers, .. } = e {
                            if modifiers.ctrl { Some(delta.y) } else { None }
                        } else { None }
                    }).sum());
                    if scroll > 0.0 {
                        terminal_data.font_size = (terminal_data.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                    } else if scroll < 0.0 {
                        terminal_data.font_size = (terminal_data.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                    }
                }

                // Ctrl+/- keyboard zoom (active tab only)
                if self.active_tab.as_ref() == Some(tab) {
                    if ui.input(|i| i.key_pressed(egui::Key::Equals) && i.modifiers.ctrl) {
                        terminal_data.font_size = (terminal_data.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
                        terminal_data.font_size = (terminal_data.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                    }
                }

                let font = TerminalFont::new(FontSettings {
                    font_type: egui::FontId::monospace(terminal_data.font_size),
                });

                // Override Tab to Ignore so we can intercept it
                let tab_override = vec![(
                    Binding {
                        target: InputKind::KeyCode(egui::Key::Tab),
                        modifiers: egui::Modifiers::NONE,
                        terminal_mode_include: alacritty_terminal::term::TermMode::empty(),
                        terminal_mode_exclude: alacritty_terminal::term::TermMode::empty(),
                    },
                    BindingAction::Ignore,
                )];

                let terminal_view = TerminalView::new(ui, &mut terminal_data.backend)
                    .set_focus(!self.renaming)
                    .set_font(font.clone())
                    .set_size(ui.available_rect_before_wrap().size())
                    .add_bindings(tab_override);
                let terminal_response = ui.add(terminal_view);

                // Ghost text: render gray suggestion after cursor (single line, no wrap)
                {
                    let content = terminal_data.backend.sync();
                    let cursor_line = content.grid.cursor.point.line.0 as usize;
                    let cursor_col = content.grid.cursor.point.column.0 as usize;
                    let mut input_line = String::new();
                    for indexed in content.grid.display_iter() {
                        if indexed.point.line.0 as usize == cursor_line {
                            input_line.push(indexed.c);
                        }
                    }
                    let prompt_end = input_line.rfind("$ ").or_else(|| input_line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                    if cursor_col > prompt_end && cursor_col <= input_line.len() {
                        let input = input_line[prompt_end..cursor_col].trim();
                        if !input.is_empty() {
                            if let Some(best) = self.completion.suggest(input).first() {
                                if best.len() > input.len() {
                                    let remaining = &best[input.len()..];
                                    let term_rect = terminal_response.rect;
                                    let cell_w = content.terminal_size.cell_width as f32;
                                    let cell_h = content.terminal_size.cell_height as f32;
                                    let cursor_pos = egui::pos2(
                                        term_rect.min.x + cursor_col as f32 * cell_w,
                                        term_rect.min.y + cursor_line as f32 * cell_h,
                                    );
                                    ui.painter().text(
                                        cursor_pos,
                                        egui::Align2::LEFT_CENTER,
                                        remaining,
                                        egui::FontId::monospace(terminal_data.font_size),
                                        egui::Color32::from_rgba_premultiplied(120, 120, 120, 100),
                                    );
                                }
                            }
                        }
                    }
                }

                // Handle Tab key: single tab fills suggestion, double tab sends native shell tab
                if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                    let now = std::time::Instant::now();
                    let is_double_tab = self.last_tab_time
                        .map(|t| now.duration_since(t).as_millis() < 500)
                        .unwrap_or(false);
                    self.last_tab_time = Some(now);

                    if is_double_tab {
                        // Double tab: send native shell tab completion
                        terminal_data.backend.process_command(
                            egui_term::BackendCommand::Write([0x09].to_vec())
                        );
                    } else {
                        // Single tab: fill our completion suggestion
                        let content = terminal_data.backend.last_content();
                        let cursor_line = content.grid.cursor.point.line.0 as usize;
                        let cursor_col = content.grid.cursor.point.column.0 as usize;
                        let mut input_line = String::new();
                        for indexed in content.grid.display_iter() {
                            if indexed.point.line.0 as usize == cursor_line {
                                input_line.push(indexed.c);
                            }
                        }
                    let prompt_end = input_line.rfind("$ ").or_else(|| input_line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                    if cursor_col > prompt_end && cursor_col <= input_line.len() {
                        let input = input_line[prompt_end..cursor_col].trim();
                        if !input.is_empty() {
                            if let Some(best) = self.completion.suggest(input).first() {
                                if best.len() > input.len() {
                                    let remaining = &best[input.len()..];
                                        terminal_data.backend.process_command(
                                            egui_term::BackendCommand::Write(remaining.as_bytes().to_vec())
                                        );
                                    }
                                } else {
                                    terminal_data.backend.process_command(
                                        egui_term::BackendCommand::Write([0x09].to_vec())
                                    );
                                }
                            } else {
                                terminal_data.backend.process_command(
                                    egui_term::BackendCommand::Write([0x09].to_vec())
                                );
                            }
                        }
                    }
                }

                // Handle Right Arrow key: fill completion suggestion
                if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                    let content = terminal_data.backend.last_content();
                    let cursor_line = content.grid.cursor.point.line.0 as usize;
                    let cursor_col = content.grid.cursor.point.column.0 as usize;
                    let mut input_line = String::new();
                    for indexed in content.grid.display_iter() {
                        if indexed.point.line.0 as usize == cursor_line {
                            input_line.push(indexed.c);
                        }
                    }
                    let prompt_end = input_line.rfind("$ ").or_else(|| input_line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                    if cursor_col > prompt_end && cursor_col <= input_line.len() {
                        let input = input_line[prompt_end..cursor_col].trim();
                        if !input.is_empty() {
                            if let Some(best) = self.completion.suggest(input).first() {
                                if best.len() > input.len() {
                                    let remaining = &best[input.len()..];
                                    terminal_data.backend.process_command(
                                        egui_term::BackendCommand::Write(remaining.as_bytes().to_vec())
                                    );
                                }
                            }
                        }
                    }
                }

                // Handle Enter key: record command to history
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let content = terminal_data.backend.last_content();
                    let cursor_line = content.grid.cursor.point.line.0 as usize;
                    let cursor_col = content.grid.cursor.point.column.0 as usize;
                    let mut input_line = String::new();
                    for indexed in content.grid.display_iter() {
                        if indexed.point.line.0 as usize == cursor_line {
                            input_line.push(indexed.c);
                        }
                    }
                    let prompt_end = input_line.rfind("$ ").or_else(|| input_line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                    if cursor_col > prompt_end && cursor_col <= input_line.len() {
                        let cmd = input_line[prompt_end..cursor_col].trim().to_string();
                        if !cmd.is_empty() {
                            self.history_db.add(tab, &cmd);
                        }
                    }
                }

                // Handle Up Arrow key: show history navigation
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                    if self.history_nav.is_none() {
                        let entries = self.history_db.get(tab, self.max_history);
                        if !entries.is_empty() {
                            *self.history_nav = Some(HistoryNav {
                                entries,
                                selected: 0,
                            });
                        }
                    }
                }

                // Handle Down Arrow key: navigate history
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                    if let Some(ref mut nav) = *self.history_nav {
                        if nav.selected + 1 < nav.entries.len() {
                            nav.selected += 1;
                        }
                    }
                }

                // Handle Escape: close history nav
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *self.history_nav = None;
                }

                // Render history navigation list
                if let Some(ref nav) = *self.history_nav {
                    let content = terminal_data.backend.sync();
                    let term_rect = terminal_response.rect;
                    let cell_h = content.terminal_size.cell_height as f32;
                    let cursor_line = content.grid.cursor.point.line.0 as usize;
                    let list_top = term_rect.min.y + (cursor_line as f32 + 1.0) * cell_h;
                    let list_width = 400.0;
                    let max_visible = 10;
                    let visible_count = nav.entries.len().min(max_visible);
                    let list_height = visible_count as f32 * cell_h;

                    let list_rect = egui::Rect::from_min_size(
                        egui::pos2(term_rect.min.x, list_top),
                        egui::vec2(list_width, list_height),
                    );

                    ui.painter().rect_filled(
                        list_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(30, 30, 30, 240),
                    );

                    let start_idx = if nav.selected >= max_visible {
                        nav.selected - max_visible + 1
                    } else {
                        0
                    };

                    for (i, entry) in nav.entries[start_idx..].iter().enumerate().take(max_visible) {
                        let y = list_top + i as f32 * cell_h;
                        let is_selected = start_idx + i == nav.selected;
                        let bg = if is_selected {
                            egui::Color32::from_rgba_unmultiplied(60, 60, 80, 255)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(30, 30, 30, 240)
                        };
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(egui::pos2(term_rect.min.x, y), egui::vec2(list_width, cell_h)),
                            0.0,
                            bg,
                        );
                        ui.painter().text(
                            egui::pos2(term_rect.min.x + 4.0, y + cell_h * 0.5),
                            egui::Align2::LEFT_CENTER,
                            entry,
                            egui::FontId::monospace(terminal_data.font_size),
                            egui::Color32::WHITE,
                        );
                    }

                    // Handle Enter to confirm selection
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(selected) = self.history_nav.as_ref().map(|n| n.entries[n.selected].clone()) {
                            terminal_data.backend.process_command(
                                egui_term::BackendCommand::Write(selected.as_bytes().to_vec())
                            );
                            *self.history_nav = None;
                        }
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
            if ui.button("Split H").clicked() {
                *self.pending_split_vertical = false;
                *self.pending_new_terminal = Some((self.active_panel, surface, node));
                ui.close_menu();
            }
            if ui.button("Split V").clicked() {
                *self.pending_split_vertical = true;
                *self.pending_new_terminal = Some((self.active_panel, surface, node));
                ui.close_menu();
            }
        });
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab, surface: SurfaceIndex, node: NodeIndex) {
        if ui.button("Rename").clicked() {
            *self.renaming_terminal = Some(tab.clone());
            if let Some(data) = self.terminals.get(tab) {
                *self.terminal_rename_buffer = data.name.clone();
            }
            self.rename_frame_count = 0;
            ui.close_menu();
        }
        ui.separator();
        if ui.button("+ New Tab").clicked() {
            *self.pending_new_terminal = Some((self.active_panel, surface, node));
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Split Horizontal (Right)").clicked() {
            *self.pending_split_after = Some(tab.clone());
            *self.pending_split_vertical = false;
            *self.pending_new_terminal = Some((self.active_panel, surface, node));
            ui.close_menu();
        }
        if ui.button("Split Vertical (Down)").clicked() {
            *self.pending_split_after = Some(tab.clone());
            *self.pending_split_vertical = true;
            *self.pending_new_terminal = Some((self.active_panel, surface, node));
            ui.close_menu();
        }
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::terminal::{render_snapshot, render_terminal, TerminalInstance};

const DEFAULT_FONT_SIZE: f32 = 14.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const FONT_SIZE_STEP: f32 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    #[serde(default = "default_max_history")]
    max_history: usize,
    #[serde(default)]
    settings_window: SettingsWindowState,
}

fn default_max_history() -> usize { 300 }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsWindowState {
    x: f32, y: f32, width: f32, height: f32,
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
            if let Ok(settings) = serde_json::from_str(&content) { return settings; }
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
        AppSettings { max_history: default_max_history(), settings_window: SettingsWindowState::default() }
    }
}

fn scene_path() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("scene.json")
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
    #[serde(default)]
    snapshot: Option<crate::snapshot::state::TerminalSnapshot>,
    #[serde(default)]
    process_info: Option<crate::snapshot::state::ProcessInfo>,
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
    cached_template_files: Vec<(String, PathBuf)>,
    completion: crate::completion::CompletionEngine,
    history_db: crate::history_db::HistoryDb,
    focused_terminal: Option<String>,
}

struct TerminalData {
    instance: TerminalInstance,
    name: String,
    font_size: f32,
    working_directory: String,
    cwd_file: PathBuf,
    restored_snapshot: Option<crate::snapshot::state::TerminalSnapshot>,
}

fn create_terminal(ctx: &egui::Context, working_dir: &str) -> Option<TerminalInstance> {
    #[cfg(unix)]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    #[cfg(not(unix))]
    let shell = "cmd.exe".to_string();

    let cwd_str = if std::path::PathBuf::from(working_dir).exists() {
        working_dir.to_string()
    } else {
        std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    };

    let screen = ctx.screen_rect();
    let cols = (((screen.width() - 190.0) / 8.0).round().max(20.0) as u16).min(300);
    let rows = (((screen.height() - 70.0) / 18.0).round().max(5.0) as u16).min(100);

    TerminalInstance::create(&shell, &cwd_str, cols, rows)
}

fn build_panel_state(app: &App, panel_idx: usize) -> Option<WorkspaceState> {
    let panel = app.panels.get(panel_idx)?;
    let dock_state = app.dock_states.get(&panel_idx)?.clone();
    let mut terminals = HashMap::new();
    for (id, data) in &app.terminals {
        let snapshot = {
            let t = &data.instance.terminal;
            let mut term = t.lock().unwrap();
            if data.instance.screen_rows > 0 {
                Some(crate::snapshot::take_snapshot(
                    &mut *term,
                    &data.working_directory,
                    data.instance.screen_cols,
                    data.instance.screen_rows,
                ))
            } else {
                None
            }
        };
        terminals.insert(id.clone(), TerminalStatePersist {
            name: data.name.clone(),
            font_size: data.font_size,
            working_directory: data.working_directory.clone(),
            snapshot,
            process_info: None,
        });
    }
    Some(WorkspaceState { panel_name: panel.name.clone(), dock_state, terminals })
}

fn save_to_file<T: Serialize>(path: &PathBuf, data: &T) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn load_scene_file(path: &PathBuf) -> Option<SceneState> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn build_scene_state(app: &App) -> SceneState {
    let mut panels = Vec::new();
    for (panel_idx, panel) in app.panels.iter().enumerate() {
        let dock_state = app.dock_states.get(&panel_idx).cloned().unwrap_or_else(|| DockState::new(vec![]));
        let mut terminals = HashMap::new();
for (id, data) in &app.terminals {
            let snapshot = {
                let t = &data.instance.terminal;
                let mut term = t.lock().unwrap();
                if data.instance.screen_rows > 0 {
                    Some(crate::snapshot::take_snapshot(
                        &mut *term,
                        &data.working_directory,
                        data.instance.screen_cols,
                        data.instance.screen_rows,
                    ))
                } else {
                    None
                }
            };
            terminals.insert(id.clone(), TerminalStatePersist {
                name: data.name.clone(),
                font_size: data.font_size,
                working_directory: data.working_directory.clone(),
                snapshot,
                process_info: None,
            });
        }
        panels.push(ScenePanel { name: panel.name.clone(), dock_state, terminals });
    }
    SceneState { panels }
}

fn save_scene(path: &PathBuf, app: &App) {
    let state = build_scene_state(app);
    if let Err(e) = save_to_file(path, &state) {
        log::error!("Failed to save scene: {}", e);
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let settings = load_settings();
        let ctx = &cc.egui_ctx.clone();
        let db_path = std::env::current_dir().unwrap_or_default().join("history.db");

        let mut app = App {
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
            pending_load_workspace: None,
            pending_load_from_template: None,
            pending_delete_template: None,
            pending_load_scene: false,
            pending_save_scene_as: false,
            pending_clear_history: false,
            settings,
            show_settings: false,
            settings_edit: AppSettings::default(),
            cached_template_files: Vec::new(),
            completion: crate::completion::CompletionEngine::new(),
            history_db: crate::history_db::HistoryDb::new(&db_path, default_max_history()),
            focused_terminal: None,
        };

        let scene_path = scene_path();
        if scene_path.exists() {
            if let Some(scene) = load_scene_file(&scene_path) {
                app.settings_edit = app.settings.clone();
                for panel in &scene.panels {
                    let idx = app.panels.len();
                    for (_id, tstate) in &panel.terminals {
                        let Some(instance) = create_terminal(ctx, &tstate.working_directory) else { continue };
                        let cwd_file = PathBuf::from(format!("/tmp/openzoo_cwd_{}", _id));
                        app.terminals.insert(_id.clone(), TerminalData {
                            instance,
                            name: tstate.name.clone(),
                            font_size: tstate.font_size,
                            working_directory: tstate.working_directory.clone(),
                            cwd_file,
                            restored_snapshot: tstate.snapshot.clone(),
                        });
                        if let Some(n) = _id.strip_prefix("terminal-").and_then(|s| s.parse::<u32>().ok()) {
                            if n > app.tab_counter { app.tab_counter = n; }
                        }
                    }
                    app.panels.push(Panel { name: panel.name.clone(), bound_file: None });
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
        std::env::current_dir().unwrap_or_default().join("templates")
    }

    fn refresh_template_files(&mut self) {
        let dir = self.templates_dir();
        let _ = std::fs::create_dir_all(&dir);
        self.cached_template_files = std::fs::read_dir(&dir).into_iter()
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

    fn save_workspace(&mut self, path: PathBuf) {
        if self.panels.is_empty() { return; }
        let Some(state) = build_panel_state(self, self.active_panel) else { return };
        if let Err(e) = save_to_file(&path, &state) {
            log::error!("Failed to save workspace: {}", e);
        }
    }

    fn load_workspace_file(&mut self, ctx: &egui::Context, path: PathBuf) {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => { log::error!("Failed to read workspace: {}", e); return; }
        };
        let state: WorkspaceState = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(e) => { log::error!("Failed to parse workspace: {}", e); return; }
        };
        self.load_workspace_state(ctx, state, Some(path));
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
                let Some(instance) = create_terminal(ctx, &tstate.working_directory) else { continue };
                self.terminals.insert(id.clone(), TerminalData {
                    instance,
                    name: tstate.name.clone(),
                    font_size: tstate.font_size,
                    working_directory: tstate.working_directory.clone(),
                    cwd_file: PathBuf::from(format!("/tmp/openzoo_cwd_{}", id)),
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
        let instance = create_terminal(ctx, &cwd)?;
        let random_suffix: String = uuid::Uuid::new_v4().as_bytes()[0..3]
            .iter().map(|b| format!("{:02x}", b)).collect();
        self.terminals.insert(id.clone(), TerminalData {
            instance,
            name: format!("Terminal {}", random_suffix),
            font_size: DEFAULT_FONT_SIZE,
            working_directory: cwd.clone(),
            cwd_file: PathBuf::from(format!("/tmp/openzoo_cwd_{}", id)),
            restored_snapshot: None,
        });
        if self.focused_terminal.is_none() {
            self.focused_terminal = Some(id.clone());
        }
        Some(id)
    }

    fn process_pending(&mut self, ctx: &egui::Context) {
        if let Some((panel_idx, surface_idx, node_idx)) = self.pending_new_terminal.take() {
            let _split_after = self.pending_split_after.take();
            let Some(tab_id) = self.create_terminal_inner(ctx) else { return };
            if let Some(tree) = self.dock_states.get_mut(&panel_idx) {
                tree.set_focused_node_and_surface((surface_idx, node_idx));
                tree.push_to_focused_leaf(tab_id);
            }
        }
        if let Some(tab) = self.pending_close.take() {
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
                            if let Some(instance) = create_terminal(ctx, &tstate.working_directory) {
                                self.terminals.insert(_id.clone(), TerminalData {
                                    instance,
                                    name: tstate.name.clone(),
                                    font_size: tstate.font_size,
                                    working_directory: tstate.working_directory.clone(),
                                    cwd_file: PathBuf::from(format!("/tmp/openzoo_cwd_{}", _id)),
                                    restored_snapshot: tstate.snapshot.clone(),
                                });
                            }
                        }
                        self.panels.push(Panel { name: panel.name.clone(), bound_file: None });
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
        let Some(tab_id) = self.create_terminal_inner(ctx) else { return };
        self.dock_states.insert(self.panels.len(), DockState::new(vec![tab_id]));
        self.panels.push(Panel { name, bound_file: None });
    }

    fn close_workspace(&mut self, i: usize) {
        if self.panels.len() <= 1 { return; }
        let panel = self.panels.swap_remove(i);
        let _ = panel;
        self.dock_states.remove(&i);
        if self.active_panel >= self.panels.len() {
            self.active_panel = self.panels.len().saturating_sub(1);
        }
    }

    fn save_scene(&self) {
        let path = scene_path();
        save_scene(&path, self);
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending(ctx);
        let renaming = self.is_renaming();
        if renaming { self.rename_frame_count += 1; }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() { self.save_scene(); ui.close_menu(); }
                    if ui.button("Load").clicked() { self.pending_load_scene = true; ui.close_menu(); }
                    if ui.button("Save As...").clicked() { self.pending_save_scene_as = true; ui.close_menu(); }
                    ui.separator();
                    if ui.button("Exit").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
                ui.menu_button("View", |ui| {
                    if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
                        let active_tab = tree.find_active_focused().map(|(_, t)| t.clone());
                        if let Some(ref tab) = active_tab {
                            if ui.button("Split Right").clicked() {
                                self.pending_split_after = Some(tab.clone());
                                self.pending_split_vertical = false;
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
                if ui.button("Settings").clicked() { self.show_settings = true; self.settings_edit = self.settings.clone(); }
            });
        });

        if self.show_settings {
            let mut open = self.show_settings;
            let ws = &self.settings_edit.settings_window;
            egui::Window::new("Settings").open(&mut open).resizable(true)
                .default_pos([ws.x, ws.y]).default_size([ws.width, ws.height]).max_width(600.0)
                .show(ctx, |ui| {
                    ui.heading("Settings"); ui.separator();
                    ui.label("Scene and templates use fixed paths:");
                    ui.label("  Scene: ./scene.json");
                    ui.label("  Templates: ./templates/");
                    ui.separator();
                    ui.label("History:");
                    ui.horizontal(|ui| {
                        ui.label("Max entries:");
                        ui.add(egui::DragValue::new(&mut self.settings_edit.max_history).range(10..=10000));
                    });
                    if ui.button("Clear All History").clicked() { self.pending_clear_history = true; }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.settings = self.settings_edit.clone();
                            self.history_db.set_max_entries(self.settings.max_history);
                            let _ = save_settings(&self.settings);
                        }
                        if ui.button("Cancel").clicked() { self.settings_edit = self.settings.clone(); }
                    });
                });
            self.show_settings = open;
            if !open {
                self.settings.settings_window = self.settings_edit.settings_window.clone();
                let _ = save_settings(&self.settings);
            }
        }

        egui::SidePanel::left("navigation").default_width(160.0).show(ctx, |ui| {
            ui.heading("Workspaces");
            ui.separator();
            let mut to_select = None;
            let panel_count = self.panels.len();
            for i in 0..panel_count {
                let is_active = i == self.active_panel;
                if self.renaming_panel == Some(i) {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.rename_buffer)
                        .font(egui::FontId::monospace(14.0)).desired_width(ui.available_width()));
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
                            if ui.button("保存为模版").clicked() { self.save_as_template(i); ui.close_menu(); }
                            ui.separator();
                            if ui.button("关闭").clicked() { self.close_workspace(i); ui.close_menu(); }
                        });
                        if self.panels.len() > 1 {
                            if ui.small_button("x").clicked() { self.close_workspace(i); return; }
                        }
                    });
                }
            }
            if let Some(i) = to_select { self.active_panel = i; }
            ui.separator();
            if ui.button("+ New Workspace").clicked() { self.add_panel(ui.ctx()); }
            if self.cached_template_files.is_empty() { self.refresh_template_files(); }
            let template_files = self.cached_template_files.clone();
            ui.menu_button("Templates", |ui| {
                if template_files.is_empty() { ui.label("(empty)"); }
                else {
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
                        focused_terminal: &mut self.focused_terminal,
                    });
            } else {
                ui.centered_and_justified(|ui| { ui.label("Click '+ New Workspace' to create one."); });
            }
        });
    }
}

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    completion: &'a crate::completion::CompletionEngine,
    history_db: &'a crate::history_db::HistoryDb,
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
    focused_terminal: &'a mut Option<String>,
}

impl<'a> egui_dock::TabViewer for TerminalTabViewer<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.terminals.get(tab).map(|d| d.name.clone().into()).unwrap_or_else(|| tab.clone().into())
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if self.renaming_terminal.as_ref() == Some(tab) {
            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(self.terminal_rename_buffer)
                        .font(egui::FontId::monospace(14.0)).desired_width(200.0).hint_text("Enter name..."),
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

        if let Some(td) = self.terminals.get_mut(tab) {
            let has_snapshot = td.restored_snapshot.is_some();
            if has_snapshot {
                let snapshot = td.restored_snapshot.as_ref().unwrap();
                let response = render_snapshot(ui, snapshot, td.font_size);
                if response.clicked() || response.hovered() {
                    td.restored_snapshot = None;
                }
                ui.label("Click to restore terminal");
            } else {
                let mouse_over = ui.rect_contains_pointer(ui.clip_rect());
                if mouse_over {
                    let scroll: f32 = ui.input(|i| i.events.iter().filter_map(|e| {
                        if let egui::Event::MouseWheel { delta, modifiers, .. } = e {
                            if modifiers.ctrl { Some(delta.y) } else { None }
                        } else { None }
                    }).sum());
                    if scroll > 0.0 { td.font_size = (td.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE); }
                    else if scroll < 0.0 { td.font_size = (td.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE); }
                }
                if self.active_tab.as_ref() == Some(tab) {
                    if ui.input(|i| i.key_pressed(egui::Key::Equals) && i.modifiers.ctrl) {
                        td.font_size = (td.font_size + FONT_SIZE_STEP).min(MAX_FONT_SIZE);
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl) {
                        td.font_size = (td.font_size - FONT_SIZE_STEP).max(MIN_FONT_SIZE);
                    }
                }

                let font_id = egui::FontId::monospace(td.font_size);
                let cell_w = ui.fonts(|f| f.glyph_width(&font_id, 'm'));
                let cell_h = ui.fonts(|f| f.row_height(&font_id));
                let avail = ui.available_size();
                let pty_cols = (avail.x / cell_w).floor() as u16;
                let pty_rows = (avail.y / cell_h).floor() as u16;
                if pty_cols > 0 && pty_rows > 0 {
                    td.instance.resize_pty(pty_cols, pty_rows);
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

                let terminal_response = render_terminal(ui, &td.instance, cell_w, cell_h);

                if terminal_response.clicked() {
                    *self.focused_terminal = Some(tab.clone());
                }

                if is_focused && !self.renaming {
                    let any_key = ui.input(|i| {
                        i.events.iter().any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Key { .. }))
                    });
                    if any_key {
                        td.restored_snapshot = None;
                    }

                    let input = ui.input(|i| i.clone());
                    for event in &input.events {
                        match event {
                            egui::Event::Text(text) => {
                                td.instance.write(text.as_bytes());
                            }
                            egui::Event::Key { key, pressed, modifiers, .. } if *pressed => {
                                match key {
                                    egui::Key::Enter => {
                                        if td.instance.history_nav.is_some() {
                                            // Menu is open: Enter is handled by the rendering section
                                        } else {
                                            let line = td.instance.get_current_line();
                                            let prompt_end = line.rfind("$ ").or_else(|| line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                                            let cmd = line[prompt_end..].trim().to_string();
                                            if !cmd.is_empty() {
                                                self.history_db.add(tab, &cmd);
                                            }
                                            td.instance.write(b"\r");
                                        }
                                    }
                                    egui::Key::Tab => {
                                        let now = std::time::Instant::now();
                                        let is_double = td.instance.last_tab_time
                                            .map(|t| now.duration_since(t).as_millis() < 500)
                                            .unwrap_or(false);
                                        td.instance.last_tab_time = Some(now);
                                        if is_double {
                                            td.instance.write(&[0x09]);
                                        } else {
                                            let line = td.instance.get_current_line();
                                            let prompt_end = line.rfind("$ ").or_else(|| line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                                            let input_text = line[prompt_end..].trim().to_string();
                                            if !input_text.is_empty() {
                                                if let Some(best) = self.completion.suggest(&input_text).first() {
                                                    if best.len() > input_text.len() {
                                                        let remaining = &best[input_text.len()..];
                                                        td.instance.write(remaining.as_bytes());
                                                    } else {
                                                        td.instance.write(&[0x09]);
                                                    }
                                                } else {
                                                    td.instance.write(&[0x09]);
                                                }
                                            } else {
                                                td.instance.write(&[0x09]);
                                            }
                                        }
                                    }
                                    egui::Key::ArrowUp => {
                                        if td.instance.history_nav.is_some() {
                                            if let Some(ref mut nav) = td.instance.history_nav {
                                                if nav.selected > 0 {
                                                    nav.selected -= 1;
                                                }
                                            }
                                        } else {
                                            let entries = self.history_db.get(tab, self.max_history);
                                            if !entries.is_empty() {
                                                td.instance.history_nav = Some(HistoryNav { entries, selected: 0 });
                                            } else {
                                                td.instance.write(b"\x1b[A");
                                            }
                                        }
                                    }
                                    egui::Key::ArrowDown => {
                                        if let Some(ref mut nav) = td.instance.history_nav {
                                            if nav.selected + 1 < nav.entries.len() {
                                                nav.selected += 1;
                                            }
                                        } else {
                                            td.instance.write(b"\x1b[B");
                                        }
                                    }
                                    egui::Key::ArrowRight => {
                                        if td.instance.history_nav.is_none() {
                                            let line = td.instance.get_current_line();
                                            let prompt_end = line.rfind("$ ").or_else(|| line.rfind("# ")).map(|p| p + 2).unwrap_or(0);
                                            let input_text = line[prompt_end..].trim().to_string();
                                            if !input_text.is_empty() {
                                                if let Some(best) = self.completion.suggest(&input_text).first() {
                                                    if best.len() > input_text.len() {
                                                        let remaining = &best[input_text.len()..];
                                                        td.instance.write(remaining.as_bytes());
                                                    } else {
                                                        td.instance.write(b"\x1b[C");
                                                    }
                                                } else {
                                                    td.instance.write(b"\x1b[C");
                                                }
                                            } else {
                                                td.instance.write(b"\x1b[C");
                                            }
                                        } else {
                                            td.instance.write(b"\x1b[C");
                                        }
                                    }
                                    egui::Key::Escape => {
                                        if td.instance.history_nav.is_some() {
                                            td.instance.history_nav = None;
                                        } else {
                                            td.instance.write(b"\x1b");
                                        }
                                    }
                                    egui::Key::Backspace => td.instance.write(b"\x7f"),
                                    egui::Key::Delete => td.instance.write(b"\x1b[3~"),
                                    egui::Key::Home => td.instance.write(b"\x1b[H"),
                                    egui::Key::End => td.instance.write(b"\x1b[F"),
                                    egui::Key::PageUp => td.instance.write(b"\x1b[5~"),
                                    egui::Key::PageDown => td.instance.write(b"\x1b[6~"),
                                    egui::Key::ArrowLeft => td.instance.write(b"\x1b[D"),
                                    _ => {
                                        if let Some(c) = key_to_char(key, modifiers) {
                                            td.instance.write(&[c as u8]);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Some(ref nav) = td.instance.history_nav {
                        let (_, cursor_row) = td.instance.cursor_position();
                        let list_width = 400.0;
                        let max_visible = 10;
                        let visible_count = nav.entries.len().min(max_visible);
                        let list_height = visible_count as f32 * cell_h;
                        let below_top = terminal_response.rect.min.y + (cursor_row as f32 + 1.0) * cell_h;
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
                        ui.painter().rect_filled(list_rect, 0.0, egui::Color32::from_rgba_unmultiplied(30, 30, 30, 240));

                        let start_idx = if nav.selected >= max_visible { nav.selected - max_visible + 1 } else { 0 };
                        for (i, entry) in nav.entries[start_idx..].iter().enumerate().take(max_visible) {
                            let y = list_top + i as f32 * cell_h;
                            let is_selected = start_idx + i == nav.selected;
                            let bg = if is_selected {
                                egui::Color32::from_rgba_unmultiplied(60, 60, 80, 255)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(30, 30, 30, 240)
                            };
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(egui::pos2(terminal_response.rect.min.x, y), egui::vec2(list_width, cell_h)),
                                0.0, bg,
                            );
                            ui.painter().text(
                                egui::pos2(terminal_response.rect.min.x + 4.0, y),
                                egui::Align2::LEFT_TOP,
                                &entry,
                                egui::FontId::monospace(td.font_size),
                                egui::Color32::WHITE,
                            );
                        }

                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Some(selected) = td.instance.history_nav.as_ref().map(|n| n.entries[n.selected].clone()) {
                                td.instance.write(selected.as_bytes());
                                td.instance.history_nav = None;
                            }
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
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

fn key_to_char(key: &egui::Key, modifiers: &egui::Modifiers) -> Option<char> {
    if modifiers.ctrl {
        return match key {
            egui::Key::A => Some('\x01'), egui::Key::B => Some('\x02'),
            egui::Key::C => Some('\x03'), egui::Key::D => Some('\x04'),
            egui::Key::E => Some('\x05'), egui::Key::F => Some('\x06'),
            egui::Key::G => Some('\x07'), egui::Key::H => Some('\x08'),
            egui::Key::I => Some('\x09'), egui::Key::J => Some('\x0a'),
            egui::Key::K => Some('\x0b'), egui::Key::L => Some('\x0c'),
            egui::Key::M => Some('\x0d'), egui::Key::N => Some('\x0e'),
            egui::Key::O => Some('\x0f'), egui::Key::P => Some('\x10'),
            egui::Key::Q => Some('\x11'), egui::Key::R => Some('\x12'),
            egui::Key::S => Some('\x13'), egui::Key::T => Some('\x14'),
            egui::Key::U => Some('\x15'), egui::Key::V => Some('\x16'),
            egui::Key::W => Some('\x17'), egui::Key::X => Some('\x18'),
            egui::Key::Y => Some('\x19'), egui::Key::Z => Some('\x1a'),
            egui::Key::Num0 => Some('\x00'),
            _ => None,
        };
    }
    match key {
        egui::Key::Space => Some(' '),
        _ => None,
    }
}
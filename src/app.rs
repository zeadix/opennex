use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use egui_term::{PtyEvent, TerminalBackend, TerminalView, TerminalFont, FontSettings};
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
    workspace: WorkspaceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceSettings {
    template_dir: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            workspace: WorkspaceSettings {
                template_dir: "workspace".to_string(),
            },
        }
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
}

// ── App ──────────────────────────────────────────────────────────

pub struct App {
    panels: Vec<Panel>,
    active_panel: usize,
    dock_states: HashMap<usize, DockState<String>>,
    terminals: HashMap<String, TerminalData>,
    tab_counter: u32,
    pending_new_terminal: Option<usize>,
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
    settings: AppSettings,
    show_settings: bool,
    settings_tab: SettingsTab,
    settings_edit: AppSettings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingsTab {
    Workspace,
}

struct Panel {
    name: String,
    bound_file: Option<PathBuf>,
}

struct TerminalData {
    backend: TerminalBackend,
    receiver: Receiver<(u64, PtyEvent)>,
    name: String,
    font_size: f32,
    working_directory: String,
}

// ── Workspace directory ──────────────────────────────────────────

fn workspace_dir() -> PathBuf {
    let settings = load_settings();
    std::env::current_dir()
        .unwrap_or_default()
        .join(&settings.workspace.template_dir)
}

fn workspace_dir_for(settings: &AppSettings) -> PathBuf {
    std::env::current_dir()
        .unwrap_or_default()
        .join(&settings.workspace.template_dir)
}

fn ensure_workspace_dir() {
    let _ = std::fs::create_dir_all(workspace_dir());
}

fn workspace_file_for(name: &str) -> PathBuf {
    workspace_dir().join(format!("{}.json", name))
}

fn list_workspace_files() -> Vec<(String, PathBuf)> {
    let ws_dir = workspace_dir();
    std::fs::read_dir(&ws_dir)
        .into_iter()
        .flat_map(|rd| rd.into_iter())
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .map(|e| e.path())
        .filter_map(|path| {
            let stem = path.file_stem()?.to_string_lossy().to_string();
            Some((stem, path))
        })
        .collect()
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

// ── App impl ─────────────────────────────────────────────────────

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        ensure_workspace_dir();
        let settings = load_settings();
        let mut app = App::empty();
        app.settings = settings.clone();
        app.settings_edit = settings;
        app.add_initial_terminal(ctx);
        app
    }

    fn empty() -> Self {
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
            settings: AppSettings::default(),
            show_settings: false,
            settings_tab: SettingsTab::Workspace,
            settings_edit: AppSettings::default(),
        }
    }

    fn add_initial_terminal(&mut self, ctx: &egui::Context) {
        let name = "Workspace 1".to_string();
        let tab_id = self.create_terminal_inner(ctx);
        self.dock_states.insert(0, DockState::new(vec![tab_id]));
        self.panels.push(Panel { name, bound_file: None });
        self.active_panel = 0;
    }

    fn load_workspace_state(&mut self, ctx: &egui::Context, state: WorkspaceState, file: Option<PathBuf>) {
        let panel_idx = self.panels.len();

        // Restore terminals
        for (id, tstate) in &state.terminals {
            if !self.terminals.contains_key(id) {
                let id_num: u64 = id.strip_prefix("terminal-")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(self.tab_counter as u64 + 1);
                let (backend, receiver) = create_terminal(ctx, id_num);
                self.terminals.insert(id.clone(), TerminalData {
                    backend,
                    receiver,
                    name: tstate.name.clone(),
                    font_size: tstate.font_size,
                    working_directory: tstate.working_directory.clone(),
                });
                if id_num >= self.tab_counter as u64 {
                    self.tab_counter = id_num as u32;
                }
            }
        }

        // Restore panel and dock state
        self.panels.push(Panel { name: state.panel_name, bound_file: file });
        self.dock_states.insert(panel_idx, state.dock_state);
    }

    fn is_renaming(&self) -> bool {
        self.renaming_panel.is_some() || self.renaming_terminal.is_some()
    }

    fn create_terminal_inner(&mut self, ctx: &egui::Context) -> String {
        self.tab_counter += 1;
        let id = format!("terminal-{}", self.tab_counter);
        let (backend, receiver) = create_terminal(ctx, self.tab_counter as u64);
        let cwd = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        self.terminals.insert(id.clone(), TerminalData {
            backend, receiver,
            name: format!("Terminal {}", self.tab_counter),
            font_size: DEFAULT_FONT_SIZE,
            working_directory: cwd,
        });
        id
    }

    fn create_terminal(&mut self, ctx: &egui::Context) -> String {
        self.create_terminal_inner(ctx)
    }

    fn process_pending(&mut self, ctx: &egui::Context) {
        if let Some(panel_idx) = self.pending_new_terminal.take() {
            let tab_id = self.create_terminal(ctx);
            if let Some(tree) = self.dock_states.get_mut(&panel_idx) {
                if let Some(ref after_tab) = self.pending_split_after.clone() {
                    if let Some((_surface, node_idx, _)) = tree.find_tab(after_tab) {
                        if self.pending_split_vertical {
                            tree.main_surface_mut().split_below(node_idx, 0.5, vec![tab_id]);
                        } else {
                            tree.main_surface_mut().split_right(node_idx, 0.5, vec![tab_id]);
                        }
                    } else {
                        tree.main_surface_mut().push_to_first_leaf(tab_id);
                    }
                } else {
                    tree.main_surface_mut().push_to_first_leaf(tab_id);
                }
                self.pending_split_after = None;
            } else {
                self.dock_states.insert(panel_idx, DockState::new(vec![tab_id]));
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
            if let Ok(state) = load_from_file(&path) {
                self.load_workspace_state(ctx, state, None);
                self.active_panel = self.panels.len() - 1;
            }
        }
    }

    fn add_panel(&mut self, ctx: &egui::Context) {
        let idx = self.panels.len();
        let name = format!("Workspace {}", idx + 1);
        let tab_id = self.create_terminal(ctx);
        self.dock_states.insert(idx, DockState::new(vec![tab_id]));
        self.panels.push(Panel { name, bound_file: None });
        self.active_panel = idx;
    }

    fn save_workspace(&mut self, panel_idx: usize) {
        if let Some(state) = build_panel_state(self, panel_idx) {
            let path = if let Some(ref file) = self.panels[panel_idx].bound_file {
                file.clone()
            } else {
                // No bound file — ask user where to save
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

    fn close_workspace(&mut self, idx: usize) {
        if self.panels.len() <= 1 {
            return;
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

fn create_terminal(ctx: &egui::Context, id: u64) -> (TerminalBackend, Receiver<(u64, PtyEvent)>) {
    #[cfg(unix)]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    #[cfg(not(unix))]
    let shell = "cmd.exe".to_string();

    let (sender, receiver) = std::sync::mpsc::channel();
    let backend = TerminalBackend::new(id, ctx.clone(), sender, egui_term::BackendSettings {
        shell, ..Default::default()
    }).unwrap();
    (backend, receiver)
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
                    if ui.button("New Workspace").clicked() {
                        self.add_panel(ui.ctx());
                        ui.close_menu();
                    }
                    if ui.button("Load Workspace").clicked() {
                        self.pending_load_workspace = true;
                        ui.close_menu();
                    }
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
                                ui.close_menu();
                            }
                            if ui.button("Split Down").clicked() {
                                self.pending_split_after = Some(tab.clone());
                                self.pending_split_vertical = true;
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
            egui::Window::new("Settings")
                .open(&mut open)
                .resizable(true)
                .default_width(500.0)
                .default_height(300.0)
                .show(ctx, |ui| {
                    ui.columns(2, |cols| {
                        // Left: tabs
                        cols[0].vertical(|ui| {
                            ui.heading("Settings");
                            ui.separator();
                            if ui.selectable_label(self.settings_tab == SettingsTab::Workspace, "Workspace").clicked() {
                                self.settings_tab = SettingsTab::Workspace;
                            }
                        });

                        // Right: content
                        cols[1].vertical(|ui| {
                            match self.settings_tab {
                                SettingsTab::Workspace => {
                                    ui.heading("Workspace Settings");
                                    ui.separator();
                                    ui.label("Template Directory:");
                                    ui.add(egui::TextEdit::singleline(&mut self.settings_edit.workspace.template_dir)
                                        .desired_width(ui.available_width()));
                                    ui.label("Default: workspace (relative to app root)");
                                }
                            }
                        });
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.settings = self.settings_edit.clone();
                            let _ = save_settings(&self.settings);
                        }
                        if ui.button("Cancel").clicked() {
                            self.settings_edit = self.settings.clone();
                        }
                    });
                });
            self.show_settings = open;
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
                    let panel_name = self.panels[i].name.clone();
                    let response = ui.selectable_label(is_active, &panel_name);
                    if response.clicked() && !renaming { to_select = Some(i); }
                    if response.double_clicked() && !renaming {
                        self.renaming_panel = Some(i);
                        self.rename_buffer = panel_name;
                        self.rename_frame_count = 0;
                    }

                    let panel_idx = i;
                    response.context_menu(|ui| {
                        if ui.button("Rename").clicked() {
                            self.renaming_panel = Some(panel_idx);
                            self.rename_buffer = self.panels[panel_idx].name.clone();
                            self.rename_frame_count = 0;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Save").clicked() {
                            self.save_workspace(panel_idx);
                            ui.close_menu();
                        }
                        if ui.button("Save As...").clicked() {
                            self.save_workspace_as(panel_idx);
                            ui.close_menu();
                        }
                        if ui.button("Load...").clicked() {
                            self.pending_load_workspace = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Close").clicked() {
                            self.close_workspace(panel_idx);
                            ui.close_menu();
                        }
                    });
                }
            }

            if let Some(i) = to_select { self.active_panel = i; }

            ui.separator();
            if ui.button("+ New Workspace").clicked() {
                self.add_panel(ui.ctx());
            }

            // Template button — click to show workspace files
            let template_files = list_workspace_files();
            ui.menu_button("Templates", |ui| {
                if template_files.is_empty() {
                    ui.label("(empty)");
                } else {
                    for (display_name, path) in &template_files {
                        let path = path.clone();
                        if ui.button(display_name.as_str()).clicked() {
                            self.pending_load_from_template = Some(path);
                            ui.close_menu();
                        }
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
    pending_close: &'a mut Option<String>,
    pending_new_terminal: &'a mut Option<usize>,
    pending_split_after: &'a mut Option<String>,
    pending_split_vertical: &'a mut bool,
    active_panel: usize,
    renaming_terminal: &'a mut Option<String>,
    terminal_rename_buffer: &'a mut String,
    renaming: bool,
    rename_frame_count: u32,
    active_tab: Option<String>,
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

            let terminal_view = TerminalView::new(ui, &mut terminal_data.backend)
                .set_focus(!self.renaming)
                .set_font(font)
                .set_size(ui.available_size());
            ui.add(terminal_view);
        } else {
            ui.label("Terminal not found");
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        *self.pending_close = Some(tab.clone());
        true
    }

    fn on_add(&mut self, _surface: SurfaceIndex, _node: NodeIndex) {
        *self.pending_new_terminal = Some(self.active_panel);
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.double_clicked() {
            *self.renaming_terminal = Some(tab.clone());
            if let Some(data) = self.terminals.get(tab) {
                *self.terminal_rename_buffer = data.name.clone();
            }
            self.rename_frame_count = 0;
        }
    }

    fn add_popup(&mut self, _ui: &mut egui::Ui, _surface: SurfaceIndex, _node: NodeIndex) {}

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab, _surface: SurfaceIndex, _node: NodeIndex) {
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
            *self.pending_new_terminal = Some(self.active_panel);
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Split Horizontal (Right)").clicked() {
            *self.pending_split_after = Some(tab.clone());
            *self.pending_split_vertical = false;
            *self.pending_new_terminal = Some(self.active_panel);
            ui.close_menu();
        }
        if ui.button("Split Vertical (Down)").clicked() {
            *self.pending_split_after = Some(tab.clone());
            *self.pending_split_vertical = true;
            *self.pending_new_terminal = Some(self.active_panel);
            ui.close_menu();
        }
    }
}

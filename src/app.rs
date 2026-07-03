use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

pub struct App {
    panels: Vec<Panel>,
    active_panel: usize,
    panel_counter: u32,
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
}

struct Panel {
    name: String,
}

struct TerminalData {
    backend: TerminalBackend,
    receiver: Receiver<(u64, PtyEvent)>,
    name: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut terminals = HashMap::new();
        let tab_id = "terminal-1".to_string();
        let (backend, receiver) = create_terminal(&cc.egui_ctx, 0);
        terminals.insert(
            tab_id.clone(),
            TerminalData {
                backend,
                receiver,
                name: "Terminal 1".to_string(),
            },
        );

        let mut dock_states = HashMap::new();
        dock_states.insert(0, DockState::new(vec![tab_id]));

        App {
            panels: vec![Panel {
                name: "Workspace 1".to_string(),
            }],
            active_panel: 0,
            panel_counter: 1,
            dock_states,
            terminals,
            tab_counter: 1,
            pending_new_terminal: None,
            pending_close: None,
            pending_split_after: None,
            pending_split_vertical: false,
            renaming_panel: None,
            rename_buffer: String::new(),
            renaming_terminal: None,
            terminal_rename_buffer: String::new(),
        }
    }

    fn create_terminal(&mut self, ctx: &egui::Context) -> String {
        self.tab_counter += 1;
        let id = format!("terminal-{}", self.tab_counter);
        let (backend, receiver) = create_terminal(ctx, self.terminals.len() as u64);
        self.terminals.insert(
            id.clone(),
            TerminalData {
                backend,
                receiver,
                name: format!("Terminal {}", self.tab_counter),
            },
        );
        id
    }

    fn process_pending(&mut self, ctx: &egui::Context) {
        if let Some(panel_idx) = self.pending_new_terminal.take() {
            let tab_id = self.create_terminal(ctx);

            if let Some(tree) = self.dock_states.get_mut(&panel_idx) {
                if let Some(ref after_tab) = self.pending_split_after.clone() {
                    if let Some((_surface, node_idx, _tab_idx)) = tree.find_tab(after_tab) {
                        if self.pending_split_vertical {
                            tree.main_surface_mut()
                                .split_below(node_idx, 0.5, vec![tab_id]);
                        } else {
                            tree.main_surface_mut()
                                .split_right(node_idx, 0.5, vec![tab_id]);
                        }
                    } else {
                        tree.main_surface_mut().push_to_first_leaf(tab_id);
                    }
                } else {
                    tree.main_surface_mut().push_to_first_leaf(tab_id);
                }
                self.pending_split_after = None;
            } else {
                self.dock_states
                    .insert(panel_idx, DockState::new(vec![tab_id]));
            }
        }

        if let Some(tab_id) = self.pending_close.take() {
            self.terminals.remove(&tab_id);
        }
    }

    fn is_renaming(&self) -> bool {
        self.renaming_panel.is_some() || self.renaming_terminal.is_some()
    }

    fn add_panel(&mut self) {
        self.panel_counter += 1;
        self.panels.push(Panel {
            name: format!("Workspace {}", self.panel_counter),
        });
        let idx = self.panels.len() - 1;
        self.active_panel = idx;
        self.pending_new_terminal = Some(idx);
    }

    fn render_rename_popup(&mut self, ctx: &egui::Context) {
        if self.renaming_panel.is_none() && self.renaming_terminal.is_none() {
            return;
        }

        let screen = ctx.screen_rect();
        let popup_size = egui::vec2(320.0, 130.0);
        let popup_rect = egui::Rect::from_center_size(screen.center(), popup_size);

        // Dark overlay to block interaction with background
        egui::Area::new(egui::Id::new("rename_overlay"))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.painter().rect_filled(screen, egui::Rounding::ZERO, egui::Color32::from_black_alpha(128));
            });

        // Popup content
        egui::Area::new(egui::Id::new("rename_popup"))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.allocate_ui_at_rect(popup_rect, |ui| {
                        ui.set_min_width(popup_size.x - 20.0);

                        if let Some(panel_idx) = self.renaming_panel {
                            ui.heading("Rename Workspace");
                            ui.separator();
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.rename_buffer)
                                    .font(egui::FontId::monospace(14.0))
                                    .desired_width(ui.available_width()),
                            );
                            response.request_focus();
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("OK").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    if !self.rename_buffer.is_empty() {
                                        self.panels[panel_idx].name = self.rename_buffer.clone();
                                    }
                                    self.renaming_panel = None;
                                }
                                if ui.button("Cancel").clicked() {
                                    self.renaming_panel = None;
                                }
                            });
                        } else if let Some(ref tab_id) = self.renaming_terminal.clone() {
                            ui.heading("Rename Terminal");
                            ui.separator();
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.terminal_rename_buffer)
                                    .font(egui::FontId::monospace(14.0))
                                    .desired_width(ui.available_width()),
                            );
                            response.request_focus();
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("OK").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    if !self.terminal_rename_buffer.is_empty() {
                                        if let Some(data) = self.terminals.get_mut(tab_id) {
                                            data.name = self.terminal_rename_buffer.clone();
                                        }
                                    }
                                    self.renaming_terminal = None;
                                }
                                if ui.button("Cancel").clicked() {
                                    self.renaming_terminal = None;
                                }
                            });
                        }
                    });
                });
            });
    }
}

fn create_terminal(ctx: &egui::Context, id: u64) -> (TerminalBackend, Receiver<(u64, PtyEvent)>) {
    #[cfg(unix)]
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    #[cfg(not(unix))]
    let shell = "cmd.exe".to_string();

    let (sender, receiver) = std::sync::mpsc::channel();
    let backend = TerminalBackend::new(
        id,
        ctx.clone(),
        sender,
        egui_term::BackendSettings {
            shell,
            ..Default::default()
        },
    )
    .unwrap();
    (backend, receiver)
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_pending(ctx);
        let renaming = self.is_renaming();

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Workspace").clicked() {
                        self.add_panel();
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
            });
        });

        // Left panel - workspace navigation
        egui::SidePanel::left("navigation")
            .default_width(160.0)
            .show(ctx, |ui| {
                ui.heading("Workspaces");
                ui.separator();

                let mut to_select = None;
                let panel_count = self.panels.len();
                for i in 0..panel_count {
                    let is_active = i == self.active_panel;
                    let panel_name = self.panels[i].name.clone();
                    let response = ui.selectable_label(is_active, &panel_name);

                    if response.clicked() && !renaming {
                        to_select = Some(i);
                    }

                    if response.double_clicked() && !renaming {
                        self.renaming_panel = Some(i);
                        self.rename_buffer = panel_name;
                    }
                }

                if let Some(i) = to_select {
                    self.active_panel = i;
                }

                ui.separator();
                if ui.button("+ New Workspace").clicked() {
                    self.add_panel();
                }

                ui.separator();
                ui.label("L1: Workspaces");
                ui.label("L2: Dock panels");
                ui.label("L3: Terminal tabs");
            });

        // Right panel - dock area
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
                        current_tab: None,
                        renaming_terminal: &mut self.renaming_terminal,
                        terminal_rename_buffer: &mut self.terminal_rename_buffer,
                        renaming,
                    });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Click '+ New Workspace' to create one.");
                });
            }
        });

        // Render rename popups last (on top of everything)
        self.render_rename_popup(ctx);
    }
}

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    pending_close: &'a mut Option<String>,
    pending_new_terminal: &'a mut Option<usize>,
    pending_split_after: &'a mut Option<String>,
    pending_split_vertical: &'a mut bool,
    active_panel: usize,
    current_tab: Option<String>,
    renaming_terminal: &'a mut Option<String>,
    terminal_rename_buffer: &'a mut String,
    renaming: bool,
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
        self.current_tab = Some(tab.clone());

        if let Some(terminal_data) = self.terminals.get_mut(tab) {
            if let Ok((_, PtyEvent::Exit)) = terminal_data.receiver.try_recv() {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            // KEY FIX: TerminalView.request_focus() steals focus every frame.
            // Only steal focus when NOT renaming, so TextEdit can receive input.
            let terminal_view = TerminalView::new(ui, &mut terminal_data.backend)
                .set_focus(!self.renaming)
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
        }
    }

    fn add_popup(&mut self, _ui: &mut egui::Ui, _surface: SurfaceIndex, _node: NodeIndex) {}

    fn context_menu(
        &mut self,
        ui: &mut egui::Ui,
        tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        ui.label("Tab Operations");
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

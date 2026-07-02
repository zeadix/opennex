use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;

pub struct App {
    tree: DockState<String>,
    terminals: HashMap<String, TerminalData>,
    tab_counter: u32,
    pending_new_tab: bool,
    pending_split_after: Option<String>,
    pending_split_vertical: bool,
    pending_close: Option<String>,
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
        let tree = DockState::new(vec![tab_id]);

        App {
            tree,
            terminals,
            tab_counter: 1,
            pending_new_tab: false,
            pending_split_after: None,
            pending_split_vertical: false,
            pending_close: None,
        }
    }

    fn create_new_terminal(&mut self, ctx: &egui::Context) -> String {
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
        if self.pending_new_tab {
            let tab_id = self.create_new_terminal(ctx);
            self.tree.main_surface_mut().push_to_first_leaf(tab_id);
            self.pending_new_tab = false;
        }

        if let Some(after_tab) = self.pending_split_after.take() {
            let tab_id = self.create_new_terminal(ctx);
            let vertical = self.pending_split_vertical;

            if let Some((_surface, node_idx, _tab_idx)) = self.tree.find_tab(&after_tab) {
                if vertical {
                    self.tree.main_surface_mut().split_below(node_idx, 0.5, vec![tab_id]);
                } else {
                    self.tree.main_surface_mut().split_right(node_idx, 0.5, vec![tab_id]);
                }
            }
        }

        if let Some(tab_id) = self.pending_close.take() {
            self.terminals.remove(&tab_id);
        }
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

        let active_tab = self.tree.find_active_focused().map(|(_, t)| t.clone());

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Terminal").clicked() {
                        self.pending_new_tab = true;
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
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
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let App {
                ref mut tree,
                ref mut terminals,
                ..
            } = *self;

            DockArea::new(tree)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut TerminalTabViewer { terminals, pending_close: &mut self.pending_close });
        });
    }
}

struct TerminalTabViewer<'a> {
    terminals: &'a mut HashMap<String, TerminalData>,
    pending_close: &'a mut Option<String>,
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
        if let Some(terminal_data) = self.terminals.get_mut(tab) {
            if let Ok((_, PtyEvent::Exit)) = terminal_data.receiver.try_recv() {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            let terminal_view = TerminalView::new(ui, &mut terminal_data.backend)
                .set_focus(true)
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
        // Will be handled in next frame via pending_new_tab
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, _surface: SurfaceIndex, _node: NodeIndex) {
        ui.label("Right-click tab for options");
    }
}

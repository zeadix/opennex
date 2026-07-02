use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::sync::mpsc::Receiver;

pub struct App {
    egui_ctx: egui::Context,
    terminals: Vec<TerminalInstance>,
    active_terminal: usize,
    pending_new_terminal: bool,
}

struct TerminalInstance {
    name: String,
    backend: TerminalBackend,
    receiver: Receiver<(u64, PtyEvent)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = App {
            egui_ctx: cc.egui_ctx.clone(),
            terminals: Vec::new(),
            active_terminal: 0,
            pending_new_terminal: false,
        };
        app.add_terminal("Terminal 1");
        app
    }

    fn add_terminal(&mut self, name: &str) {
        #[cfg(unix)]
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        #[cfg(not(unix))]
        let shell = "cmd.exe".to_string();

        let (sender, receiver) = std::sync::mpsc::channel();
        let backend = TerminalBackend::new(
            self.terminals.len() as u64,
            self.egui_ctx.clone(),
            sender,
            egui_term::BackendSettings {
                shell,
                ..Default::default()
            },
        )
        .unwrap();

        self.terminals.push(TerminalInstance {
            name: name.to_string(),
            backend,
            receiver,
        });
        self.active_terminal = self.terminals.len() - 1;
    }

    fn close_terminal(&mut self, index: usize) {
        if self.terminals.len() > 1 {
            self.terminals.remove(index);
            if self.active_terminal >= self.terminals.len() {
                self.active_terminal = self.terminals.len() - 1;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for terminal in &self.terminals {
            if let Ok((_, PtyEvent::Exit)) = terminal.receiver.try_recv() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Terminal").clicked() {
                        self.pending_new_terminal = true;
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Terminal", |ui| {
                    if ui.button("Close Current").clicked() {
                        self.close_terminal(self.active_terminal);
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::left("terminal_list")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Terminals");
                ui.separator();

                let mut to_close = None;
                let mut to_select = None;
                for (i, terminal) in self.terminals.iter().enumerate() {
                    let is_active = i == self.active_terminal;
                    if ui.selectable_label(is_active, &terminal.name).clicked() {
                        to_select = Some(i);
                    }
                    if is_active && self.terminals.len() > 1 {
                        if ui.small_button("x").clicked() {
                            to_close = Some(i);
                        }
                    }
                }

                if let Some(i) = to_close {
                    self.close_terminal(i);
                }
                if let Some(i) = to_select {
                    self.active_terminal = i;
                }

                ui.separator();
                if ui.button("+ New Terminal").clicked() {
                    self.pending_new_terminal = true;
                }

                ui.separator();
                ui.label("Shortcuts:");
                ui.label("Ctrl+Q: Quit");
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(terminal) = self.terminals.get_mut(self.active_terminal) {
                if let Ok((_, PtyEvent::Exit)) = terminal.receiver.try_recv() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }

                let terminal_view = TerminalView::new(ui, &mut terminal.backend)
                    .set_focus(true)
                    .set_size(ui.available_size());

                ui.add(terminal_view);
            }
        });

        if self.pending_new_terminal {
            let name = format!("Terminal {}", self.terminals.len() + 1);
            self.add_terminal(&name);
            self.pending_new_terminal = false;
        }
    }
}

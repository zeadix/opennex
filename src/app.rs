use eframe::egui;

pub struct App {
    terminals: Vec<Terminal>,
    active_terminal: usize,
    tab_counter: u32,
    pending_command: Option<String>,
}

struct Terminal {
    id: String,
    title: String,
    content: String,
    input_buffer: String,
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            terminals: Vec::new(),
            active_terminal: 0,
            tab_counter: 0,
            pending_command: None,
        };
        app.new_terminal();
        app
    }
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn new_terminal(&mut self) {
        self.tab_counter += 1;
        self.terminals.push(Terminal {
            id: format!("terminal-{}", self.tab_counter),
            title: format!("终端 {}", self.tab_counter),
            content: String::new(),
            input_buffer: String::new(),
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

    fn execute_command(&mut self, command: &str) {
        let output = match command.trim() {
            "help" => "可用命令: help, clear, ls, pwd, echo <text>".to_string(),
            "clear" => {
                self.terminals[self.active_terminal].content.clear();
                return;
            }
            "ls" => "文件列表: src/ tests/ Cargo.toml README.md".to_string(),
            "pwd" => std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/".to_string()),
            cmd if cmd.starts_with("echo ") => cmd[5..].to_string(),
            cmd => format!("命令已执行: {}", cmd),
        };

        self.terminals[self.active_terminal]
            .content
            .push_str(&format!("$ {}\n{}\n", command, output));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("新建终端").clicked() {
                        self.new_terminal();
                        ui.close_menu();
                    }
                    if ui.button("退出").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("终端", |ui| {
                    if ui.button("清屏").clicked() {
                        self.terminals[self.active_terminal].content.clear();
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::left("terminal_list")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("终端");
                ui.separator();

                let mut to_close = None;
                for (i, terminal) in self.terminals.iter().enumerate() {
                    let is_active = i == self.active_terminal;
                    if ui
                        .selectable_label(is_active, &terminal.title)
                        .clicked()
                    {
                        self.active_terminal = i;
                    }
                    if is_active && self.terminals.len() > 1 {
                        if ui.small_button("×").clicked() {
                            to_close = Some(i);
                        }
                    }
                }

                if let Some(index) = to_close {
                    self.close_terminal(index);
                }

                ui.separator();
                if ui.button("+ 新建终端").clicked() {
                    self.new_terminal();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(terminal) = self.terminals.get_mut(self.active_terminal) {
                ui.heading(&terminal.title);
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut terminal.content)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                        );
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("$");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut terminal.input_buffer)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(ui.available_width()),
                    );

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        let command = terminal.input_buffer.clone();
                        terminal.input_buffer.clear();
                        self.pending_command = Some(command);
                    }
                });
            }

            if let Some(command) = self.pending_command.take() {
                self.execute_command(&command);
            }
        });
    }
}

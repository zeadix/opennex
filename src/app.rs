use eframe::egui;
use egui::FontData;

pub struct App {
    terminals: Vec<Terminal>,
    active_terminal: usize,
    tab_counter: u32,
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
        };
        app.new_terminal();
        app
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);
        Self::default()
    }

    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let font_paths = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        ];
        for path in &font_paths {
            if std::path::Path::new(path).exists() {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert("chinese".to_owned(), FontData::from_owned(font_data));
                    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                    fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
                    break;
                }
            }
        }
        ctx.set_fonts(fonts);
    }

    fn new_terminal(&mut self) {
        self.tab_counter += 1;
        self.terminals.push(Terminal {
            id: format!("terminal-{}", self.tab_counter),
            title: format!("Terminal {}", self.tab_counter),
            content: "Welcome to OpenZoo Terminal Manager\nType 'help' for available commands\n\n".to_string(),
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

    fn run_command(&mut self, command: String) {
        let cmd = command.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        let output = match cmd.as_str() {
            "help" => "Available commands:\n  help  - Show this help\n  clear - Clear screen\n  ls    - List files\n  pwd   - Print working directory\n  echo  - Echo text".to_string(),
            "clear" => {
                self.terminals[self.active_terminal].content.clear();
                return;
            }
            "ls" => "Files:\n  src/\n  tests/\n  Cargo.toml".to_string(),
            "pwd" => std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            other if other.starts_with("echo ") => other[5..].to_string(),
            other => format!("Executed: {}", other),
        };
        self.terminals[self.active_terminal].content.push_str(&format!("$ {}\n{}\n\n", cmd, output));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Terminal").clicked() { self.new_terminal(); }
                    if ui.button("Exit").clicked() { std::process::exit(0); }
                });
                ui.menu_button("Terminal", |ui| {
                    if ui.button("Clear").clicked() { self.terminals[self.active_terminal].content.clear(); }
                });
            });
        });

        egui::SidePanel::left("terminals").default_width(140.0).show(ctx, |ui| {
            ui.heading("Terminals");
            ui.separator();
            let mut to_close = None;
            for (i, t) in self.terminals.iter().enumerate() {
                if ui.selectable_label(i == self.active_terminal, &t.title).clicked() {
                    self.active_terminal = i;
                }
                if i == self.active_terminal && self.terminals.len() > 1 {
                    if ui.small_button("x").clicked() { to_close = Some(i); }
                }
            }
            if let Some(i) = to_close { self.close_terminal(i); }
            ui.separator();
            if ui.button("+ New").clicked() { self.new_terminal(); }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(terminal) = self.terminals.get_mut(self.active_terminal) {
                ui.heading(&terminal.title);
                ui.separator();

                egui::ScrollArea::vertical().auto_shrink([false; 2]).stick_to_bottom(true).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut terminal.content).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).interactive(false));
                });

                ui.separator();

                ui.label("Input command:");
                let edit_response = ui.add(
                    egui::TextEdit::multiline(&mut terminal.input_buffer)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(1)
                        .frame(true),
                );

                let mut should_run = false;
                let mut command = String::new();

                if ui.button("Execute").clicked() {
                    command = terminal.input_buffer.trim().to_string();
                    should_run = true;
                }

                if edit_response.has_focus() {
                    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let ctrl_enter = ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl);
                    if ctrl_enter || (enter && !terminal.input_buffer.ends_with('\n')) {
                        command = terminal.input_buffer.trim().to_string();
                        should_run = true;
                    }
                }

                if should_run && !command.is_empty() {
                    terminal.input_buffer.clear();
                    drop(terminal);
                    self.run_command(command);
                }
            }
        });
    }
}

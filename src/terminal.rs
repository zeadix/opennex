use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use wezterm_term::color::ColorPalette;
use wezterm_term::{Terminal, TerminalConfiguration, TerminalSize};

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Box<dyn Write + Send>>>);

impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

#[derive(Debug)]
pub struct SimpleConfig;

impl TerminalConfiguration for SimpleConfig {
    fn color_palette(&self) -> ColorPalette {
        ColorPalette::default()
    }
}

pub struct TerminalInstance {
    pub terminal: Arc<Mutex<Terminal>>,
    writer: SharedWriter,
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
    pub name: String,
    pub font_size: f32,
    pub working_directory: String,
    pub cwd_file: PathBuf,
    pub restored_snapshot: Option<crate::snapshot::state::TerminalSnapshot>,
    pub history_nav: Option<crate::app::HistoryNav>,
    pub screen_rows: usize,
    pub screen_cols: usize,
    grid_initialized: bool,
}

fn srgb_to_egui(srgb: wezterm_term::color::SrgbaTuple) -> egui::Color32 {
    egui::Color32::from_rgb(
        (srgb.0 * 255.0) as u8,
        (srgb.1 * 255.0) as u8,
        (srgb.2 * 255.0) as u8,
    )
}

impl TerminalInstance {
    pub fn create(shell: &str, cwd: &str, cols: u16, rows: u16) -> Option<Self> {
        let pair = native_pty_system().openpty(PtySize {
            rows, cols,
            pixel_width: cols * 8,
            pixel_height: rows * 18,
        }).ok()?;

        let master = pair.master;
        let reader = master.try_clone_reader().ok()?;
        let raw_writer = master.take_writer().ok()?;
        let shared_writer = Arc::new(Mutex::new(raw_writer));
        let child = pair.slave.spawn_command(
            {
                let mut cmd = CommandBuilder::new(shell);
                cmd.cwd(cwd);
                cmd.env("TERM", "xterm-256color");
                cmd
            },
        ).ok()?;

        let terminal_writer: Box<dyn Write + Send> = Box::new(SharedWriter(shared_writer.clone()));
        let user_writer = SharedWriter(shared_writer.clone());

        let size = TerminalSize {
            rows: rows as usize,
            cols: cols as usize,
            pixel_width: cols as usize * 8,
            pixel_height: rows as usize * 18,
            dpi: 96,
        };
        let config: Arc<dyn TerminalConfiguration + Send + Sync> = Arc::new(SimpleConfig);
        let terminal = Arc::new(Mutex::new(Terminal::new(
            size, config, "open-zoo", "0.1.0", terminal_writer,
        )));

        let t = terminal.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 65536];
            let mut reader = reader;
            loop {
                let n = match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };
                if let Ok(mut term) = t.lock() {
                    term.advance_bytes(&buf[..n]);
                }
            }
        });

        let cwd_str = cwd.to_string();
        Some(TerminalInstance {
            terminal,
writer: user_writer,
            master,
            _child: child,
            name: String::new(),
            font_size: 14.0,
            working_directory: cwd_str,
            cwd_file: PathBuf::new(),
            restored_snapshot: None,
            history_nav: None,
            screen_rows: rows as usize,
            screen_cols: cols as usize,
            grid_initialized: false,
        })
    }

    pub fn resize_pty(&mut self, cols: u16, rows: u16) {
        let _ = self.master.resize(PtySize {
            rows, cols,
            pixel_width: cols * 8,
            pixel_height: rows * 18,
        });

        if !self.grid_initialized {
            self.grid_initialized = true;
            if let Ok(mut term) = self.terminal.lock() {
                let size = TerminalSize {
                    rows: rows as usize,
                    cols: cols as usize,
                    pixel_width: cols as usize * 8,
                    pixel_height: rows as usize * 18,
                    dpi: 96,
                };
                term.resize(size);
                self.screen_cols = cols as usize;
                self.screen_rows = rows as usize;
            }
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        if let Ok(term) = self.terminal.lock() {
            let pos = term.cursor_pos();
            (pos.x as usize, pos.y as usize)
        } else {
            (0, 0)
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.screen_cols, self.screen_rows)
    }

    pub fn get_current_line(&self) -> String {
        let Ok(mut term) = self.terminal.lock() else { return String::new() };
        let pos = term.cursor_pos();
        let row = pos.y as usize;
        let mut line = String::new();
        let s = term.screen_mut();
        for col in 0..self.screen_cols {
            if let Some(cell) = s.get_cell(col, row as i64) {
                line.push_str(cell.str());
            }
        }
        line
    }
}

pub fn render_terminal(
    ui: &mut egui::Ui,
    instance: &TerminalInstance,
    cell_w: f32,
    cell_h: f32,
    bg_color: egui::Color32,
    fg_color: egui::Color32,
    cell_spacing: f32,
) -> egui::Response {
    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());
    let rect = response.rect;

    let effective_cell_w = cell_w * cell_spacing;

    painter.rect_filled(rect, egui::CornerRadius::ZERO, bg_color);

    if let Ok(mut term) = instance.terminal.lock() {
        let palette = term.palette();
        let show_rows = (rect.height() / cell_h).floor() as usize;
        let show_cols = (rect.width() / effective_cell_w).floor() as usize;
        let rows = show_rows.min(instance.screen_rows);
        let cols = show_cols.min(instance.screen_cols);
        let s = term.screen_mut();
        let mut skip_col = false;

        for row in 0..rows {
            let y = rect.min.y + row as f32 * cell_h;
            skip_col = false;
            for col in 0..cols {
                if skip_col { skip_col = false; continue; }
                if let Some(cell) = s.get_cell(col, row as i64) {
                    let text = cell.str();
                    if text.is_empty() || text == " " { continue; }
                    let attrs = cell.attrs();
                    let fg = srgb_to_egui(palette.resolve_fg(attrs.foreground()));
                    let x = rect.min.x + col as f32 * effective_cell_w;
                    let cw = cell.width().max(1) as f32 * effective_cell_w;

                    // Only draw cell background if it's not the default terminal background
                    if attrs.background() != wezterm_term::color::ColorAttribute::Default {
                        let bg = srgb_to_egui(palette.resolve_bg(attrs.background()));
                        if bg != bg_color {
                            painter.rect_filled(
                                egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cw, cell_h)),
                                egui::CornerRadius::ZERO, bg,
                            );
                        }
                    }

                    painter.text(
                        egui::pos2(x, y), egui::Align2::LEFT_TOP,
                        text,
                        egui::FontId::monospace(instance.font_size),
                        fg,
                    );
                    if cell.width() > 1 { skip_col = true; }
                }
            }
        }

        let pos = term.cursor_pos();
        let (pcol, prow) = (pos.x as usize, pos.y as usize);
        if pcol < cols && prow < rows {
            let cx = rect.min.x + pcol as f32 * effective_cell_w;
            let cy = rect.min.y + prow as f32 * cell_h;
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(cx, cy), egui::vec2(effective_cell_w, cell_h)),
                egui::CornerRadius::ZERO, egui::Color32::WHITE,
            );
        }
    }

    response
}

pub fn render_snapshot(
    ui: &mut egui::Ui,
    snapshot: &crate::snapshot::state::TerminalSnapshot,
    font_size: f32,
) -> egui::Response {
    let size = ui.available_size();
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
    let rect = response.rect;
    if let Some(first_row) = snapshot.grid.first() {
        let cols = first_row.len();
        let rows = snapshot.grid.len();
        let cell_w = if cols > 0 { size.x / cols as f32 } else { size.x };
        let cell_h = if rows > 0 { size.y / rows as f32 } else { size.y };
        painter.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::BLACK);
        for (row_idx, row) in snapshot.grid.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let x = rect.min.x + col_idx as f32 * cell_w;
                let y = rect.min.y + row_idx as f32 * cell_h;
                let fg = egui::Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2]);
                let bg = egui::Color32::from_rgb(cell.bg[0], cell.bg[1], cell.bg[2]);
                if bg != egui::Color32::BLACK {
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_w, cell_h)),
                        egui::CornerRadius::ZERO, bg,
                    );
                }
                painter.text(
                    egui::pos2(x, y), egui::Align2::LEFT_TOP,
                    cell.ch.to_string(),
                    egui::FontId::monospace(font_size), fg,
                );
            }
        }
        let (cx, cy) = snapshot.cursor;
        let cursor_x = rect.min.x + cx as f32 * cell_w;
        let cursor_y = rect.min.y + cy as f32 * cell_h;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(cursor_x, cursor_y), egui::vec2(cell_w, cell_h)),
            egui::CornerRadius::ZERO, egui::Color32::WHITE,
        );
    }
    response
}
pub mod grid;
pub mod parser;
pub mod render;

pub use grid::{Cell, CellFlags, Grid};
pub use render::{render_terminal, render_snapshot};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use vte::Parser as VteParser;

pub struct TerminalInstance {
    pub grid: Arc<Mutex<Grid>>,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
    pub name: String,
    pub font_size: f32,
    pub working_directory: String,
    pub cwd_file: PathBuf,
    pub restored_snapshot: Option<crate::snapshot::state::TerminalSnapshot>,
    pub history_nav: Option<crate::app::HistoryNav>,
    pub screen_cols: usize,
    pub screen_rows: usize,
}

#[derive(Clone)]
struct SharedWriter(Arc<Mutex<Box<dyn Write + Send>>>);
impl Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.0.lock().unwrap().write(buf) }
    fn flush(&mut self) -> std::io::Result<()> { self.0.lock().unwrap().flush() }
}

impl TerminalInstance {
    pub fn create(shell: &str, cwd: &str, cols: u16, rows: u16) -> Option<Self> {
        let pair = native_pty_system().openpty(PtySize {
            rows, cols, pixel_width: cols * 8, pixel_height: rows * 18,
        }).ok()?;

        let master = pair.master;
        let reader = master.try_clone_reader().ok()?;
        let raw_writer = master.take_writer().ok()?;
        let shared = Arc::new(Mutex::new(raw_writer));

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd).ok()?;

        let grid = Arc::new(Mutex::new(Grid::new(cols as usize, rows as usize)));
        let user_writer = SharedWriter(shared.clone());
        let terminal_writer: Box<dyn Write + Send> = Box::new(SharedWriter(shared));

        // Reader thread: PTY -> vte parser -> grid
        let g = grid.clone();
        let w = terminal_writer;
        std::thread::spawn(move || {
            let mut buf = [0u8; 65536];
            let mut reader = reader;
            let mut vte_parser = VteParser::new();
            let mut handler = parser::TerminalHandler::new(g, w);
            loop {
                let n = match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(_) => break,
                };
                vte_parser.advance(&mut handler, &buf[..n]);
            }
        });

        let cwd_str = cwd.to_string();
        Some(TerminalInstance {
            grid,
            writer: Box::new(user_writer),
            master,
            _child: child,
            name: String::new(),
            font_size: 14.0,
            working_directory: cwd_str,
            cwd_file: PathBuf::new(),
            restored_snapshot: None,
            history_nav: None,
            screen_cols: cols as usize,
            screen_rows: rows as usize,
        })
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if cols as usize == self.screen_cols && rows as usize == self.screen_rows {
            return;
        }
        let _ = self.master.resize(PtySize {
            rows, cols, pixel_width: cols * 8, pixel_height: rows * 18,
        });
        if let Ok(mut g) = self.grid.lock() {
            g.resize(cols as usize, rows as usize);
            g.clear_screen(2);
            g.cursor_col = 0;
            g.cursor_row = 0;
        }
        self.screen_cols = cols as usize;
        self.screen_rows = rows as usize;
    }

    pub fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        if let Ok(g) = self.grid.lock() {
            (g.cursor_col, g.cursor_row)
        } else { (0, 0) }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.screen_cols, self.screen_rows)
    }

    pub fn get_current_line(&self) -> String {
        let Ok(g) = self.grid.lock() else { return String::new() };
        let row = g.cursor_row;
        g.row_text(row)
    }
}
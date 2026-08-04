pub mod adapter;

pub use adapter::ShellInfo;

use egui_term::{BackendSettings, TerminalBackend};

pub struct TerminalInstance {
    pub backend: TerminalBackend,
    pub cwd: String,
    pub shell_info: adapter::ShellInfo,
    pub history_nav: Option<crate::app::HistoryNav>,
}

impl TerminalInstance {
    pub fn create(
        ctx: &egui::Context,
        id: u64,
        shell: &str,
        cwd: &str,
        _cols: u16,
        _rows: u16,
    ) -> Option<Self> {
        let settings = BackendSettings {
            shell: shell.to_string(),
            args: vec![],
            working_directory: Some(std::path::PathBuf::from(cwd)),
        };

        let backend = TerminalBackend::new(id, ctx.clone(), settings).ok()?;

        let shell_info = adapter::ShellInfo::new();
        if let Ok(mut cwd_guard) = shell_info.cwd.lock() {
            *cwd_guard = cwd.to_string();
        }

        Some(TerminalInstance {
            backend,
            cwd: cwd.to_string(),
            shell_info,
            history_nav: None,
        })
    }

    pub fn resize(&mut self, _cols: u16, _rows: u16) {}

    pub fn write(&mut self, data: &[u8]) {
        self.backend
            .process_command(egui_term::BackendCommand::Write(data.to_vec()));
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        let content = self.backend.last_content();
        let cursor = &content.grid.cursor;
        (cursor.point.column.0 as usize, cursor.point.line.0 as usize)
    }

    pub fn size(&self) -> (usize, usize) {
        let content = self.backend.last_content();
        use alacritty_terminal::grid::Dimensions;
        (content.grid.columns(), content.grid.screen_lines())
    }

    pub fn get_current_line(&mut self) -> String {
        use alacritty_terminal::grid::Dimensions;
        // Force sync so we read the latest grid content
        self.backend.set_dirty();
        let content = self.backend.sync();
        let grid = &content.grid;
        let cursor_point = content.grid.cursor.point;
        let line = cursor_point.line;
        let mut text = String::new();
        for col in 0..grid.columns() {
            let point = alacritty_terminal::index::Point {
                line,
                column: alacritty_terminal::index::Column(col),
            };
            let cell = &grid[point];
            text.push(cell.c);
        }
        text
    }

    pub fn poll_cwd(&mut self) {
        #[cfg(not(unix))]
        return;

        #[cfg(unix)]
        let proc_path = format!("/proc/{}/cwd", self.backend.child_pid());

        #[cfg(unix)]
        if let Ok(path) = std::fs::read_link(proc_path) {
            let cwd_str = path.to_string_lossy().to_string();
            if self.cwd != cwd_str {
                self.cwd = cwd_str.clone();
                if let Ok(mut cwd_guard) = self.shell_info.cwd.lock() {
                    *cwd_guard = cwd_str;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalInstance;

    #[test]
    fn poll_cwd_reads_the_shell_process_directory() {
        let mut instance =
            TerminalInstance::create(&egui::Context::default(), 1, "/bin/sh", "/tmp", 80, 24)
                .expect("shell should start");

        instance.write(b"cd /\r");
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.poll_cwd();
            if instance.cwd == "/" {
                break;
            }
        }

        assert_eq!(instance.cwd, "/");
    }
}

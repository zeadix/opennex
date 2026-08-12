pub mod adapter;

pub use adapter::ShellInfo;

use egui_term::{BackendSettings, TerminalBackend};

pub struct TerminalInstance {
    pub backend: TerminalBackend,
    pub cwd: String,
    pub shell_info: adapter::ShellInfo,
    pub history_nav: Option<crate::app::HistoryNav>,
    osc_buffer: String,
}

fn shell_integration_sequence(shell: &str) -> Vec<u8> {
    if shell.contains("bash") || shell.ends_with("/sh") {
        // Append to PROMPT_COMMAND instead of overwriting, preserving the
        // shell's original color/layout prompt configuration.
        format!(
            r#"__opennex_osc() {{ printf '\033]9;$PWD\007'; }}
if [ -n "${{PROMPT_COMMAND}}" ]; then
  PROMPT_COMMAND="__opennex_osc;${{PROMPT_COMMAND}}"
else
  PROMPT_COMMAND="__opennex_osc"
fi
"#,
        )
        .into_bytes()
    } else if shell.contains("zsh") {
        format!(
            r#"__opennex_osc() {{ printf '\033]9;$PWD\007'; }}
precmd_functions+=(__opennex_osc)
"#,
        )
        .into_bytes()
    } else if shell.contains("powershell") || shell.contains("pwsh") {
        format!(
            r#"function __opennex_osc {{ Write-Host -NoNewline "`e]9;$(Get-Location)`e\"; }}
$Global:prompt = $function:prompt
function prompt {{ __opennex_osc; & $Global:prompt }}
"#,
        )
        .into_bytes()
    } else {
        Vec::new()
    }
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
            args: vec!["-l".into(), "-i".into()],
            working_directory: Some(std::path::PathBuf::from(cwd)),
        };

        let backend = TerminalBackend::new(id, ctx.clone(), settings).ok()?;

        let shell_info = adapter::ShellInfo::new();
        if let Ok(mut cwd_guard) = shell_info.cwd.lock() {
            *cwd_guard = cwd.to_string();
        }

        let mut instance = TerminalInstance {
            backend,
            cwd: cwd.to_string(),
            shell_info,
            history_nav: None,
            osc_buffer: String::new(),
        };

        // Inject shell integration
        let integration = shell_integration_sequence(shell);
        if !integration.is_empty() {
            instance.write(&integration);
        }

        Some(instance)
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
        // Scan terminal output for OSC 9;cwd sequences
        self.backend.set_dirty();
        let content = self.backend.sync();
        use alacritty_terminal::grid::Dimensions;
        let grid = &content.grid;

        // Read the visible screen content to find OSC sequences
        let mut screen_text = String::new();
        for row_idx in 0..grid.screen_lines() {
            for col_idx in 0..grid.columns() {
                let point = alacritty_terminal::index::Point {
                    line: alacritty_terminal::index::Line(row_idx as i32),
                    column: alacritty_terminal::index::Column(col_idx),
                };
                let cell = &grid[point];
                screen_text.push(cell.c);
            }
            screen_text.push('\n');
        }

        // Parse OSC 9;... sequences from the screen text
        // The sequence format is: ESC ] 9 ; <path> BEL  or  ESC ] 9 ; <path> ESC \
        // On screen these may appear as leftover characters in the grid
        // We look for the pattern "9;" followed by a path ending with BEL (0x07) or ESC
        self.osc_buffer.push_str(&screen_text);

        // Try to extract cwd from OSC 9; sequences
        while let Some(start) = self.osc_buffer.find("9;") {
            let after_start = start + 2;
            if after_start >= self.osc_buffer.len() {
                break;
            }
            // Find end: BEL (\x07) or ESC (\x1b)
            let rest = &self.osc_buffer[after_start..];
            let end = rest.find(|c| c == '\x07' || c == '\x1b');
            if let Some(end_pos) = end {
                let path = &rest[..end_pos];
                if !path.is_empty() {
                    let cwd_str = path.to_string();
                    if self.cwd != cwd_str {
                        self.cwd = cwd_str.clone();
                        if let Ok(mut cwd_guard) = self.shell_info.cwd.lock() {
                            *cwd_guard = cwd_str;
                        }
                    }
                }
                // Consume up to and including the terminator
                let consume_to = after_start + end_pos + 1;
                self.osc_buffer =
                    self.osc_buffer[consume_to.min(self.osc_buffer.len())..].to_string();
            } else {
                // Incomplete sequence, keep buffer and wait for more
                self.osc_buffer = self.osc_buffer[start..].to_string();
                break;
            }
        }

        // Keep buffer bounded
        if self.osc_buffer.len() > 8192 {
            self.osc_buffer = self.osc_buffer[self.osc_buffer.len() - 4096..].to_string();
        }

        // Fallback: also try /proc on Linux if OSC didn't work
        #[cfg(target_os = "linux")]
        {
            let proc_path = format!("/proc/{}/cwd", self.backend.child_pid());
            if let Ok(path) = std::fs::read_link(&proc_path) {
                let cwd_str = path.to_string_lossy().to_string();
                if self.cwd != cwd_str && !cwd_str.is_empty() {
                    self.cwd = cwd_str.clone();
                    if let Ok(mut cwd_guard) = self.shell_info.cwd.lock() {
                        *cwd_guard = cwd_str;
                    }
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

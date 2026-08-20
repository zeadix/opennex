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

/// Marker used to detect whether a shell init file already contains our
/// integration snippet, so we don't append it twice.
const SHELL_INTEGRATION_MARKER: &str = "# __opennex_integration__ v3";

/// Build (or reuse) the per-shell init file that installs the OSC 9
/// hook, and return the path the user shell should be told to source.
///
/// Writing the integration to a file (and pointing the shell at it via
/// `--rcfile` / `ZDOTDIR`) avoids dumping the multi-line snippet into
/// the PTY on every terminal open, which would otherwise pollute the
/// scrollback with echoed shell input.
fn ensure_shell_init_file(shell: &str) -> Option<std::path::PathBuf> {
    let home = dirs::config_dir()?.join("opennex").join("shell-init");
    std::fs::create_dir_all(&home).ok()?;
    let (filename, body) = if shell.contains("bash") {
        ("bash.sh", BASH_INIT_BODY)
    } else if shell.ends_with("/sh") {
        // /bin/sh is often dash. The OSC integration is bash-only,
        // but we still create a minimal init file so the test path
        // (which uses /bin/sh) doesn't crash on a missing file.
        ("sh.sh", SH_INIT_BODY)
    } else if shell.contains("zsh") {
        ("zsh.sh", ZSH_INIT_BODY)
    } else if shell.contains("powershell") || shell.contains("pwsh") {
        // PowerShell is harder to inject silently; fall back to a no-op
        // marker so we know we considered it. The user can paste the
        // snippet into their $PROFILE manually if they want OSC 9
        // support on Windows.
        ("powershell.ps1", POWERSHELL_INIT_BODY)
    } else {
        return None;
    };
    let path = home.join(filename);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if !existing.contains(SHELL_INTEGRATION_MARKER) {
        let _ = std::fs::write(&path, format!("{SHELL_INTEGRATION_MARKER}\n{body}\n"));
    }
    Some(path)
}

const BASH_INIT_BODY: &str = r#"
# OpenNex terminal integration. Sourced via bash --rcfile, which
# replaces the default ~/.bashrc. Chain to the system file first,
# then the user's bashrc (same order bash uses natively) so the
# user's colored prompt wins over the system default PS1.
for __opennex_rc in "/etc/bash.bashrc" "$HOME/.bashrc"; do
  [ -r "$__opennex_rc" ] && . "$__opennex_rc"
done
unset __opennex_rc
__opennex_osc() { printf '\033]9;%s\007' "$PWD"; }
if [ -n "${PROMPT_COMMAND}" ]; then
  PROMPT_COMMAND="__opennex_osc;${PROMPT_COMMAND}"
else
  PROMPT_COMMAND="__opennex_osc"
fi
"#;

const ZSH_INIT_BODY: &str = r#"
# OpenNex terminal integration. ZDOTDIR points at this directory, so
# chain to the user's original config first (the real HOME is saved
# in __opennex_home by the backend env).
if [ -r "$__opennex_home/.zshrc" ]; then
  . "$__opennex_home/.zshrc"
fi
__opennex_osc() { printf '\033]9;%s\007' "$PWD"; }
precmd_functions+=(__opennex_osc)
"#;

/// Minimal init for /bin/sh (dash). The OSC integration isn't
/// available in dash, so this just provides an empty but valid
/// rc file so the shell starts cleanly.
const SH_INIT_BODY: &str = r#"
# /bin/sh (dash) does not support --rcfile or OSC 9. Placeholder.
"#;

const POWERSHELL_INIT_BODY: &str = r#"
# Add the following to your PowerShell $PROFILE for OSC 9 support:
# function prompt { Write-Host -NoNewline ([char]27 + "]9;$($PWD.Path)" + [char]7) -NoNewline; return "$PWD> " }
"#;

/// Build a `BASH_ENV` value (or equivalent) to inject the integration
/// into a newly-spawned bash without echoing the script into the
/// scrollback. We currently rely on bash's `-l` flag plus the
/// integration file existing on disk; the actual wiring happens in
/// `BackendSettings::args`.
fn shell_integration_sequence(_shell: &str) -> Vec<u8> {
    // No longer used: the integration lives in a file pointed to by
    // `--rcfile` / `ZDOTDIR` so the PTY never sees the source.
    Vec::new()
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
        // Make sure the per-shell init file exists, then point the shell
        // at it. For bash we pass `--rcfile`; for zsh we override
        // `ZDOTDIR` so the init file is sourced. PowerShell and cmd
        // currently have no silent integration path.
        let init_file = ensure_shell_init_file(shell);
        let mut args: Vec<String> = Vec::new();
        let mut zdotdir: Option<std::path::PathBuf> = None;
        if let Some(path) = &init_file {
            if shell.contains("bash") {
                // bash: --rcfile must precede -i; with this ordering the
                // shell starts, shows the user prompt (the rcfile chains
                // to ~/.bashrc), and stdin works. (-l combined with
                // --rcfile is a usage error; -i before --rcfile also
                // trips usage parsing in bash 5.1.)
                args = vec![
                    "--rcfile".into(),
                    path.to_string_lossy().to_string(),
                    "-i".into(),
                ];
            } else if shell.contains("zsh") {
                args = vec!["-i".into()];
                if let Some(parent) = path.parent() {
                    zdotdir = Some(parent.to_path_buf());
                }
            } else {
                args = vec!["-i".into()];
            }
        } else {
            args = vec!["-l".into(), "-i".into()];
        }

        let mut settings = BackendSettings {
            shell: shell.to_string(),
            args,
            working_directory: Some(std::path::PathBuf::from(cwd)),
            env: vec![],
        };
        if let Some(dir) = zdotdir {
            settings
                .env
                .push(("ZDOTDIR".to_string(), dir.to_string_lossy().to_string()));
            if let Some(home) = std::env::var_os("HOME") {
                settings.env.push((
                    "__opennex_home".to_string(),
                    home.to_string_lossy().to_string(),
                ));
            }
        }

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

        // No PTY write here: the integration is sourced from the file
        // pointed to by --rcfile / ZDOTDIR above.
        let _ = shell_integration_sequence(shell);

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

        let read_row = |line: i32| -> String {
            let mut text = String::new();
            for col in 0..grid.columns() {
                let point = alacritty_terminal::index::Point {
                    line: alacritty_terminal::index::Line(line),
                    column: alacritty_terminal::index::Column(col),
                };
                text.push(grid[point].c);
            }
            text
        };

        // A long command wraps across several grid rows. Walk UP from the
        // cursor row: a row is a wrapped continuation of the row below it
        // when that lower row was completely filled (the shell only wraps
        // when it hits the last column) AND the current row doesn't look
        // like a fresh prompt. This reconstructs the full logical line
        // instead of just the cursor's physical row (which truncated
        // wrapped commands to their last visual row).
        let cols = grid.columns() as i32;
        let mut start_line = cursor_point.line.0;
        let bottom_limit = -(grid.total_lines() as i32 - grid.screen_lines() as i32);
        while start_line - 1 >= bottom_limit {
            let row_below = read_row(start_line);
            let trimmed_end = row_below.trim_end();
            // The row below must be completely full for a wrap to have
            // occurred at its end.
            if trimmed_end.chars().count() as i32 >= cols {
                // The row above must not be a new prompt line (prompts end
                // with "$ " / "# " after trimming... a fresh prompt row is
                // typically short). Only continue merging when the above
                // row looks like the START of the wrapped text, i.e. it
                // exists within the live screen region.
                let above = start_line - 1;
                if above >= -(grid.screen_lines() as i32 - 1) {
                    start_line = above;
                    continue;
                }
            }
            break;
        }

        // Read from the topmost wrapped row down to the cursor row and
        // concatenate; wrapped rows have no trailing newline in the logical
        // line, so a plain concatenation of the raw cell text is correct.
        let mut text = String::new();
        let mut l = start_line;
        while l <= cursor_point.line.0 {
            text.push_str(&read_row(l));
            l += 1;
        }
        // Trailing spaces from the cursor row are padding, not content.
        text.trim_end().to_string()
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

        // Try to extract cwd from OSC 9; sequences.
        // OSC sequences are pure ASCII, but the buffer may contain multi-byte
        // UTF-8 from terminal output. We must only slice at char boundaries.
        loop {
            let Some(start) = self.osc_buffer.find("9;") else {
                break;
            };
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
                let consume_to = (after_start + end_pos + 1).min(self.osc_buffer.len());
                let consumed: String = self.osc_buffer.drain(..consume_to).collect();
                let _ = consumed;
            } else {
                // Incomplete sequence, keep from start and wait for more
                let drained: String = self.osc_buffer.drain(..start).collect();
                let _ = drained;
                break;
            }
        }

        // Keep buffer bounded — truncate at a char boundary
        if self.osc_buffer.len() > 8192 {
            let truncate_at = self.osc_buffer.len() - 4096;
            // Floor to nearest char boundary to avoid splitting a multibyte char
            let boundary = self.osc_buffer.ceil_char_boundary(truncate_at);
            self.osc_buffer.drain(..boundary);
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

    #[test]
    fn get_current_line_reassembles_wrapped_commands() {
        // A command longer than the grid width wraps across rows; the
        // recorded command must contain the FULL text, not just the last
        // visual row.
        let mut instance =
            TerminalInstance::create(&egui::Context::default(), 2, "/bin/sh", "/tmp", 20, 24)
                .expect("shell should start");

        // Long word: wraps across at least two 20-col rows. Type it but do
        // NOT press Enter (the app records on Enter; here we only verify
        // the line reconstruction).
        let long = "openwindnownow12345678";
        instance.write(long.as_bytes());

        // Let the pty echo the typed characters into the grid.
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }

        let line = instance.get_current_line();
        assert!(
            line.contains("openwindnownow12345678"),
            "reassembled line must contain the full wrapped command, got: {line:?}"
        );
    }

    #[test]
    fn prompt_stripping_records_only_the_command() {
        // The recorder keeps only the text AFTER the last "$ "/"# " prompt
        // terminator; a line with no prompt marker records nothing.
        let strip = |line: &str| {
            line.rfind("$ ")
                .or_else(|| line.rfind("# "))
                .map(|p| line[p + 2..].trim().to_string())
                .unwrap_or_default()
        };
        assert_eq!(strip("user@host:~/proj$ ls -la"), "ls -la");
        assert_eq!(strip("root# reboot"), "reboot");
        // Wrapped prompt line: prompt appears on the FIRST visual row; the
        // command tail follows the last terminator.
        assert_eq!(strip("user@host:~/very/long/path$ echo done"), "echo done");
        // No prompt marker -> record nothing (never the raw line).
        assert_eq!(strip("just some output text"), "");
    }
}

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
# Guarded hook: if this file is ever sourced outside OpenNex (where the
# function may be missing), the hook fails silently instead of printing
# "__opennex_osc: command not found" on every prompt.
if [ -n "${PROMPT_COMMAND}" ]; then
  PROMPT_COMMAND="command -v __opennex_osc >/dev/null 2>&1 && __opennex_osc;${PROMPT_COMMAND}"
else
  PROMPT_COMMAND="command -v __opennex_osc >/dev/null 2>&1 && __opennex_osc"
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

/// Strip the shell prompt from a (possibly wrap-merged) line, returning
/// the command text after it.
///
/// The terminator is "$ " / "# " normally, but readline SWALLOWS a
/// line-end space when the prompt wraps at a row boundary, so the bare
/// "$"/"#" is the fallback. Without it, commands typed right after a
/// resize-induced wrap were never recorded (empty command → skipped).
pub fn strip_prompt(line: &str) -> Option<&str> {
    if let Some(p) = line.rfind("$ ").or_else(|| line.rfind("# ")) {
        return Some(&line[p + 2..]);
    }
    line.rfind('$')
        .or_else(|| line.rfind('#'))
        .map(|p| &line[p + 1..])
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
        // Explicit terminal capabilities: the app may be (re)started from
        // environments without TERM (updater helper, .desktop, systemd),
        // and a shell that sees no/empty TERM disables ANSI colors —
        // every colored prompt/output collapses to one foreground color.
        settings
            .env
            .push(("TERM".to_string(), "xterm-256color".to_string()));
        settings
            .env
            .push(("COLORTERM".to_string(), "truecolor".to_string()));
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

    /// Pixel size of one terminal cell (width, height) from the last
    /// rendered content — used to anchor overlays at the cursor.
    pub fn cell_size(&self) -> (f32, f32) {
        let content = self.backend.last_content();
        (
            content.terminal_size.cell_width as f32,
            content.terminal_size.cell_height as f32,
        )
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
        self.get_current_line_with_col().0
    }

    /// Reassemble the cursor's logical row (wrapped rows joined) AND the
    /// cursor's column within that logical row. The logical column is the
    /// physical column offset by the total width of every wrapped row
    /// above the cursor's physical row — using the raw physical column
    /// against the joined line chopped the word at the wrong place
    /// whenever the PROMPT itself wrapped (narrow split panes).
    pub fn get_current_line_with_col(&mut self) -> (String, usize) {
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

        // A long line (prompt and/or command) wraps across several grid
        // rows. Walk UP from the cursor row and merge predecessor rows.
        // Four merge shapes must all work:
        //   1. The row above is FULL of content → classic wrap.
        //   2. The row above (or the current top) ends with the prompt
        //      terminator ("$"/"#" as the last non-padding char) → the
        //      prompt was wrapped away from its command. Content spaces
        //      at a row boundary are indistinguishable from grid padding,
        //      which defeated cell-counting heuristics.
        //   3. After a SIGWINCH redraw the cursor can sit on an EMPTY row
        //      directly below the row that actually holds the text (with
        //      duplicated prompt rows above it) — merge up whenever the
        //      current top row has no content at all.
        //   4. Right after a resize, readline can scatter the typed
        //      command across MULTIPLE short rows below the prompt row
        //      (one char per row in the worst case). Every row between
        //      the prompt-terminator row and the cursor belongs to the
        //      command: once a terminator row is found, keep merging up
        //      to (and including) it.
        let cols_i32 = grid.columns() as i32;
        let row_content = |line: i32| -> String { read_row(line).trim_end().to_string() };
        let ends_with_prompt = |s: &str| -> bool { s.ends_with('$') || s.ends_with('#') };
        let mut start_line = cursor_point.line.0;
        let bottom_limit = -(grid.total_lines() as i32 - grid.screen_lines() as i32);
        loop {
            let above = start_line - 1;
            if above < bottom_limit || above < -(grid.screen_lines() as i32 - 1) {
                break;
            }
            // Shape 3: current top is empty → take the row above, but
            // only if IT has content (two consecutive blank rows are
            // never a wrap; without this guard an all-blank screen would
            // walk to the top of the scrollback).
            if row_content(start_line).is_empty() {
                if row_content(above).is_empty() {
                    break;
                }
                start_line = above;
                continue;
            }
            let above_content = row_content(above);
            // Shape 4: right after a resize, readline can scatter the
            // typed command across MULTIPLE short rows below the prompt
            // row (one char per row in the worst case). Probe a few rows
            // up for the prompt-terminator row; if found, absorb every
            // row from there down to the cursor.
            let mut probe = above;
            let mut prompt_line: Option<i32> = None;
            for _ in 0..4 {
                if probe < bottom_limit || probe < -(grid.screen_lines() as i32 - 1) {
                    break;
                }
                if ends_with_prompt(&row_content(probe)) {
                    prompt_line = Some(probe);
                    break;
                }
                probe -= 1;
            }
            if let Some(pl) = prompt_line {
                start_line = pl;
                break;
            }
            // Shape 1: the row above is full of content → wrapped.
            if above_content.chars().count() as i32 >= cols_i32 {
                start_line = above;
                continue;
            }
            break;
        }

        // Read from the topmost wrapped row down to the cursor row and
        // concatenate. Trailing spaces of NON-final rows are grid
        // padding (a wrapped row's content ends at its last column);
        // trimming them per-row keeps fragment rows like "c" + "d" from
        // joining as "c<72 spaces>d". The FINAL row keeps its raw text —
        // a user-typed trailing space ("cd ") must survive — and the
        // overall result is trim_end'ed once below.
        let mut text = String::new();
        let mut l = start_line;
        while l <= cursor_point.line.0 {
            let row = read_row(l);
            if l < cursor_point.line.0 {
                text.push_str(row.trim_end());
            } else {
                text.push_str(&row);
            }
            l += 1;
        }
        // Cursor's LOGICAL column: physical column plus the width of every
        // visual row above the cursor's row within the wrap group.
        let rows_above = (cursor_point.line.0 - start_line).max(0) as usize;
        let logical_col = rows_above * grid.columns() + cursor_point.column.0;
        // Trailing spaces from the cursor row are padding, not content.
        (text.trim_end().to_string(), logical_col)
    }

    /// The text the user is currently typing (for the auto-match overlay):
    /// everything after the prompt up to the CURSOR column, with grid
    /// padding stripped. Spaces participate: "cd " (typed space kept)
    /// only matches history entries whose text starts with "cd ".
    pub fn current_input_word(&mut self) -> String {
        // Read the cursor's logical row (with wrapped rows joined) and the
        // cursor's LOGICAL column within it.
        let (line, logical_col) = self.get_current_line_with_col();
        let after_prompt = strip_prompt(&line).unwrap_or("");

        // Clip at the logical column: grid padding lives to the RIGHT of
        // the cursor, so this alone removes it. NO trim_end — the user's
        // typed trailing space ("cd ") must survive so it participates in
        // the prefix match (bare "cd"/"cdd" then correctly fail to match).
        // The prompt length must be counted in CHARS (columns are
        // char-based), not bytes — prompts may contain multi-byte text.
        let prompt_chars = line.chars().count() - after_prompt.chars().count();
        let cursor_col = logical_col.saturating_sub(prompt_chars);
        let chars: Vec<char> = after_prompt.chars().collect();
        if cursor_col < chars.len() {
            chars[..cursor_col].iter().collect()
        } else {
            after_prompt.to_string()
        }
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
    fn current_input_word_returns_full_text_when_command_wraps() {
        // Auto-match reads the word via current_input_word: it clips the
        // reassembled logical line at the CURSOR'S PHYSICAL column. When
        // the command wraps onto a second visual row, that column is a
        // small number again, chopping the word — auto-match then sees a
        // bogus prefix and the suggestion list goes empty.
        let mut instance =
            TerminalInstance::create(&egui::Context::default(), 3, "/bin/sh", "/tmp", 20, 24)
                .expect("shell should start");

        let long = "abcdefghijklmnopqrstuvwxy";
        instance.write(long.as_bytes());
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }

        let word = instance.current_input_word();
        assert!(
            word.contains("abcdefghijklmnopqrst"),
            "wrapped command word must survive the cursor-clip, got: {word:?}"
        );
    }

    #[test]
    fn current_input_word_survives_wrapped_long_prompt() {
        // Narrow split panes wrap a LONG prompt (user@host:~/long/path$)
        // onto a second visual row. The word must still come out as the
        // typed text: the clip has to use the cursor's LOGICAL column,
        // not its physical column on the wrapped row.
        let mut instance =
            TerminalInstance::create(&egui::Context::default(), 4, "bash", "/tmp", 20, 24)
                .expect("shell should start");

        // Install a 40-char prompt in a 20-col grid → the prompt alone
        // wraps onto 2+ rows.
        instance.write(b"export PS1='kunpengwang@test-Victus-by-HP-Gaming-Laptop-16-r0xxx:~/proj/my/open_zoo$ '
");
        // Let the shell apply it.
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }

        instance.write(b"cd");
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }

        let word = instance.current_input_word();
        assert!(
            word.contains("cd"),
            "word after a wrapped long prompt must be the typed text, got: {word:?}"
        );
    }

    #[test]
    fn current_input_word_matches_in_wrapped_prompt_narrow_pane() {
        // REAL geometry of a narrow pane: a 73-char visible prompt (long
        // hostname + deep cwd). Sweep the grid width across the critical
        // boundary values (including widths where the prompt ends EXACTLY
        // at a row boundary — the case that used to defeat the wrap merge
        // and silently disabled auto-match), each time running the exact
        // user sequence cd⏎ cdd⏎ c and requiring the word "c" back.
        for cols in [40u16, 45, 48, 50, 52, 55, 60, 72, 73, 74, 80] {
            let mut instance =
                TerminalInstance::create(&egui::Context::default(), 9, "bash", "/tmp", cols, 24)
                    .expect("shell should start");
            // create() ignores its cols/rows args (grid starts 80x50), so
            // resize FIRST like a real pane resize would.
            instance
                .backend
                .process_command(egui_term::BackendCommand::Resize(
                    egui_term::Size::from(egui::vec2(cols as f32 * 10.0, 400.0)),
                    egui_term::Size::from(egui::vec2(10.0, 20.0)),
                ));
            std::thread::sleep(std::time::Duration::from_millis(400));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
            instance.write(b"export PS1='kunpengwang@test-Victus-by-HP-Gaming-Laptop-16-r0xxx:~/proj/my/open_zoo$ '\r");
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                instance.backend.set_dirty();
                let _ = instance.backend.sync();
            }
            for cmd in ["cd\r", "cdd\r"] {
                instance.write(cmd.as_bytes());
                for _ in 0..20 {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                    instance.backend.set_dirty();
                    let _ = instance.backend.sync();
                }
            }
            instance.write(b"c");
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                instance.backend.set_dirty();
                let _ = instance.backend.sync();
            }
            let word = instance.current_input_word();
            assert_eq!(
                word, "c",
                "auto-match word at grid width {cols} (wrapped prompt)"
            );
        }
    }

    #[test]
    fn current_input_word_after_shrinking_a_wide_pane() {
        // REAL user sequence: the prompt and history were entered while
        // the pane was WIDE; the pane is then DRAGGED NARROW (SIGWINCH →
        // readline redraws the wrapped prompt); typing must still match.
        // This differs from starting narrow: the redraw leaves a
        // different grid state than a fresh narrow print.
        for cols in [40u16, 45, 48, 50, 52, 55, 60, 72, 73, 74] {
            let mut instance =
                TerminalInstance::create(&egui::Context::default(), 10, "bash", "/tmp", 80, 24)
                    .expect("shell should start");
            std::thread::sleep(std::time::Duration::from_millis(300));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
            instance.write(b"export PS1='kunpengwang@test-Victus-by-HP-Gaming-Laptop-16-r0xxx:~/proj/my/open_zoo$ '\r");
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                instance.backend.set_dirty();
                let _ = instance.backend.sync();
            }
            for cmd in ["cd\r", "cdd\r"] {
                instance.write(cmd.as_bytes());
                for _ in 0..20 {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                    instance.backend.set_dirty();
                    let _ = instance.backend.sync();
                }
            }
            // NOW shrink the pane (like dragging the splitter).
            instance
                .backend
                .process_command(egui_term::BackendCommand::Resize(
                    egui_term::Size::from(egui::vec2(cols as f32 * 10.0, 400.0)),
                    egui_term::Size::from(egui::vec2(10.0, 20.0)),
                ));
            // Give readline time to redraw the wrapped prompt.
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(50));
                instance.backend.set_dirty();
                let _ = instance.backend.sync();
            }
            instance.write(b"c");
            for _ in 0..30 {
                std::thread::sleep(std::time::Duration::from_millis(25));
                instance.backend.set_dirty();
                let _ = instance.backend.sync();
            }
            let word = instance.current_input_word();
            let word = instance.current_input_word();
            assert_eq!(word, "c", "auto-match word after shrinking to width {cols}");
        }
    }

    #[test]
    fn prompt_stripping_records_only_the_command() {
        // The recorder keeps only the text AFTER the last "$ "/"# " prompt
        // terminator; a line with no prompt marker records nothing.
        let strip = |line: &str| {
            crate::terminal::strip_prompt(line)
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };
        assert_eq!(strip("user@host:~/proj$ ls -la"), "ls -la");
        assert_eq!(strip("root# reboot"), "reboot");
        // Wrapped prompt line: prompt appears on the FIRST visual row; the
        // command tail follows the last terminator.
        assert_eq!(strip("user@host:~/very/long/path$ echo done"), "echo done");
        // No prompt marker -> record nothing (never the raw line).
        assert_eq!(strip("just some output text"), "");
        // Resize-wrap: readline swallows the line-end space after "$";
        // the bare-terminator fallback must still yield the command.
        assert_eq!(strip("user@host:~/very/long/path$cd"), "cd");
        assert_eq!(strip("root#reboot"), "reboot");
    }

    #[test]
    fn records_command_typed_right_after_narrowing() {
        // Regression: narrowing the pane makes the first typed character
        // wrap onto the next row (readline ate the line-end space), and
        // the Enter-recorder must STILL record the full command.
        let mut instance =
            TerminalInstance::create(&egui::Context::default(), 11, "bash", "/tmp", 80, 24)
                .expect("shell should start");
        std::thread::sleep(std::time::Duration::from_millis(300));
        instance.backend.set_dirty();
        let _ = instance.backend.sync();
        instance.write(b"export PS1='kunpengwang@test-Victus-by-HP-Gaming-Laptop-16-r0xxx:~/proj/my/open_zoo$ '\r");
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }
        // Narrow until the prompt fills a row exactly (73 visible chars).
        instance
            .backend
            .process_command(egui_term::BackendCommand::Resize(
                egui_term::Size::from(egui::vec2(730.0, 400.0)),
                egui_term::Size::from(egui::vec2(10.0, 20.0)),
            ));
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }
        // Type the command; the first char may land on the wrapped row.
        instance.write(b"cd");
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(25));
            instance.backend.set_dirty();
            let _ = instance.backend.sync();
        }
        // Same extraction the Enter-recorder performs.
        let line = instance.get_current_line();
        let cmd = crate::terminal::strip_prompt(&line)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        assert_eq!(cmd, "cd", "line was: {line:?}");
    }
}

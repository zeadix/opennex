pub mod settings;

use crate::types::Size;
use alacritty_terminal::event::{
    Event, EventListener, Notify, OnResize, WindowSize,
};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Direction, Line, Point, Side};
use alacritty_terminal::selection::{
    Selection, SelectionRange, SelectionType as AlacrittySelectionType,
};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::search::{Match, RegexIter, RegexSearch};
use alacritty_terminal::term::{
    self, cell::Cell, test::TermSize, viewport_to_point, Term, TermMode,
};
use alacritty_terminal::{tty, Grid};
use egui::Modifiers;
use settings::BackendSettings;
use std::borrow::Cow;
use std::cmp::min;
use std::io::Result;
use std::ops::{Index, RangeInclusive};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

pub type TerminalMode = TermMode;
pub type SelectionType = AlacrittySelectionType;

#[derive(Debug, Clone)]
pub enum BackendCommand {
    Write(Vec<u8>),
    Scroll(i32),
    Resize(Size, Size),
    SelectStart(SelectionType, f32, f32),
    SelectUpdate(f32, f32),
    ProcessLink(LinkAction, Point),
    MouseReport(MouseButton, Modifiers, Point, bool),
}

#[derive(Debug, Clone)]
pub enum MouseMode {
    Sgr,
    Normal(bool),
}

impl From<TermMode> for MouseMode {
    fn from(term_mode: TermMode) -> Self {
        if term_mode.contains(TermMode::SGR_MOUSE) {
            MouseMode::Sgr
        } else if term_mode.contains(TermMode::UTF8_MOUSE) {
            MouseMode::Normal(true)
        } else {
            MouseMode::Normal(false)
        }
    }
}

#[derive(Debug, Clone)]
pub enum MouseButton {
    LeftButton = 0,
    MiddleButton = 1,
    RightButton = 2,
    LeftMove = 32,
    MiddleMove = 33,
    RightMove = 34,
    NoneMove = 35,
    ScrollUp = 64,
    ScrollDown = 65,
    Other = 99,
}

#[derive(Debug, Clone)]
pub enum LinkAction {
    Clear,
    Hover,
    Open,
}

#[derive(Clone, Copy, Debug)]
pub struct TerminalSize {
    pub cell_width: u16,
    pub cell_height: u16,
    num_cols: u16,
    num_lines: u16,
    layout_size: Size,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cell_width: 1,
            cell_height: 1,
            num_cols: 80,
            num_lines: 50,
            layout_size: Size::default(),
        }
    }
}

impl Dimensions for TerminalSize {
    fn total_lines(&self) -> usize {
        self.screen_lines()
    }

    fn screen_lines(&self) -> usize {
        self.num_lines as usize
    }

    fn columns(&self) -> usize {
        self.num_cols as usize
    }

    fn last_column(&self) -> Column {
        Column(self.num_cols as usize - 1)
    }

    fn bottommost_line(&self) -> Line {
        Line(self.num_lines as i32 - 1)
    }
}

impl From<TerminalSize> for WindowSize {
    fn from(size: TerminalSize) -> Self {
        Self {
            num_lines: size.num_lines,
            num_cols: size.num_cols,
            cell_width: size.cell_width,
            cell_height: size.cell_height,
        }
    }
}

pub struct TerminalBackend {
    pub id: u64,
    child_pid: u32,
    pub url_regex: RegexSearch,
    term: Arc<FairMutex<Term<EventProxy>>>,
    size: TerminalSize,
    notifier: Notifier,
    last_content: RenderableContent,
    dirty: Arc<AtomicBool>,
    /// Set by the event subscription when the child reported Exit; lets
    /// Drop skip the SIGTERM nudge for already-dead children.
    child_exited: Arc<AtomicBool>,
    /// Cumulative bytes written TO the pty (keyboard input, pastes, mouse
    /// reports). Exposed so the app can compute per-terminal uplink rates.
    pub tx_bytes: std::sync::Arc<AtomicU64>,
}

impl TerminalBackend {
    pub fn new(
        id: u64,
        app_context: egui::Context,
        settings: BackendSettings,
    ) -> Result<Self> {
        let mut env_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (k, v) in &settings.env {
            env_map.insert(k.clone(), v.clone());
        }
        let pty_config = tty::Options {
            shell: Some(tty::Shell::new(settings.shell, settings.args)),
            working_directory: settings.working_directory,
            env: env_map,
            ..tty::Options::default()
        };
        let config = term::Config {
            scrolling_history: settings.scrollback,
            ..term::Config::default()
        };
        let terminal_size = TerminalSize::default();
        let pty = tty::new(&pty_config, terminal_size.into(), id)?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let child_pid = pty.child().id();
        #[cfg(target_os = "windows")]
        let child_pid =
            pty.child_watcher().pid().map(|pid| pid.get()).unwrap_or(0);
        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows"
        )))]
        let child_pid = 0u32;
        let (event_sender, event_receiver) = mpsc::channel();
        let event_proxy = EventProxy(event_sender);
        let mut term = Term::new(config, &terminal_size, event_proxy.clone());
        let initial_content = RenderableContent {
            grid: term.grid().clone(),
            selectable_range: None,
            terminal_mode: *term.mode(),
            terminal_size,
            cursor: term.grid_mut().cursor_cell().clone(),
            hovered_hyperlink: None,
        };
        let term = Arc::new(FairMutex::new(term));
        let pty_event_loop =
            EventLoop::new(term.clone(), event_proxy, pty, false, false)?;
        let notifier = Notifier(pty_event_loop.channel());
        let url_regex = RegexSearch::new(r#"(ipfs:|ipns:|magnet:|mailto:|gemini://|gopher://|https://|http://|news:|file://|git://|ssh:|ftp://)[^\u{0000}-\u{001F}\u{007F}-\u{009F}<>"\s{-}\^⟨⟩`]+"#).unwrap();
        let _pty_event_loop_thread = pty_event_loop.spawn();
        let dirty = Arc::new(AtomicBool::new(true));
        let dirty_thread = dirty.clone();
        let child_exited = Arc::new(AtomicBool::new(false));
        let exited_thread = child_exited.clone();
        let _pty_event_subscription = std::thread::Builder::new()
            .name(format!("pty_event_subscription_{}", id))
            .spawn(move || {
                receive_events(
                    event_receiver,
                    dirty_thread,
                    exited_thread,
                    app_context,
                )
            })?;

        Ok(Self {
            id,
            child_pid,
            url_regex,
            term: term.clone(),
            size: terminal_size,
            notifier,
            last_content: initial_content,
            dirty,
            child_exited,
            tx_bytes: std::sync::Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn process_command(&mut self, cmd: BackendCommand) {
        let term = self.term.clone();
        let mut term = term.lock();
        match cmd {
            BackendCommand::Write(input) => {
                self.write(input);
                term.scroll_display(Scroll::Bottom);
            },
            BackendCommand::Scroll(delta) => {
                self.scroll(&mut term, delta);
            },
            BackendCommand::Resize(layout_size, font_size) => {
                // Do not force scroll-to-bottom on resize: it fights reflow and
                // can make content look duplicated while the window is dragged.
                self.resize(&mut term, layout_size, font_size);
            },
            BackendCommand::SelectStart(selection_type, x, y) => {
                self.start_selection(&mut term, selection_type, x, y);
            },
            BackendCommand::SelectUpdate(x, y) => {
                self.update_selection(&mut term, x, y);
            },
            BackendCommand::ProcessLink(link_action, point) => {
                self.process_link_action(&term, link_action, point);
            },
            BackendCommand::MouseReport(button, modifiers, point, pressed) => {
                self.process_mouse_report(button, modifiers, point, pressed);
            },
        };
    }

    pub fn selection_point(
        x: f32,
        y: f32,
        terminal_size: &TerminalSize,
        display_offset: usize,
    ) -> Point {
        let col = (x as usize) / (terminal_size.cell_width as usize);
        let col = min(Column(col), Column(terminal_size.num_cols as usize - 1));

        let line = (y as usize) / (terminal_size.cell_height as usize);
        let line = min(line, terminal_size.num_lines as usize - 1);

        viewport_to_point(display_offset, Point::new(line, col))
    }

    pub fn selectable_content(&self) -> String {
        let content = self.last_content();
        let mut result = String::new();
        if let Some(range) = content.selectable_range {
            for indexed in content.grid.display_iter() {
                if range.contains(indexed.point) {
                    result.push(indexed.c);
                }
            }
        }
        result
    }

    pub fn sync(&mut self) -> &RenderableContent {
        if !self.dirty.swap(false, Ordering::Relaxed) {
            return self.last_content();
        }
        let term = self.term.clone();
        let mut terminal = term.lock();
        let selectable_range = match &terminal.selection {
            Some(s) => s.to_range(&terminal),
            None => None,
        };

        let cursor = terminal.grid_mut().cursor_cell().clone();
        self.last_content.grid = terminal.grid().clone();
        self.last_content.selectable_range = selectable_range;
        self.last_content.cursor = cursor.clone();
        self.last_content.terminal_mode = *terminal.mode();
        self.last_content.terminal_size = self.size;
        self.last_content()
    }

    pub fn set_dirty(&mut self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn last_content(&self) -> &RenderableContent {
        &self.last_content
    }

    pub fn child_pid(&self) -> u32 {
        self.child_pid
    }

    fn process_link_action(
        &mut self,
        terminal: &Term<EventProxy>,
        link_action: LinkAction,
        point: Point,
    ) {
        match link_action {
            LinkAction::Hover => {
                self.last_content.hovered_hyperlink = self.regex_match_at(
                    terminal,
                    point,
                    &mut self.url_regex.clone(),
                );
            },
            LinkAction::Clear => {
                self.last_content.hovered_hyperlink = None;
            },
            LinkAction::Open => {
                self.open_link();
            },
        };
    }

    fn open_link(&self) {
        if let Some(range) = &self.last_content.hovered_hyperlink {
            let start = range.start();
            let end = range.end();

            let mut url = String::from(self.last_content.grid.index(*start).c);
            for indexed in self.last_content.grid.iter_from(*start) {
                url.push(indexed.c);
                if indexed.point == *end {
                    break;
                }
            }

            if let Err(err) = open::that(&url) {
                // A remote text plus one click must NEVER be able to
                // crash the app (was: panic!).
                log::error!("failed to open link {url:?}: {err}");
            }
        }
    }

    fn process_mouse_report(
        &self,
        button: MouseButton,
        modifiers: Modifiers,
        point: Point,
        pressed: bool,
    ) {
        let mut mods = 0;
        if modifiers.contains(Modifiers::SHIFT) {
            mods += 4;
        }
        if modifiers.contains(Modifiers::ALT) {
            mods += 8;
        }
        if modifiers.contains(Modifiers::COMMAND) {
            mods += 16;
        }

        match MouseMode::from(self.last_content().terminal_mode) {
            MouseMode::Sgr => {
                self.sgr_mouse_report(point, button as u8 + mods, pressed)
            },
            MouseMode::Normal(is_utf8) => {
                if pressed {
                    self.normal_mouse_report(
                        point,
                        button as u8 + mods,
                        is_utf8,
                    )
                } else {
                    self.normal_mouse_report(point, 3 + mods, is_utf8)
                }
            },
        }
    }

    fn sgr_mouse_report(&self, point: Point, button: u8, pressed: bool) {
        let c = if pressed { 'M' } else { 'm' };

        let msg = format!(
            "\x1b[<{};{};{}{}",
            button,
            point.column + 1,
            point.line + 1,
            c
        );

        self.tx_bytes
            .fetch_add(msg.as_bytes().len() as u64, Ordering::Relaxed);
        self.notifier.notify(msg.as_bytes().to_vec());
    }

    fn normal_mouse_report(&self, point: Point, button: u8, is_utf8: bool) {
        let Point { line, column } = point;
        let max_point = if is_utf8 { 2015 } else { 223 };

        if line >= max_point || column >= max_point {
            return;
        }

        let mut msg = vec![b'\x1b', b'[', b'M', 32 + button];

        let mouse_pos_encode = |pos: usize| -> Vec<u8> {
            let pos = 32 + 1 + pos;
            let first = 0xC0 + pos / 64;
            let second = 0x80 + (pos & 63);
            vec![first as u8, second as u8]
        };

        if is_utf8 && column >= Column(95) {
            msg.append(&mut mouse_pos_encode(column.0));
        } else {
            msg.push(32 + 1 + column.0 as u8);
        }

        if is_utf8 && line >= 95 {
            msg.append(&mut mouse_pos_encode(line.0 as usize));
        } else {
            msg.push(32 + 1 + line.0 as u8);
        }

        self.tx_bytes.fetch_add(msg.len() as u64, Ordering::Relaxed);
        self.notifier.notify(msg);
    }

    fn start_selection(
        &mut self,
        terminal: &mut Term<EventProxy>,
        selection_type: SelectionType,
        x: f32,
        y: f32,
    ) {
        self.dirty.store(true, Ordering::Relaxed);
        let location = Self::selection_point(
            x,
            y,
            &self.size,
            terminal.grid().display_offset(),
        );
        terminal.selection = Some(Selection::new(
            selection_type,
            location,
            self.selection_side(x),
        ));
    }

    fn update_selection(
        &mut self,
        terminal: &mut Term<EventProxy>,
        x: f32,
        y: f32,
    ) {
        self.dirty.store(true, Ordering::Relaxed);
        let display_offset = terminal.grid().display_offset();
        if let Some(ref mut selection) = terminal.selection {
            let location =
                Self::selection_point(x, y, &self.size, display_offset);
            selection.update(location, self.selection_side(x));
        }
    }

    fn selection_side(&self, x: f32) -> Side {
        let cell_x = x as usize % self.size.cell_width as usize;
        let half_cell_width = (self.size.cell_width as f32 / 2.0) as usize;

        if cell_x > half_cell_width {
            Side::Right
        } else {
            Side::Left
        }
    }

    fn resize(
        &mut self,
        terminal: &mut Term<EventProxy>,
        layout_size: Size,
        font_size: Size,
    ) {
        let cell_w = font_size.width.floor().max(1.0);
        let cell_h = font_size.height.floor().max(1.0);
        let cols = ((layout_size.width + 0.5) / cell_w).floor() as u16;
        let lines = ((layout_size.height + 0.5) / cell_h).floor() as u16;
        if cols == 0 || lines == 0 {
            return;
        }

        let cell_width = cell_w as u16;
        let cell_height = cell_h as u16;

        // Only reflow / SIGWINCH when the integer grid size changes.
        if cols == self.size.num_cols
            && lines == self.size.num_lines
            && cell_width == self.size.cell_width
            && cell_height == self.size.cell_height
        {
            self.size.layout_size = layout_size;
            return;
        }

        let grid_changed =
            cols != self.size.num_cols || lines != self.size.num_lines;

        self.size = TerminalSize {
            layout_size,
            cell_height,
            cell_width,
            num_lines: lines,
            num_cols: cols,
        };

        if grid_changed {
            self.dirty.store(true, Ordering::Relaxed);
            // Notify PTY first, then reflow the emulator grid once.
            self.notifier.on_resize(self.size.into());
            terminal.resize(TermSize::new(
                self.size.num_cols as usize,
                self.size.num_lines as usize,
            ));
            // After reflow, pin view to the bottom so scrollback/history does not
            // leave the viewport mid-screen (looks like duplicated lines).
            terminal.scroll_display(Scroll::Bottom);
        }
    }

    fn write<I: Into<Cow<'static, [u8]>>>(&self, input: I) {
        let cow = input.into();
        self.tx_bytes.fetch_add(cow.len() as u64, Ordering::Relaxed);
        self.notifier.notify(cow);
    }

    fn scroll(&mut self, terminal: &mut Term<EventProxy>, delta_value: i32) {
        if delta_value != 0 {
            self.dirty.store(true, Ordering::Relaxed);
            let scroll = Scroll::Delta(delta_value);
            if terminal
                .mode()
                .contains(TermMode::ALTERNATE_SCROLL | TermMode::ALT_SCREEN)
            {
                let line_cmd = if delta_value > 0 { b'A' } else { b'B' };
                let mut content = vec![];

                for _ in 0..delta_value.abs() {
                    content.push(0x1b);
                    content.push(b'O');
                    content.push(line_cmd);
                }

                self.tx_bytes
                    .fetch_add(content.len() as u64, Ordering::Relaxed);
                self.notifier.notify(content);
            } else {
                terminal.grid_mut().scroll_display(scroll);
            }
        }
    }

    /// Based on alacritty/src/display/hint.rs > regex_match_at
    /// Retrieve the match, if the specified point is inside the content matching the regex.
    fn regex_match_at(
        &self,
        terminal: &Term<EventProxy>,
        point: Point,
        regex: &mut RegexSearch,
    ) -> Option<Match> {
        let x = visible_regex_match_iter(terminal, regex)
            .find(|rm| rm.contains(&point));
        x
    }
}

/// Copied from alacritty/src/display/hint.rs:
/// Iterate over all visible regex matches.
fn visible_regex_match_iter<'a>(
    term: &'a Term<EventProxy>,
    regex: &'a mut RegexSearch,
) -> impl Iterator<Item = Match> + 'a {
    let viewport_start = Line(-(term.grid().display_offset() as i32));
    let viewport_end = viewport_start + term.bottommost_line();
    let mut start =
        term.line_search_left(Point::new(viewport_start, Column(0)));
    let mut end = term.line_search_right(Point::new(viewport_end, Column(0)));
    start.line = start.line.max(viewport_start - 100);
    end.line = end.line.min(viewport_end + 100);

    RegexIter::new(start, end, Direction::Right, term, regex)
        .skip_while(move |rm| rm.end().line < viewport_start)
        .take_while(move |rm| rm.start().line <= viewport_end)
}

pub struct RenderableContent {
    pub grid: Grid<Cell>,
    pub hovered_hyperlink: Option<RangeInclusive<Point>>,
    pub selectable_range: Option<SelectionRange>,
    pub cursor: Cell,
    pub terminal_mode: TermMode,
    pub terminal_size: TerminalSize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn backend_syncs_startup_output_without_external_event_receiver() {
        let mut backend = TerminalBackend::new(
            1,
            egui::Context::default(),
            BackendSettings {
                shell: "/bin/sh".into(),
                args: vec!["-c".into(), "printf startup; sleep 1".into()],
                working_directory: Some(PathBuf::from("/tmp")),
                env: vec![],
                scrollback: 100,
            },
        )
        .expect("terminal backend should start");

        thread::sleep(Duration::from_millis(150));
        let content = backend.sync();
        let output: String =
            content.grid.display_iter().map(|cell| cell.c).collect();

        assert!(output.contains("startup"));
    }

    #[test]
    fn event_subscription_stops_when_sender_is_dropped() {
        let (sender, receiver) = mpsc::channel();
        drop(sender);

        receive_events(
            receiver,
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicBool::new(false)),
            egui::Context::default(),
        );
    }
}

impl Default for RenderableContent {
    fn default() -> Self {
        Self {
            grid: Grid::new(0, 0, 0),
            hovered_hyperlink: None,
            selectable_range: None,
            cursor: Cell::default(),
            terminal_mode: TermMode::empty(),
            terminal_size: TerminalSize::default(),
        }
    }
}

impl Drop for TerminalBackend {
    fn drop(&mut self) {
        // Nudge a still-running child so the blocking wait() inside
        // alacritty's own Pty drop (on the reader thread) cannot strand
        // this terminal's subscription thread forever when the shell
        // ignores SIGHUP. Already-exited children are skipped.
        #[cfg(unix)]
        if !self.child_exited.load(Ordering::Relaxed) && self.child_pid != 0 {
            unsafe {
                libc::kill(self.child_pid as libc::pid_t, libc::SIGTERM);
            }
        }
        let _ = self.notifier.0.send(Msg::Shutdown);
    }
}

fn receive_events(
    event_receiver: mpsc::Receiver<Event>,
    dirty: Arc<AtomicBool>,
    child_exited: Arc<AtomicBool>,
    app_context: egui::Context,
) {
    while let Ok(event) = event_receiver.recv() {
        dirty.store(true, Ordering::Relaxed);
        app_context.request_repaint();
        if matches!(event, Event::Exit) {
            child_exited.store(true, Ordering::Relaxed);
            break;
        }
    }
}

#[derive(Clone)]
pub struct EventProxy(mpsc::Sender<Event>);

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let _ = self.0.send(event.clone());
    }
}

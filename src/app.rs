use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

use crate::ui::layout_tree::{LayoutNode, SplitDirection};
use crate::ui::tab_group::{TabGroup, Tab};
use crate::ui::drag::DragState;

pub struct App {
    should_quit: bool,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    layout: LayoutNode,
    tab_group: TabGroup,
    drag_state: DragState,
    active_pane_id: Option<String>,
    pane_counter: u32,
    tab_counter: u32,
}

impl App {
    pub async fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(App {
            should_quit: false,
            terminal,
            layout: LayoutNode::default(),
            tab_group: TabGroup::default(),
            drag_state: DragState::new(),
            active_pane_id: Some("terminal-1".to_string()),
            pane_counter: 1,
            tab_counter: 1,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        while !self.should_quit {
            self.draw()?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let terminal = &mut self.terminal;
        terminal.draw(|f| {
            crate::ui::layout::render_layout_tree(f, f.size(), &self.layout);
        })?;
        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => self.handle_key_event(key),
                Event::Mouse(mouse) => {
                    crate::ui::drag::handle_mouse_event(mouse, &mut self.drag_state);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            (KeyCode::Char('\\'), KeyModifiers::CONTROL) => {
                self.split_horizontal();
            }
            (KeyCode::Char('|'), KeyModifiers::CONTROL) => {
                self.split_vertical();
            }
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                self.new_tab();
            }
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                self.close_tab();
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.new_pane();
            }
            _ => {}
        }
    }

    fn split_horizontal(&mut self) {
        if let Some(pane_id) = &self.active_pane_id {
            self.pane_counter += 1;
            let new_pane_id = format!("terminal-{}", self.pane_counter);
            let new_title = format!("终端 {}", self.pane_counter);
            self.layout.split_pane(pane_id, SplitDirection::Horizontal, &new_pane_id, &new_title);
        }
    }

    fn split_vertical(&mut self) {
        if let Some(pane_id) = &self.active_pane_id {
            self.pane_counter += 1;
            let new_pane_id = format!("terminal-{}", self.pane_counter);
            let new_title = format!("终端 {}", self.pane_counter);
            self.layout.split_pane(pane_id, SplitDirection::Vertical, &new_pane_id, &new_title);
        }
    }

    fn new_tab(&mut self) {
        self.tab_counter += 1;
        let tab_id = format!("tab-{}", self.tab_counter);
        let title = format!("选项卡 {}", self.tab_counter);
        self.pane_counter += 1;
        let terminal_id = format!("terminal-{}", self.pane_counter);

        self.tab_group.add_tab(Tab {
            id: tab_id,
            title,
            terminal_id,
        });
    }

    fn close_tab(&mut self) {
        if let Some(active_tab) = self.tab_group.get_active_tab().cloned() {
            self.tab_group.remove_tab(&active_tab.id);
        }
    }

    fn new_pane(&mut self) {
        self.split_vertical();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        );
        let _ = self.terminal.show_cursor();
    }
}

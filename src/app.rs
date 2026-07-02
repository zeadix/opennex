use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};
use std::io;

pub struct App {
    should_quit: bool,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
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
            Self::ui_static(f);
        })?;
        Ok(())
    }

    fn ui_static(f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(f.size());

        // 顶部选项卡栏
        let menu_titles = vec!["终端 1", "终端 2", "终端 3"];
        let tabs = Tabs::new(menu_titles)
            .block(Block::default().borders(Borders::ALL).title("AI 终端管理器"))
            .select(0)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, chunks[0]);

        // 终端内容区域
        let block = Block::default()
            .borders(Borders::ALL)
            .title("终端内容");
        f.render_widget(block, chunks[1]);
    }

    async fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key);
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
            _ => {}
        }
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

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs, Wrap},
    Frame, Terminal,
};
use std::io;

mod app;
mod terminal;
mod state;

use app::App;

fn main() -> Result<()> {
    env_logger::init();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Char('n'), KeyModifiers::CONTROL) => app.new_tab(),
                    (KeyCode::Char('w'), KeyModifiers::CONTROL) => app.close_tab(),
                    (KeyCode::Char('\\'), KeyModifiers::CONTROL) => app.split_horizontal(),
                    (KeyCode::Char('|'), KeyModifiers::CONTROL) => app.split_vertical(),
                    (KeyCode::Tab, _) => app.next_pane(),
                    (KeyCode::BackTab, KeyModifiers::SHIFT) => app.prev_pane(),
                    (KeyCode::Char('1'), KeyModifiers::CONTROL) => app.select_pane(0),
                    (KeyCode::Char('2'), KeyModifiers::CONTROL) => app.select_pane(1),
                    (KeyCode::Char('3'), KeyModifiers::CONTROL) => app.select_pane(2),
                    (KeyCode::Char('4'), KeyModifiers::CONTROL) => app.select_pane(3),
                    (KeyCode::Char('5'), KeyModifiers::CONTROL) => app.select_pane(4),
                    (KeyCode::Char('6'), KeyModifiers::CONTROL) => app.select_pane(5),
                    (KeyCode::Char('7'), KeyModifiers::CONTROL) => app.select_pane(6),
                    (KeyCode::Char('8'), KeyModifiers::CONTROL) => app.select_pane(7),
                    (KeyCode::Char('9'), KeyModifiers::CONTROL) => app.select_pane(8),
                    (KeyCode::Char('l'), KeyModifiers::CONTROL) => app.clear_active(),
                    _ => app.handle_input(key),
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(f.size());

    render_tab_bar(f, chunks[0], app);
    render_content(f, chunks[1], app);
}

fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<&str> = app.tabs.iter().map(|t| t.name.as_str()).collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" OpenZoo Terminal Manager "))
        .select(app.active_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &mut App) {
    if let Some(tab) = app.tabs.get(app.active_tab) {
        let panes = &tab.panes;
        let active = tab.active_pane;
        let total = panes.len();

        if total == 1 {
            render_pane(f, area, &panes[0], 0 == active);
        } else {
            let direction = if total == 2 {
                Direction::Horizontal
            } else {
                Direction::Vertical
            };
            let constraints: Vec<Constraint> = panes.iter().map(|_| Constraint::Ratio(1, total as u32)).collect();
            let chunks = Layout::default().direction(direction).constraints(constraints).split(area);
            for (i, pane) in panes.iter().enumerate() {
                render_pane(f, chunks[i], pane, i == active);
            }
        }
    }
}

fn render_pane(f: &mut Frame, area: Rect, pane: &terminal::Pane, is_active: bool) {
    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if is_active {
        format!(" {} [ACTIVE] ", pane.name)
    } else {
        format!(" {} ", pane.name)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(border_style);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if pane.content.is_empty() {
        let help = "Commands: help, ls, pwd, echo <text>, clear, calc <expr>\n\
                     Shortcuts: Ctrl+N=New Tab, Ctrl+W=Close Tab, Ctrl+\\=Split H, Ctrl+|=Split V\n\
                     Ctrl+1-9=Switch Pane, Tab=Next Pane, Ctrl+L=Clear, Ctrl+Q=Quit";
        let paragraph = ratatui::widgets::Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    } else {
        let paragraph = ratatui::widgets::Paragraph::new(pane.content.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }

    if is_active {
        let input_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        let input_text = format!("$ {}", pane.input);
        let input = ratatui::widgets::Paragraph::new(input_text)
            .style(Style::default().fg(Color::Green).bg(Color::Black));
        f.render_widget(input, input_area);
        f.set_cursor(
            input_area.x + 2 + pane.input.len() as u16,
            input_area.y,
        );
    }
}

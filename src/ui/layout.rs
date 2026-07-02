use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};

use super::tabs;

pub fn render(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 选项卡栏
            Constraint::Min(1),    // 终端内容
        ])
        .split(area);

    // 渲染选项卡栏
    tabs::render_tab_bar(f, chunks[0]);

    // 渲染终端内容
    render_terminal_content(f, chunks[1]);
}

fn render_terminal_content(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("终端")
        .style(Style::default().fg(Color::White));
    f.render_widget(block, area);
}

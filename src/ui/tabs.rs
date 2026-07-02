use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Tabs};

pub fn render_tab_bar(f: &mut Frame, area: Rect) {
    let menu_titles = vec!["终端 1", "终端 2", "终端 3"];
    let tabs = Tabs::new(menu_titles)
        .block(Block::default().borders(Borders::ALL).title("AI 终端管理器"))
        .select(0)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

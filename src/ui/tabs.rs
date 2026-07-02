use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Tabs};

use super::tab_group::TabGroup;

pub fn render_tab_bar(f: &mut Frame, area: Rect) {
    let tab_group = TabGroup::default();
    let titles: Vec<&str> = tab_group.tabs.iter().map(|t| t.title.as_str()).collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("AI 终端管理器"))
        .select(tab_group.active_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

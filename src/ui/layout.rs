use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};

use super::layout_tree::{LayoutNode, SplitDirection};
use super::tabs;

pub fn render(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    tabs::render_tab_bar(f, chunks[0]);
    render_layout_tree(f, chunks[1], &LayoutNode::default());
}

pub fn render_layout_tree(f: &mut Frame, area: Rect, node: &LayoutNode) {
    match node {
        LayoutNode::Pane { id, title } => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .style(Style::default().fg(Color::White));
            f.render_widget(block, area);
        }
        LayoutNode::Split { direction, ratio, children } => {
            let constraints = if children.len() == 2 {
                let first = (ratio * 100.0) as u16;
                let second = 100 - first;
                vec![
                    Constraint::Percentage(first),
                    Constraint::Percentage(second),
                ]
            } else {
                children.iter().map(|_| Constraint::Ratio(1, children.len() as u32)).collect()
            };

            let dir = match direction {
                SplitDirection::Horizontal => Direction::Vertical,
                SplitDirection::Vertical => Direction::Horizontal,
            };

            let chunks = Layout::default()
                .direction(dir)
                .constraints(constraints)
                .split(area);

            for (i, child) in children.iter().enumerate() {
                if i < chunks.len() {
                    render_layout_tree(f, chunks[i], child);
                }
            }
        }
    }
}

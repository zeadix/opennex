pub mod layout;
pub mod tabs;
pub mod layout_tree;
pub mod tab_group;
pub mod drag;

use ratatui::Frame;
use ratatui::layout::Rect;

pub fn render(f: &mut Frame, area: Rect) {
    layout::render(f, area);
}

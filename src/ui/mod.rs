pub mod layout;
pub mod tabs;

use ratatui::Frame;
use ratatui::layout::Rect;

pub fn render(f: &mut Frame, area: Rect) {
    layout::render(f, area);
}

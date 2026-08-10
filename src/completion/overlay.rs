use egui::{Align2, Color32, Pos2};

pub struct SuggestionOverlay {
    pub suggestion: Option<String>,
}

impl SuggestionOverlay {
    pub fn new() -> Self {
        Self { suggestion: None }
    }

    pub fn render(&self, painter: &egui::Painter, cursor_pos: Pos2, font_id: egui::FontId) {
        if let Some(ref text) = self.suggestion {
            painter.text(
                cursor_pos,
                Align2::LEFT_CENTER,
                text,
                font_id.clone(),
                Color32::from_rgba_premultiplied(128, 128, 128, 128),
            );
        }
    }
}

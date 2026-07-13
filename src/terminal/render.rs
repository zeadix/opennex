use crate::terminal::TerminalInstance;
use crate::terminal::grid::Cell;

pub fn render_terminal(
    ui: &mut egui::Ui,
    instance: &TerminalInstance,
    cell_w: f32,
    cell_h: f32,
    bg_color: egui::Color32,
    _fg_color: egui::Color32,
    cell_spacing: f32,
) -> egui::Response {
    let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click());
    let rect = response.rect;

    let effective_cell_w = cell_w * cell_spacing;
    painter.rect_filled(rect, egui::CornerRadius::ZERO, bg_color);

    if let Ok(g) = instance.grid.lock() {
        let show_rows = (rect.height() / cell_h).floor() as usize;
        let show_cols = (rect.width() / effective_cell_w).floor() as usize;
        let rows = show_rows.min(g.rows);
        let cols = show_cols.min(g.cols);

        for row in 0..rows {
            let y = rect.min.y + row as f32 * cell_h;
            for col in 0..cols {
                let cell = &g.cells[row][col];
                if cell.ch == ' ' || cell.ch == '\0' { continue; }

                let x = rect.min.x + col as f32 * effective_cell_w;

                let mut fg = egui::Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2]);
                let mut bg = egui::Color32::from_rgb(cell.bg[0], cell.bg[1], cell.bg[2]);

                if cell.flags.inverse {
                    std::mem::swap(&mut fg, &mut bg);
                }
                if cell.flags.dim {
                    fg = fg.linear_multiply(0.7);
                }

                // Only draw cell background if it's not the default black background
                let is_default_bg = cell.bg == [0, 0, 0];
                if !is_default_bg && bg != bg_color {
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(effective_cell_w, cell_h)),
                        0.0, bg,
                    );
                }

                let mut text = egui::RichText::new(cell.ch.to_string())
                    .color(fg)
                    .font(egui::FontId::monospace(instance.font_size));
                if cell.flags.bold { text = text.strong(); }
                if cell.flags.italic { text = text.italics(); }
                if cell.flags.underline { text = text.underline(); }

                painter.text(
                    egui::pos2(x, y), egui::Align2::LEFT_TOP,
                    text.text(),
                    egui::FontId::monospace(instance.font_size),
                    fg,
                );
            }
        }

        // Draw cursor
        if g.cursor_visible && g.cursor_row < rows && g.cursor_col < cols {
            let cx = rect.min.x + g.cursor_col as f32 * effective_cell_w;
            let cy = rect.min.y + g.cursor_row as f32 * cell_h;
            painter.rect_filled(
                egui::Rect::from_min_size(egui::pos2(cx, cy), egui::vec2(effective_cell_w, cell_h)),
                0.0, egui::Color32::WHITE,
            );
            // Draw cursor char in black
            if g.cursor_row < g.rows && g.cursor_col < g.cols {
                let cell = &g.cells[g.cursor_row][g.cursor_col];
                if cell.ch != ' ' && cell.ch != '\0' {
                    painter.text(
                        egui::pos2(cx, cy), egui::Align2::LEFT_TOP,
                        cell.ch.to_string(),
                        egui::FontId::monospace(instance.font_size),
                        egui::Color32::BLACK,
                    );
                }
            }
        }
    }

    response
}

pub fn render_snapshot(
    ui: &mut egui::Ui,
    snapshot: &crate::snapshot::state::TerminalSnapshot,
    font_size: f32,
) -> egui::Response {
    let size = ui.available_size();
    let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
    let rect = response.rect;
    if let Some(first_row) = snapshot.grid.first() {
        let cols = first_row.len();
        let rows = snapshot.grid.len();
        let cell_w = if cols > 0 { size.x / cols as f32 } else { size.x };
        let cell_h = if rows > 0 { size.y / rows as f32 } else { size.y };
        painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
        for (row_idx, row) in snapshot.grid.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let x = rect.min.x + col_idx as f32 * cell_w;
                let y = rect.min.y + row_idx as f32 * cell_h;
                let fg = egui::Color32::from_rgb(cell.fg[0], cell.fg[1], cell.fg[2]);
                let bg = egui::Color32::from_rgb(cell.bg[0], cell.bg[1], cell.bg[2]);
                if bg != egui::Color32::BLACK {
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_w, cell_h)),
                        0.0, bg,
                    );
                }
                painter.text(
                    egui::pos2(x, y), egui::Align2::LEFT_TOP,
                    cell.ch.to_string(),
                    egui::FontId::monospace(font_size), fg,
                );
            }
        }
        let (cx, cy) = snapshot.cursor;
        let cursor_x = rect.min.x + cx as f32 * cell_w;
        let cursor_y = rect.min.y + cy as f32 * cell_h;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(cursor_x, cursor_y), egui::vec2(cell_w, cell_h)),
            0.0, egui::Color32::WHITE,
        );
    }
    response
}
pub mod model;
pub mod palettes;
pub mod store;
pub mod ui;

pub use model::{ThemeColor, ThemeDefinition, ThemeError};

use egui::{Color32, Stroke};

/// Apply a `ThemeDefinition`'s application colors to egui visuals.
pub fn apply_theme_definition(ctx: &egui::Context, theme: &ThemeDefinition) {
    let a = &theme.app;
    let style = (*ctx.style()).clone();
    let mut visuals = style.visuals.clone();

    let app_bg = a.app_bg.to_egui();
    let sidebar = a.sidebar.to_egui();
    let panel = a.panel.to_egui();
    let hover = a.hover.to_egui();
    let active = a.active.to_egui();
    let border = a.border.to_egui();
    let text = a.text.to_egui();
    let weak_text = a.weak_text.to_egui();
    let accent = a.accent.to_egui();
    let input_bg = a.input_bg.to_egui();
    let [sr, sg, sb, sa] = a.window_shadow.to_array();

    visuals.panel_fill = app_bg;
    visuals.extreme_bg_color = input_bg;
    visuals.faint_bg_color = sidebar;
    visuals.widgets.noninteractive.bg_fill = app_bg;
    visuals.widgets.noninteractive.weak_bg_fill = app_bg;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, weak_text);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);

    visuals.widgets.hovered.bg_fill = hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.active.bg_fill = active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.open.bg_fill = hover;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, accent);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text);

    visuals.widgets.hovered.weak_bg_fill = hover;
    visuals.widgets.active.weak_bg_fill = active;
    visuals.widgets.open.weak_bg_fill = hover;

    visuals.warn_fg_color = a.warning.to_egui();
    visuals.error_fg_color = a.danger.to_egui();

    visuals.selection.bg_fill = a.selection_bg.to_egui();
    visuals.selection.stroke = Stroke::new(1.0, a.selection_text.to_egui());

    visuals.hyperlink_color = accent;

    visuals.window_fill = panel;
    visuals.window_stroke = Stroke::new(1.0, border);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: Color32::from_rgba_unmultiplied(sr, sg, sb, sa),
    };
    visuals.text_cursor.stroke = Stroke::new(2.0, accent);

    ctx.set_style(style);
    ctx.set_visuals(visuals);
}

/// Build an `egui_term::TerminalTheme` from a `ThemeDefinition`.
pub fn terminal_theme(theme: &ThemeDefinition) -> egui_term::TerminalTheme {
    let t = &theme.terminal;
    let mk = |c: &ThemeColor| c.as_hex();
    let palette = Box::new(egui_term::ColorPalette {
        foreground: mk(&t.foreground),
        background: mk(&t.background),
        black: mk(&t.normal.black),
        red: mk(&t.normal.red),
        green: mk(&t.normal.green),
        yellow: mk(&t.normal.yellow),
        blue: mk(&t.normal.blue),
        magenta: mk(&t.normal.magenta),
        cyan: mk(&t.normal.cyan),
        white: mk(&t.normal.white),
        bright_black: mk(&t.bright.black),
        bright_red: mk(&t.bright.red),
        bright_green: mk(&t.bright.green),
        bright_yellow: mk(&t.bright.yellow),
        bright_blue: mk(&t.bright.blue),
        bright_magenta: mk(&t.bright.magenta),
        bright_cyan: mk(&t.bright.cyan),
        bright_white: mk(&t.bright.white),
        bright_foreground: None,
        dim_foreground: mk(&t.dim_foreground),
        dim_black: mk(&t.dim.black),
        dim_red: mk(&t.dim.red),
        dim_green: mk(&t.dim.green),
        dim_yellow: mk(&t.dim.yellow),
        dim_blue: mk(&t.dim.blue),
        dim_magenta: mk(&t.dim.magenta),
        dim_cyan: mk(&t.dim.cyan),
        dim_white: mk(&t.dim.white),
    });
    let visual = egui_term::TerminalVisualColors {
        cursor: t.cursor.to_egui(),
        selection_bg: t.selection_bg.to_egui(),
        selection_text: t.selection_text.to_egui(),
        link: t.link.to_egui(),
    };
    egui_term::TerminalTheme::new(palette, visual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_default_theme_applies_without_error() {
        let theme = store::default_theme().unwrap();
        let ctx = egui::Context::default();
        apply_theme_definition(&ctx, &theme);
        let visuals = &ctx.style().visuals;
        assert_eq!(visuals.panel_fill, theme.app.app_bg.to_egui());
    }

    #[test]
    fn terminal_theme_maps_all_palette_colors() {
        let theme = store::default_theme().unwrap();
        let tt = terminal_theme(&theme);
        assert_eq!(tt.cursor_color(), theme.terminal.cursor.to_egui());
        assert_eq!(tt.link_color(), theme.terminal.link.to_egui());
    }
}

// Text accessors that delegate to `ui::texts`, used by app.rs.
// These are kept here so app.rs doesn't need to know the internal module layout.
pub fn ui_appearance_text() -> String {
    ui::texts::ui_appearance()
}
pub fn terminal_appearance_text() -> String {
    ui::texts::terminal_appearance()
}
pub fn ansi_palette_text() -> String {
    "ANSI 调色板".into()
}
pub fn theme_label_text() -> String {
    ui::texts::theme_label()
}
pub fn unsaved_text() -> String {
    ui::texts::unsaved()
}
pub fn new_theme_text() -> String {
    ui::texts::new_theme()
}
pub fn copy_theme_text() -> String {
    ui::texts::copy_theme()
}
pub fn rename_theme_text() -> String {
    ui::texts::rename_theme()
}
pub fn delete_theme_text() -> String {
    ui::texts::delete_theme()
}
pub fn import_theme_text() -> String {
    ui::texts::import_theme()
}
pub fn export_theme_text() -> String {
    ui::texts::export_theme()
}
pub fn ui_font_label() -> String {
    ui::texts::ui_font()
}
pub fn ui_font_size_label() -> String {
    ui::texts::ui_font_size()
}
pub fn base_colors_text() -> String {
    ui::texts::base_colors()
}
pub fn app_bg_label() -> String {
    ui::texts::app_bg()
}
pub fn sidebar_label() -> String {
    ui::texts::sidebar()
}
pub fn panel_label() -> String {
    ui::texts::panel()
}
pub fn input_bg_label() -> String {
    ui::texts::input_bg()
}
pub fn text_colors_text() -> String {
    ui::texts::text_colors()
}
pub fn text_label() -> String {
    ui::texts::text()
}
pub fn weak_text_label() -> String {
    ui::texts::weak_text()
}
pub fn status_colors_text() -> String {
    ui::texts::status_colors()
}
pub fn accent_label() -> String {
    ui::texts::accent()
}
pub fn warning_label() -> String {
    ui::texts::warning()
}
pub fn danger_label() -> String {
    ui::texts::danger()
}
pub fn interaction_colors_text() -> String {
    ui::texts::interaction_colors()
}
pub fn hover_label() -> String {
    ui::texts::hover()
}
pub fn active_label() -> String {
    ui::texts::active()
}
pub fn selection_bg_label() -> String {
    ui::texts::selection_bg()
}
pub fn selection_text_label() -> String {
    ui::texts::selection_text()
}
pub fn border_label() -> String {
    ui::texts::border()
}
pub fn lock_label() -> String {
    ui::texts::lock()
}
pub fn terminal_font_size_label() -> String {
    ui::texts::terminal_font_size()
}
pub fn cell_spacing_label() -> String {
    ui::texts::cell_spacing()
}
pub fn palette_template_label() -> String {
    ui::texts::palette_template()
}
pub fn apply_template_text() -> String {
    ui::texts::apply_template()
}
pub fn terminal_base_colors_text() -> String {
    ui::texts::terminal_base_colors()
}
pub fn fg_label() -> String {
    ui::texts::fg()
}
pub fn bg_label() -> String {
    ui::texts::bg()
}
pub fn cursor_label() -> String {
    ui::texts::cursor()
}
pub fn link_label() -> String {
    ui::texts::link()
}
pub fn normal_label() -> String {
    ui::texts::normal()
}
pub fn bright_label() -> String {
    ui::texts::bright()
}
pub fn dim_label() -> String {
    ui::texts::dim()
}
pub fn black_label() -> &'static str {
    ui::texts::black()
}
pub fn red_label() -> &'static str {
    ui::texts::red()
}
pub fn green_label() -> &'static str {
    ui::texts::green()
}
pub fn yellow_label() -> &'static str {
    ui::texts::yellow()
}
pub fn blue_label() -> &'static str {
    ui::texts::blue()
}
pub fn magenta_label() -> &'static str {
    ui::texts::magenta()
}
pub fn cyan_label() -> &'static str {
    ui::texts::cyan()
}
pub fn white_label() -> &'static str {
    ui::texts::white()
}
pub fn copy_dialog_title() -> String {
    ui::texts::copy_dialog_title()
}
pub fn copy_dialog_hint() -> String {
    ui::texts::copy_dialog_hint()
}
pub fn new_dialog_title() -> String {
    ui::texts::new_dialog_title()
}
pub fn new_dialog_hint() -> String {
    ui::texts::new_dialog_hint()
}
pub fn rename_dialog_title() -> String {
    ui::texts::rename_dialog_title()
}
pub fn delete_confirm_text() -> String {
    ui::texts::delete_confirm()
}
pub fn switch_confirm_text() -> String {
    ui::texts::switch_confirm()
}
pub fn save_and_switch_text() -> String {
    ui::texts::save_and_switch()
}
pub fn discard_and_switch_text() -> String {
    ui::texts::discard_and_switch()
}
pub fn cancel_text() -> String {
    ui::texts::cancel()
}
pub fn confirm_text() -> String {
    ui::texts::confirm()
}
pub fn ok_text() -> String {
    ui::texts::ok()
}
pub fn builtin_readonly_text() -> String {
    ui::texts::builtin_readonly()
}

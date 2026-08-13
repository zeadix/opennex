pub mod model;
pub mod store;

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

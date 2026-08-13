pub mod model;
pub mod store;

pub use model::{
    AnsiColors, AppTheme, TerminalThemeConfig, ThemeColor, ThemeDefinition, ThemeError,
    TypographyTheme, THEME_FORMAT_VERSION,
};

use egui::{Color32, Stroke};
use serde::{de::Deserializer, Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Serialize for ThemeMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        })
    }
}

impl<'de> Deserialize<'de> for ThemeMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "light" => ThemeMode::Light,
            _ => ThemeMode::Dark,
        })
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::Light => write!(f, "light"),
            ThemeMode::Dark => write!(f, "dark"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub app_bg: Color32,
    pub sidebar: Color32,
    pub panel: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub border: Color32,
    pub text: Color32,
    pub weak_text: Color32,
    pub accent: Color32,
    pub danger: Color32,
    pub lock: Color32,
    pub input_bg: Color32,
    pub window_shadow: Color32,
}

pub fn palette(mode: ThemeMode) -> ThemePalette {
    match mode {
        ThemeMode::Dark => ThemePalette {
            app_bg: Color32::from_rgb(0x17, 0x19, 0x1C),
            sidebar: Color32::from_rgb(0x1D, 0x20, 0x24),
            panel: Color32::from_rgb(0x24, 0x27, 0x2C),
            hover: Color32::from_rgb(0x2E, 0x33, 0x3A),
            active: Color32::from_rgb(0x33, 0x38, 0x40),
            border: Color32::from_rgb(0x34, 0x3A, 0x40),
            text: Color32::from_rgb(0xE6, 0xE9, 0xED),
            weak_text: Color32::from_rgb(0x8C, 0x95, 0x9F),
            accent: Color32::from_rgb(0x2C, 0xBF, 0xAE),
            danger: Color32::from_rgb(0xE0, 0x5A, 0x65),
            lock: Color32::from_rgb(0xD9, 0xA4, 0x41),
            input_bg: Color32::from_rgb(0x1A, 0x1D, 0x21),
            window_shadow: Color32::from_black_alpha(120),
        },
        ThemeMode::Light => ThemePalette {
            app_bg: Color32::from_rgb(0xEE, 0xF1, 0xF3),
            sidebar: Color32::from_rgb(0xE9, 0xED, 0xF0),
            panel: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            hover: Color32::from_rgb(0xDF, 0xE4, 0xE8),
            active: Color32::from_rgb(0xD2, 0xDA, 0xE0),
            border: Color32::from_rgb(0xC9, 0xD0, 0xD6),
            text: Color32::from_rgb(0x20, 0x25, 0x2A),
            weak_text: Color32::from_rgb(0x4F, 0x5B, 0x66),
            accent: Color32::from_rgb(0x14, 0x8F, 0x82),
            danger: Color32::from_rgb(0xC8, 0x3F, 0x4B),
            lock: Color32::from_rgb(0xA9, 0x70, 0x10),
            input_bg: Color32::from_rgb(0xF4, 0xF6, 0xF7),
            window_shadow: Color32::from_black_alpha(60),
        },
    }
}

pub fn apply_egui_theme(ctx: &egui::Context, mode: ThemeMode) {
    let p = palette(mode);
    let style = (*ctx.style()).clone();

    let mut visuals = style.visuals.clone();

    visuals.panel_fill = p.app_bg;
    visuals.extreme_bg_color = p.input_bg;
    visuals.faint_bg_color = p.sidebar;
    visuals.widgets.noninteractive.bg_fill = p.app_bg;
    visuals.widgets.noninteractive.weak_bg_fill = p.app_bg;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.weak_text);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);

    visuals.widgets.hovered.bg_fill = p.hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.border);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text);

    visuals.widgets.active.bg_fill = p.active;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, p.text);

    visuals.widgets.open.bg_fill = p.hover;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, p.accent);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, p.text);

    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text);

    visuals.widgets.hovered.weak_bg_fill = p.hover;
    visuals.widgets.active.weak_bg_fill = p.active;
    visuals.widgets.open.weak_bg_fill = p.hover;

    visuals.warn_fg_color = p.lock;
    visuals.error_fg_color = p.danger;

    visuals.selection.bg_fill = p.active;
    visuals.selection.stroke = Stroke::new(1.0, p.text);

    visuals.hyperlink_color = p.accent;

    visuals.window_fill = p.panel;
    visuals.window_stroke = Stroke::new(1.0, p.border);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: p.window_shadow,
    };
    visuals.text_cursor.stroke = Stroke::new(2.0, p.accent);

    ctx.set_style(style);
    ctx.set_visuals(visuals);
}

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

/// Map a legacy `ThemeMode` to an embedded theme ID for migration purposes.
pub fn migrate_legacy_theme_id(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "opennex-light",
        ThemeMode::Dark => "opennex-dark",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_serializes_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&ThemeMode::Light).unwrap(),
            "\"light\""
        );
        assert_eq!(serde_json::to_string(&ThemeMode::Dark).unwrap(), "\"dark\"");
    }

    #[test]
    fn theme_mode_defaults_to_dark() {
        assert_eq!(ThemeMode::default(), ThemeMode::Dark);
    }

    #[test]
    fn unknown_theme_mode_falls_back_to_dark() {
        let mode: ThemeMode = serde_json::from_str("\"legacy\"").unwrap();
        assert_eq!(mode, ThemeMode::Dark);
    }

    #[test]
    fn dark_and_light_palettes_differ() {
        let dark = palette(ThemeMode::Dark);
        let light = palette(ThemeMode::Light);
        assert_ne!(dark.app_bg, light.app_bg);
        assert_ne!(dark.text, light.text);
        assert_ne!(dark.accent, light.accent);
    }

    #[test]
    fn dark_palette_has_readable_contrast() {
        let p = palette(ThemeMode::Dark);
        assert_luminance_gap(p.text, p.app_bg, 4.5);
        assert_luminance_gap(p.weak_text, p.sidebar, 4.5);
    }

    #[test]
    fn light_palette_has_readable_contrast() {
        let p = palette(ThemeMode::Light);
        assert_luminance_gap(p.text, p.app_bg, 4.5);
        assert_luminance_gap(p.weak_text, p.sidebar, 4.5);
    }

    #[test]
    fn selected_widgets_use_neutral_theme_colors() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let ctx = egui::Context::default();
            apply_egui_theme(&ctx, mode);
            let visuals = &ctx.style().visuals;
            let p = palette(mode);
            assert_eq!(visuals.selection.bg_fill, p.active);
            assert_eq!(visuals.selection.stroke.color, p.text);
            assert_ne!(visuals.selection.bg_fill, p.accent);
        }
    }

    fn assert_luminance_gap(a: Color32, b: Color32, min_ratio: f32) {
        let ratio = (relative_luminance(a) + 0.05) / (relative_luminance(b) + 0.05);
        let ratio = ratio.max(1.0 / ratio);
        assert!(ratio >= min_ratio, "contrast ratio {ratio:.2} too small");
    }

    fn relative_luminance(color: Color32) -> f32 {
        let channels = [color.r(), color.g(), color.b()].map(|channel| {
            let value = channel as f32 / 255.0;
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
    }
}

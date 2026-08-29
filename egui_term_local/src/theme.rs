use alacritty_terminal::vte::ansi::{self, NamedColor};
use egui::Color32;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub foreground: String,
    pub background: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
    pub bright_foreground: Option<String>,
    pub dim_foreground: String,
    pub dim_black: String,
    pub dim_red: String,
    pub dim_green: String,
    pub dim_yellow: String,
    pub dim_blue: String,
    pub dim_magenta: String,
    pub dim_cyan: String,
    pub dim_white: String,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            foreground: String::from("#d8d8d8"),
            background: String::from("#181818"),
            black: String::from("#181818"),
            red: String::from("#ac4242"),
            green: String::from("#90a959"),
            yellow: String::from("#f4bf75"),
            blue: String::from("#6a9fb5"),
            magenta: String::from("#aa759f"),
            cyan: String::from("#75b5aa"),
            white: String::from("#d8d8d8"),
            bright_black: String::from("#6b6b6b"),
            bright_red: String::from("#c55555"),
            bright_green: String::from("#aac474"),
            bright_yellow: String::from("#feca88"),
            bright_blue: String::from("#82b8c8"),
            bright_magenta: String::from("#c28cb8"),
            bright_cyan: String::from("#93d3c3"),
            bright_white: String::from("#f8f8f8"),
            bright_foreground: None,
            dim_foreground: String::from("#828482"),
            dim_black: String::from("#0f0f0f"),
            dim_red: String::from("#712b2b"),
            dim_green: String::from("#5f6f3a"),
            dim_yellow: String::from("#a17e4d"),
            dim_blue: String::from("#456877"),
            dim_magenta: String::from("#704d68"),
            dim_cyan: String::from("#4d7770"),
            dim_white: String::from("#8e8e8e"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TerminalVisualColors {
    pub cursor: Color32,
    pub selection_bg: Color32,
    pub selection_text: Color32,
    pub link: Color32,
}

impl Default for TerminalVisualColors {
    fn default() -> Self {
        let palette = ColorPalette::default();
        TerminalVisualColors {
            cursor: hex_to_color(&palette.foreground).unwrap_or(Color32::WHITE),
            selection_bg: hex_to_color(&palette.bright_black)
                .unwrap_or(Color32::from_rgb(0x33, 0x38, 0x40)),
            selection_text: hex_to_color(&palette.foreground)
                .unwrap_or(Color32::WHITE),
            link: hex_to_color(&palette.blue)
                .unwrap_or(Color32::from_rgb(0x61, 0xaf, 0xef)),
        }
    }
}

/// A terminal theme whose heavyweight parts (palette, ANSI-256 table,
/// and the pre-parsed palette colors) are shared behind an `Arc`. Cloning
/// a `TerminalTheme` is therefore O(1) — the terminal view does it every
/// frame, so a deep clone would rebuild 28 strings + a 256-entry HashMap
/// + 27 color parses per terminal per frame.
#[derive(Debug)]
pub struct TerminalTheme {
    inner: Arc<TerminalThemeInner>,
}

#[derive(Debug)]
struct TerminalThemeInner {
    visual: TerminalVisualColors,
    ansi256_colors: HashMap<u8, Color32>,
    resolved: ResolvedColors,
}

/// Palette colors pre-parsed to `Color32` once at construction. This
/// removes the per-cell `hex_to_color` (a string→int parse) from the hot
/// render path — previously every visible cell did 2 parses per frame.
#[derive(Debug)]
struct ResolvedColors {
    foreground: Color32,
    background: Color32,
    normal: [Color32; 8],
    bright: [Color32; 8],
    bright_foreground: Color32,
    dim_foreground: Color32,
    dim: [Color32; 8],
}

impl ResolvedColors {
    fn from_palette(p: &ColorPalette) -> Self {
        let h = |s: &str| hex_to_color(s).unwrap_or(Color32::WHITE);
        Self {
            foreground: h(&p.foreground),
            background: h(&p.background),
            normal: [
                h(&p.black),
                h(&p.red),
                h(&p.green),
                h(&p.yellow),
                h(&p.blue),
                h(&p.magenta),
                h(&p.cyan),
                h(&p.white),
            ],
            bright: [
                h(&p.bright_black),
                h(&p.bright_red),
                h(&p.bright_green),
                h(&p.bright_yellow),
                h(&p.bright_blue),
                h(&p.bright_magenta),
                h(&p.bright_cyan),
                h(&p.bright_white),
            ],
            bright_foreground: p
                .bright_foreground
                .as_deref()
                .map(h)
                .unwrap_or_else(|| h(&p.foreground)),
            dim_foreground: h(&p.dim_foreground),
            dim: [
                h(&p.dim_black),
                h(&p.dim_red),
                h(&p.dim_green),
                h(&p.dim_yellow),
                h(&p.dim_blue),
                h(&p.dim_magenta),
                h(&p.dim_cyan),
                h(&p.dim_white),
            ],
        }
    }
}

impl Clone for TerminalTheme {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self::from_palette(Box::<ColorPalette>::default())
    }
}

impl TerminalTheme {
    pub fn new(
        palette: Box<ColorPalette>,
        visual: TerminalVisualColors,
    ) -> Self {
        let resolved = ResolvedColors::from_palette(&palette);
        Self {
            inner: Arc::new(TerminalThemeInner {
                visual,
                ansi256_colors: TerminalTheme::get_ansi256_colors(),
                resolved,
            }),
        }
    }

    /// Convenience constructor that derives visual colors from the palette.
    pub fn from_palette(palette: Box<ColorPalette>) -> Self {
        let visual = TerminalVisualColors {
            cursor: hex_to_color(&palette.foreground).unwrap_or(Color32::WHITE),
            selection_bg: hex_to_color(&palette.bright_black)
                .unwrap_or(Color32::from_rgb(0x33, 0x38, 0x40)),
            selection_text: hex_to_color(&palette.foreground)
                .unwrap_or(Color32::WHITE),
            link: hex_to_color(&palette.blue)
                .unwrap_or(Color32::from_rgb(0x61, 0xaf, 0xef)),
        };
        Self::new(palette, visual)
    }

    pub fn cursor_color(&self) -> Color32 {
        self.inner.visual.cursor
    }

    pub fn selection_bg_color(&self) -> Color32 {
        self.inner.visual.selection_bg
    }

    pub fn selection_text_color(&self) -> Color32 {
        self.inner.visual.selection_text
    }

    pub fn link_color(&self) -> Color32 {
        self.inner.visual.link
    }

    fn get_ansi256_colors() -> HashMap<u8, Color32> {
        let mut ansi256_colors = HashMap::new();

        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    // Reserve the first 16 colors for config.
                    let index = 16 + r * 36 + g * 6 + b;
                    let color = Color32::from_rgb(
                        if r == 0 { 0 } else { r * 40 + 55 },
                        if g == 0 { 0 } else { g * 40 + 55 },
                        if b == 0 { 0 } else { b * 40 + 55 },
                    );
                    ansi256_colors.insert(index, color);
                }
            }
        }

        let index: u8 = 232;
        for i in 0..24 {
            let value = i * 10 + 8;
            ansi256_colors
                .insert(index + i, Color32::from_rgb(value, value, value));
        }

        ansi256_colors
    }

    pub fn get_color(&self, c: ansi::Color) -> Color32 {
        match c {
            ansi::Color::Spec(rgb) => Color32::from_rgb(rgb.r, rgb.g, rgb.b),
            ansi::Color::Indexed(index) => {
                if index <= 15 {
                    let i = index as usize;
                    if i < 8 {
                        self.inner.resolved.normal[i]
                    } else {
                        self.inner.resolved.bright[i - 8]
                    }
                } else {
                    self.inner
                        .ansi256_colors
                        .get(&index)
                        .copied()
                        .unwrap_or(Color32::from_rgb(0, 0, 0))
                }
            },
            ansi::Color::Named(c) => {
                let r = &self.inner.resolved;
                match c {
                    NamedColor::Foreground => r.foreground,
                    NamedColor::Background => r.background,
                    NamedColor::Black => r.normal[0],
                    NamedColor::Red => r.normal[1],
                    NamedColor::Green => r.normal[2],
                    NamedColor::Yellow => r.normal[3],
                    NamedColor::Blue => r.normal[4],
                    NamedColor::Magenta => r.normal[5],
                    NamedColor::Cyan => r.normal[6],
                    NamedColor::White => r.normal[7],
                    NamedColor::BrightBlack => r.bright[0],
                    NamedColor::BrightRed => r.bright[1],
                    NamedColor::BrightGreen => r.bright[2],
                    NamedColor::BrightYellow => r.bright[3],
                    NamedColor::BrightBlue => r.bright[4],
                    NamedColor::BrightMagenta => r.bright[5],
                    NamedColor::BrightCyan => r.bright[6],
                    NamedColor::BrightWhite => r.bright[7],
                    NamedColor::BrightForeground => r.bright_foreground,
                    NamedColor::DimForeground => r.dim_foreground,
                    NamedColor::DimBlack => r.dim[0],
                    NamedColor::DimRed => r.dim[1],
                    NamedColor::DimGreen => r.dim[2],
                    NamedColor::DimYellow => r.dim[3],
                    NamedColor::DimBlue => r.dim[4],
                    NamedColor::DimMagenta => r.dim[5],
                    NamedColor::DimCyan => r.dim[6],
                    NamedColor::DimWhite => r.dim[7],
                    _ => r.background,
                }
            },
        }
    }
}

fn hex_to_color(hex: &str) -> anyhow::Result<Color32> {
    let stripped = hex.strip_prefix('#').unwrap_or(hex);
    match stripped.len() {
        6 => {
            let r = u8::from_str_radix(&stripped[0..2], 16)?;
            let g = u8::from_str_radix(&stripped[2..4], 16)?;
            let b = u8::from_str_radix(&stripped[4..6], 16)?;
            Ok(Color32::from_rgb(r, g, b))
        },
        8 => {
            let r = u8::from_str_radix(&stripped[0..2], 16)?;
            let g = u8::from_str_radix(&stripped[2..4], 16)?;
            let b = u8::from_str_radix(&stripped[4..6], 16)?;
            let a = u8::from_str_radix(&stripped[6..8], 16)?;
            Ok(Color32::from_rgba_unmultiplied(r, g, b, a))
        },
        _ => Err(anyhow::format_err!("input string is in non valid format")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_theme_exposes_visual_colors() {
        let theme = TerminalTheme::new(
            Box::new(ColorPalette::default()),
            TerminalVisualColors {
                cursor: Color32::from_rgb(1, 2, 3),
                selection_bg: Color32::from_rgb(4, 5, 6),
                selection_text: Color32::from_rgb(7, 8, 9),
                link: Color32::from_rgb(10, 11, 12),
            },
        );
        assert_eq!(theme.cursor_color(), Color32::from_rgb(1, 2, 3));
        assert_eq!(theme.selection_bg_color(), Color32::from_rgb(4, 5, 6));
        assert_eq!(theme.selection_text_color(), Color32::from_rgb(7, 8, 9));
        assert_eq!(theme.link_color(), Color32::from_rgb(10, 11, 12));
    }

    #[test]
    fn from_palette_derives_consistent_visual_colors() {
        let mut palette = ColorPalette::default();
        palette.foreground = "#abcdef".into();
        palette.blue = "#112233".into();
        let theme = TerminalTheme::from_palette(Box::new(palette));
        assert_eq!(theme.cursor_color(), Color32::from_rgb(0xab, 0xcd, 0xef));
        assert_eq!(theme.link_color(), Color32::from_rgb(0x11, 0x22, 0x33));
    }
}

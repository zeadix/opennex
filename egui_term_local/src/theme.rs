use alacritty_terminal::vte::ansi::{self, NamedColor};
use egui::Color32;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct TerminalTheme {
    palette: Box<ColorPalette>,
    visual: TerminalVisualColors,
    ansi256_colors: HashMap<u8, Color32>,
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self {
            palette: Box::<ColorPalette>::default(),
            visual: TerminalVisualColors::default(),
            ansi256_colors: TerminalTheme::get_ansi256_colors(),
        }
    }
}

impl TerminalTheme {
    pub fn new(
        palette: Box<ColorPalette>,
        visual: TerminalVisualColors,
    ) -> Self {
        Self {
            palette,
            visual,
            ansi256_colors: TerminalTheme::get_ansi256_colors(),
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
        self.visual.cursor
    }

    pub fn selection_bg_color(&self) -> Color32 {
        self.visual.selection_bg
    }

    pub fn selection_text_color(&self) -> Color32 {
        self.visual.selection_text
    }

    pub fn link_color(&self) -> Color32 {
        self.visual.link
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
                    let color = match index {
                        // Normal terminal colors
                        0 => &self.palette.black,
                        1 => &self.palette.red,
                        2 => &self.palette.green,
                        3 => &self.palette.yellow,
                        4 => &self.palette.blue,
                        5 => &self.palette.magenta,
                        6 => &self.palette.cyan,
                        7 => &self.palette.white,
                        // Bright terminal colors
                        8 => &self.palette.bright_black,
                        9 => &self.palette.bright_red,
                        10 => &self.palette.bright_green,
                        11 => &self.palette.bright_yellow,
                        12 => &self.palette.bright_blue,
                        13 => &self.palette.bright_magenta,
                        14 => &self.palette.bright_cyan,
                        15 => &self.palette.bright_white,
                        _ => &self.palette.background,
                    };

                    return hex_to_color(color)
                        .unwrap_or_else(|_| panic!("invalid color {}", color));
                }

                // Other colors
                match self.ansi256_colors.get(&index) {
                    Some(color) => *color,
                    None => Color32::from_rgb(0, 0, 0),
                }
            },
            ansi::Color::Named(c) => {
                let color = match c {
                    NamedColor::Foreground => &self.palette.foreground,
                    NamedColor::Background => &self.palette.background,
                    // Normal terminal colors
                    NamedColor::Black => &self.palette.black,
                    NamedColor::Red => &self.palette.red,
                    NamedColor::Green => &self.palette.green,
                    NamedColor::Yellow => &self.palette.yellow,
                    NamedColor::Blue => &self.palette.blue,
                    NamedColor::Magenta => &self.palette.magenta,
                    NamedColor::Cyan => &self.palette.cyan,
                    NamedColor::White => &self.palette.white,
                    // Bright terminal colors
                    NamedColor::BrightBlack => &self.palette.bright_black,
                    NamedColor::BrightRed => &self.palette.bright_red,
                    NamedColor::BrightGreen => &self.palette.bright_green,
                    NamedColor::BrightYellow => &self.palette.bright_yellow,
                    NamedColor::BrightBlue => &self.palette.bright_blue,
                    NamedColor::BrightMagenta => &self.palette.bright_magenta,
                    NamedColor::BrightCyan => &self.palette.bright_cyan,
                    NamedColor::BrightWhite => &self.palette.bright_white,
                    NamedColor::BrightForeground => {
                        match &self.palette.bright_foreground {
                            Some(color) => color,
                            None => &self.palette.foreground,
                        }
                    },
                    // Dim terminal colors
                    NamedColor::DimForeground => &self.palette.dim_foreground,
                    NamedColor::DimBlack => &self.palette.dim_black,
                    NamedColor::DimRed => &self.palette.dim_red,
                    NamedColor::DimGreen => &self.palette.dim_green,
                    NamedColor::DimYellow => &self.palette.dim_yellow,
                    NamedColor::DimBlue => &self.palette.dim_blue,
                    NamedColor::DimMagenta => &self.palette.dim_magenta,
                    NamedColor::DimCyan => &self.palette.dim_cyan,
                    NamedColor::DimWhite => &self.palette.dim_white,
                    _ => &self.palette.background,
                };

                hex_to_color(color)
                    .unwrap_or_else(|_| panic!("invalid color {}", color))
            },
        }
    }
}

fn hex_to_color(hex: &str) -> anyhow::Result<Color32> {
    if hex.len() != 7 {
        return Err(anyhow::format_err!("input string is in non valid format"));
    }

    let r = u8::from_str_radix(&hex[1..3], 16)?;
    let g = u8::from_str_radix(&hex[3..5], 16)?;
    let b = u8::from_str_radix(&hex[5..7], 16)?;

    Ok(Color32::from_rgb(r, g, b))
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

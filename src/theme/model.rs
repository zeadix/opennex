use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const THEME_FORMAT_VERSION: u32 = 1;

/// A validated `#RRGGBB` or `#RRGGBBAA` color string.
///
/// Stored canonically as lowercase ASCII hex with a leading `#`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeColor([u8; 4]);

impl ThemeColor {
    pub fn parse(input: &str) -> Result<Self> {
        let hex = input
            .strip_prefix('#')
            .ok_or_else(|| anyhow!("color must start with '#': {input}"))?;
        let (r, g, b, a) = match hex.len() {
            6 => (
                u8::from_str_radix(&hex[0..2], 16)?,
                u8::from_str_radix(&hex[2..4], 16)?,
                u8::from_str_radix(&hex[4..6], 16)?,
                255,
            ),
            8 => (
                u8::from_str_radix(&hex[0..2], 16)?,
                u8::from_str_radix(&hex[2..4], 16)?,
                u8::from_str_radix(&hex[4..6], 16)?,
                u8::from_str_radix(&hex[6..8], 16)?,
            ),
            _ => return Err(anyhow!("color must be #RRGGBB or #RRGGBBAA: {input}")),
        };
        Ok(ThemeColor([r, g, b, a]))
    }

    #[cfg(test)]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        ThemeColor([r, g, b, 255])
    }

    /// Construct an opaque RGB color. Used by theme default-value helpers
    /// and theme editors; `from_rgb` is reserved for unit tests.
    pub fn from_rgb_opaque(r: u8, g: u8, b: u8) -> Self {
        ThemeColor([r, g, b, 255])
    }

    pub fn to_array(&self) -> [u8; 4] {
        self.0
    }

    pub fn to_egui(&self) -> egui::Color32 {
        let [r, g, b, a] = self.0;
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    pub fn as_hex(&self) -> String {
        if self.0[3] == 255 {
            format!("#{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2])
        } else {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3]
            )
        }
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_hex())
    }
}

impl Serialize for ThemeColor {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&self.as_hex())
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let s = String::deserialize(de)?;
        ThemeColor::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeError {
    UnsupportedVersion(u32),
    InvalidField { field: String, reason: String },
    MissingDefault,
    Json(String),
    Io(String),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::UnsupportedVersion(v) => {
                write!(f, "unsupported theme format version: {v}")
            }
            ThemeError::InvalidField { field, reason } => {
                write!(f, "invalid field '{field}': {reason}")
            }
            ThemeError::MissingDefault => write!(f, "default theme is missing"),
            ThemeError::Json(msg) => write!(f, "theme JSON error: {msg}"),
            ThemeError::Io(msg) => write!(f, "theme I/O error: {msg}"),
        }
    }
}

impl std::error::Error for ThemeError {}

impl From<serde_json::Error> for ThemeError {
    fn from(err: serde_json::Error) -> Self {
        ThemeError::Json(err.to_string())
    }
}

impl From<std::io::Error> for ThemeError {
    fn from(err: std::io::Error) -> Self {
        ThemeError::Io(err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnsiColors {
    pub black: ThemeColor,
    pub red: ThemeColor,
    pub green: ThemeColor,
    pub yellow: ThemeColor,
    pub blue: ThemeColor,
    pub magenta: ThemeColor,
    pub cyan: ThemeColor,
    pub white: ThemeColor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppTheme {
    pub app_bg: ThemeColor,
    pub sidebar: ThemeColor,
    pub panel: ThemeColor,
    pub hover: ThemeColor,
    pub active: ThemeColor,
    pub border: ThemeColor,
    pub text: ThemeColor,
    pub weak_text: ThemeColor,
    pub accent: ThemeColor,
    pub warning: ThemeColor,
    pub danger: ThemeColor,
    pub lock: ThemeColor,
    pub input_bg: ThemeColor,
    pub selection_bg: ThemeColor,
    pub selection_text: ThemeColor,
    pub window_shadow: ThemeColor,
    #[serde(default = "default_ui_font_families")]
    pub ui_font_families: Vec<String>,
    #[serde(default = "default_ui_font_size")]
    pub ui_font_size: f32,
    // === Minimalist UI chrome fields (all optional for backward compat) ===
    /// Background of the top menu bar and the bottom status bar.
    #[serde(default = "default_menu_bg")]
    pub menu_bg: ThemeColor,
    /// Foreground (text) of the top menu bar and bottom status bar.
    #[serde(default = "default_menu_fg")]
    pub menu_fg: ThemeColor,
    /// Background of inline buttons (e.g. "[+ 新建]", 模板 ▼).
    #[serde(default = "default_button_bg")]
    pub button_bg: ThemeColor,
    /// Foreground (text) of inline buttons.
    #[serde(default = "default_button_fg")]
    pub button_fg: ThemeColor,
    /// Background of inline buttons on hover.
    #[serde(default = "default_button_hover_bg")]
    pub button_hover_bg: ThemeColor,
    /// Right border of the workspace sidebar (1px line).
    #[serde(default = "default_sidebar_border")]
    pub sidebar_border: ThemeColor,
}

fn default_ui_font_families() -> Vec<String> {
    vec!["system-ui".into()]
}

fn default_ui_font_size() -> f32 {
    14.0
}

fn default_menu_bg() -> ThemeColor {
    // Default matches `panel` (filled in by apply when missing from JSON).
    ThemeColor::from_rgb_opaque(0x14, 0x14, 0x17)
}
fn default_menu_fg() -> ThemeColor {
    ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed)
}
fn default_button_bg() -> ThemeColor {
    ThemeColor::from_rgb_opaque(0x1a, 0x1a, 0x1d)
}
fn default_button_fg() -> ThemeColor {
    ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed)
}
fn default_button_hover_bg() -> ThemeColor {
    ThemeColor::from_rgb_opaque(0x26, 0x26, 0x2a)
}
fn default_sidebar_border() -> ThemeColor {
    ThemeColor::from_rgb_opaque(0x26, 0x26, 0x2a)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalThemeConfig {
    pub foreground: ThemeColor,
    pub background: ThemeColor,
    pub normal: AnsiColors,
    pub bright: AnsiColors,
    pub dim: AnsiColors,
    pub dim_foreground: ThemeColor,
    pub cursor: ThemeColor,
    pub selection_bg: ThemeColor,
    pub selection_text: ThemeColor,
    pub link: ThemeColor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypographyTheme {
    pub terminal_font_families: Vec<String>,
    pub terminal_font_size: f32,
    pub cell_spacing: f32,
    pub menu_font_size: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub format_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: String,
    pub app: AppTheme,
    pub terminal: TerminalThemeConfig,
    pub typography: TypographyTheme,
}

impl ThemeDefinition {
    /// Validate the theme against the format-1 rules.
    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.format_version != THEME_FORMAT_VERSION {
            return Err(ThemeError::UnsupportedVersion(self.format_version));
        }

        if self.id.is_empty()
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Err(ThemeError::InvalidField {
                field: "id".into(),
                reason: "must contain only ASCII lowercase, digits, '-' or '_'".into(),
            });
        }

        validate_str_len(&self.name, "name", 80, true)?;
        validate_str_len(&self.author, "author", 80, false)?;

        if self.typography.terminal_font_families.is_empty() {
            return Err(ThemeError::InvalidField {
                field: "terminal_font_families".into(),
                reason: "must contain at least one font family".into(),
            });
        }
        if self
            .typography
            .terminal_font_families
            .iter()
            .any(|f| f.trim().is_empty())
        {
            return Err(ThemeError::InvalidField {
                field: "terminal_font_families".into(),
                reason: "entries must be non-empty".into(),
            });
        }

        validate_range(
            self.typography.terminal_font_size,
            "terminal_font_size",
            8.0,
            32.0,
        )?;
        validate_range(self.typography.cell_spacing, "cell_spacing", 0.5, 2.0)?;
        validate_range(self.typography.menu_font_size, "menu_font_size", 8.0, 32.0)?;

        if self.app.ui_font_families.is_empty() {
            return Err(ThemeError::InvalidField {
                field: "ui_font_families".into(),
                reason: "must contain at least one font family".into(),
            });
        }
        validate_range(self.app.ui_font_size, "ui_font_size", 8.0, 32.0)?;

        Ok(())
    }
}

fn validate_str_len(
    value: &str,
    field: &str,
    max: usize,
    non_empty: bool,
) -> Result<(), ThemeError> {
    let count = value.chars().count();
    if (non_empty && count == 0) || count > max {
        return Err(ThemeError::InvalidField {
            field: field.into(),
            reason: if non_empty && count == 0 {
                "must not be empty".into()
            } else {
                format!("must be at most {max} characters")
            },
        });
    }
    Ok(())
}

fn validate_range(value: f32, field: &str, min: f32, max: f32) -> Result<(), ThemeError> {
    if !(min..=max).contains(&value) {
        return Err(ThemeError::InvalidField {
            field: field.into(),
            reason: format!("must be in range {min}..={max}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ThemeDefinition {
        /// Minimal valid theme used only by model-level tests.
        pub fn opennex_dark_for_test() -> Self {
            ThemeDefinition {
                format_version: THEME_FORMAT_VERSION,
                id: "opennex-dark".into(),
                name: "OpenNex Dark".into(),
                author: String::new(),
                app: AppTheme {
                    app_bg: ThemeColor::from_rgb_opaque(0x17, 0x19, 0x1c),
                    sidebar: ThemeColor::from_rgb_opaque(0x1d, 0x20, 0x24),
                    panel: ThemeColor::from_rgb_opaque(0x24, 0x27, 0x2c),
                    hover: ThemeColor::from_rgb_opaque(0x2e, 0x33, 0x3a),
                    active: ThemeColor::from_rgb_opaque(0x33, 0x38, 0x40),
                    border: ThemeColor::from_rgb_opaque(0x34, 0x3a, 0x40),
                    text: ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed),
                    weak_text: ThemeColor::from_rgb_opaque(0x8c, 0x95, 0x9f),
                    accent: ThemeColor::from_rgb_opaque(0x2c, 0xbf, 0xae),
                    warning: ThemeColor::from_rgb_opaque(0xd9, 0xa4, 0x41),
                    danger: ThemeColor::from_rgb_opaque(0xe0, 0x5a, 0x65),
                    lock: ThemeColor::from_rgb_opaque(0xd9, 0xa4, 0x41),
                    input_bg: ThemeColor::from_rgb_opaque(0x1a, 0x1d, 0x21),
                    selection_bg: ThemeColor::from_rgb_opaque(0x33, 0x38, 0x40),
                    selection_text: ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed),
                    window_shadow: ThemeColor::from_rgb_opaque(0x00, 0x00, 0x00),
                    ui_font_families: vec!["system-ui".into()],
                    ui_font_size: 14.0,
                    menu_bg: ThemeColor::from_rgb_opaque(0x14, 0x14, 0x17),
                    menu_fg: ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed),
                    button_bg: ThemeColor::from_rgb_opaque(0x1a, 0x1a, 0x1d),
                    button_fg: ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed),
                    button_hover_bg: ThemeColor::from_rgb_opaque(0x26, 0x26, 0x2a),
                    sidebar_border: ThemeColor::from_rgb_opaque(0x26, 0x26, 0x2a),
                },
                terminal: TerminalThemeConfig {
                    foreground: ThemeColor::from_rgb_opaque(0xab, 0xb2, 0xbf),
                    background: ThemeColor::from_rgb_opaque(0x28, 0x2c, 0x34),
                    normal: AnsiColors {
                        black: ThemeColor::from_rgb_opaque(0x28, 0x2c, 0x34),
                        red: ThemeColor::from_rgb_opaque(0xe0, 0x6c, 0x75),
                        green: ThemeColor::from_rgb_opaque(0x98, 0xc3, 0x79),
                        yellow: ThemeColor::from_rgb_opaque(0xe5, 0xc0, 0x7b),
                        blue: ThemeColor::from_rgb_opaque(0x61, 0xaf, 0xef),
                        magenta: ThemeColor::from_rgb_opaque(0xc6, 0x78, 0xdd),
                        cyan: ThemeColor::from_rgb_opaque(0x56, 0xb6, 0xc2),
                        white: ThemeColor::from_rgb_opaque(0xab, 0xb2, 0xbf),
                    },
                    bright: AnsiColors {
                        black: ThemeColor::from_rgb_opaque(0x5c, 0x63, 0x70),
                        red: ThemeColor::from_rgb_opaque(0xe0, 0x6c, 0x75),
                        green: ThemeColor::from_rgb_opaque(0x98, 0xc3, 0x79),
                        yellow: ThemeColor::from_rgb_opaque(0xe5, 0xc0, 0x7b),
                        blue: ThemeColor::from_rgb_opaque(0x61, 0xaf, 0xef),
                        magenta: ThemeColor::from_rgb_opaque(0xc6, 0x78, 0xdd),
                        cyan: ThemeColor::from_rgb_opaque(0x56, 0xb6, 0xc2),
                        white: ThemeColor::from_rgb_opaque(0xff, 0xff, 0xff),
                    },
                    dim: AnsiColors {
                        black: ThemeColor::from_rgb_opaque(0x1c, 0x20, 0x26),
                        red: ThemeColor::from_rgb_opaque(0x9c, 0x4f, 0x56),
                        green: ThemeColor::from_rgb_opaque(0x6c, 0x8f, 0x57),
                        yellow: ThemeColor::from_rgb_opaque(0xa6, 0x89, 0x5a),
                        blue: ThemeColor::from_rgb_opaque(0x47, 0x7c, 0xab),
                        magenta: ThemeColor::from_rgb_opaque(0x8d, 0x55, 0x9d),
                        cyan: ThemeColor::from_rgb_opaque(0x3d, 0x82, 0x8a),
                        white: ThemeColor::from_rgb_opaque(0x75, 0x7b, 0x85),
                    },
                    dim_foreground: ThemeColor::from_rgb_opaque(0x6b, 0x72, 0x80),
                    cursor: ThemeColor::from_rgb_opaque(0x2c, 0xbf, 0xae),
                    selection_bg: ThemeColor::from_rgb_opaque(0x33, 0x38, 0x40),
                    selection_text: ThemeColor::from_rgb_opaque(0xe6, 0xe9, 0xed),
                    link: ThemeColor::from_rgb_opaque(0x61, 0xaf, 0xef),
                },
                typography: TypographyTheme {
                    terminal_font_families: vec!["monospace".into()],
                    terminal_font_size: 14.0,
                    cell_spacing: 1.0,
                    menu_font_size: 14.0,
                },
            }
        }
    }

    #[test]
    fn theme_color_accepts_rgb_and_rgba() {
        assert_eq!(
            ThemeColor::parse("#61AFEF").unwrap().to_array(),
            [0x61, 0xaf, 0xef, 0xff]
        );
        assert_eq!(
            ThemeColor::parse("#11223380").unwrap().to_array(),
            [0x11, 0x22, 0x33, 0x80]
        );
    }

    #[test]
    fn theme_color_rejects_invalid_values() {
        assert!(ThemeColor::parse("61afef").is_err());
        assert!(ThemeColor::parse("#xyzxyz").is_err());
        assert!(ThemeColor::parse("#12345").is_err());
        assert!(ThemeColor::parse("#1234567").is_err());
    }

    #[test]
    fn theme_color_round_trips_as_lowercase_hex() {
        let color = ThemeColor::parse("#ABCDEF12").unwrap();
        assert_eq!(color.as_hex(), "#abcdef12");
        let parsed =
            serde_json::from_str::<ThemeColor>(&serde_json::to_string(&color).unwrap()).unwrap();
        assert_eq!(parsed, color);
    }

    #[test]
    fn validation_rejects_unknown_version_and_out_of_range_sizes() {
        let mut theme = ThemeDefinition::opennex_dark_for_test();
        theme.format_version = 2;
        assert!(matches!(
            theme.validate(),
            Err(ThemeError::UnsupportedVersion(2))
        ));
        theme.format_version = THEME_FORMAT_VERSION;
        theme.typography.terminal_font_size = 100.0;
        assert!(matches!(
            theme.validate(),
            Err(ThemeError::InvalidField { .. })
        ));
    }

    #[test]
    fn validation_rejects_bad_id_and_empty_font_list() {
        let mut theme = ThemeDefinition::opennex_dark_for_test();
        theme.id = "OpenNex Dark".into();
        assert!(matches!(
            theme.validate(),
            Err(ThemeError::InvalidField { field, .. }) if field == "id"
        ));
        theme.id = "opennex-dark".into();
        theme.typography.terminal_font_families = Vec::new();
        assert!(matches!(
            theme.validate(),
            Err(ThemeError::InvalidField { field, .. }) if field == "terminal_font_families"
        ));
    }

    #[test]
    fn theme_round_trips_without_data_loss() {
        let theme = ThemeDefinition::opennex_dark_for_test();
        let json = serde_json::to_string_pretty(&theme).unwrap();
        assert_eq!(
            serde_json::from_str::<ThemeDefinition>(&json).unwrap(),
            theme
        );
    }
}

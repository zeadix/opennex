use crate::theme::model::{AnsiColors, TerminalThemeConfig, ThemeColor};

/// A built-in terminal color palette template used for quick-fill only.
///
/// Templates are NOT stored as references in themes. Applying a template copies
/// concrete color values into the current theme's terminal config. The saved
/// theme is fully self-contained and does not depend on the template existing.
pub struct TerminalPaletteTemplate {
    pub id: &'static str,
    pub name: &'static str,
}

pub fn templates() -> [TerminalPaletteTemplate; 5] {
    [
        TerminalPaletteTemplate {
            id: "one-dark",
            name: "One Dark",
        },
        TerminalPaletteTemplate {
            id: "solarized-dark",
            name: "Solarized Dark",
        },
        TerminalPaletteTemplate {
            id: "gruvbox-dark",
            name: "Gruvbox Dark",
        },
        TerminalPaletteTemplate {
            id: "dracula",
            name: "Dracula",
        },
        TerminalPaletteTemplate {
            id: "opennex-light-terminal",
            name: "OpenNex Light Terminal",
        },
    ]
}

/// Return the terminal colors for a template, or `None` if the ID is unknown.
pub fn terminal_colors(template_id: &str) -> Option<TerminalThemeConfig> {
    Some(match template_id {
        "one-dark" => one_dark(),
        "solarized-dark" => solarized_dark(),
        "gruvbox-dark" => gruvbox_dark(),
        "dracula" => dracula(),
        "opennex-light-terminal" => opennex_light_terminal(),
        _ => return None,
    })
}

fn rgb(hex: &str) -> ThemeColor {
    ThemeColor::parse(hex).unwrap()
}

fn ansi(
    black: &str,
    red: &str,
    green: &str,
    yellow: &str,
    blue: &str,
    magenta: &str,
    cyan: &str,
    white: &str,
) -> AnsiColors {
    AnsiColors {
        black: rgb(black),
        red: rgb(red),
        green: rgb(green),
        yellow: rgb(yellow),
        blue: rgb(blue),
        magenta: rgb(magenta),
        cyan: rgb(cyan),
        white: rgb(white),
    }
}

fn config(
    fg: &str,
    bg: &str,
    normal: AnsiColors,
    bright: AnsiColors,
    dim: AnsiColors,
    dim_fg: &str,
    cursor: &str,
    sel_bg: &str,
    sel_text: &str,
    link: &str,
) -> TerminalThemeConfig {
    TerminalThemeConfig {
        foreground: rgb(fg),
        background: rgb(bg),
        normal,
        bright,
        dim,
        dim_foreground: rgb(dim_fg),
        cursor: rgb(cursor),
        selection_bg: rgb(sel_bg),
        selection_text: rgb(sel_text),
        link: rgb(link),
    }
}

fn one_dark() -> TerminalThemeConfig {
    config(
        "#abb2bf",
        "#282c34",
        ansi(
            "#282c34", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#abb2bf",
        ),
        ansi(
            "#5c6370", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#ffffff",
        ),
        ansi(
            "#1c2026", "#9c4f56", "#6c8f57", "#a6895a", "#477cab", "#8d559d", "#3d828a", "#757b85",
        ),
        "#6b7280",
        "#2cbfae",
        "#333840",
        "#e6e9ed",
        "#61afef",
    )
}

fn solarized_dark() -> TerminalThemeConfig {
    config(
        "#839496",
        "#002b36",
        ansi(
            "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5",
        ),
        ansi(
            "#586e75", "#cb4b16", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#fdf6e3",
        ),
        ansi(
            "#053440", "#a22824", "#667600", "#886800", "#1d6a9c", "#a02963", "#207a72", "#b6b3a6",
        ),
        "#657b83",
        "#268bd2",
        "#185d6f",
        "#eee8d5",
        "#2aa198",
    )
}

fn gruvbox_dark() -> TerminalThemeConfig {
    config(
        "#ebdbb2",
        "#282828",
        ansi(
            "#282828", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a", "#a89984",
        ),
        ansi(
            "#928374", "#fb4934", "#b8bb26", "#fabd2f", "#83a598", "#d3869b", "#8ec07c", "#ebdbb2",
        ),
        ansi(
            "#1d2021", "#9d1a15", "#7a7014", "#a77619", "#36656a", "#894a69", "#4f7452", "#837768",
        ),
        "#a89984",
        "#fabd2f",
        "#504945",
        "#ebdbb2",
        "#83a598",
    )
}

fn dracula() -> TerminalThemeConfig {
    config(
        "#f8f8f2",
        "#282a36",
        ansi(
            "#21222c", "#ff5555", "#50fa7b", "#f1fa8c", "#bd93f9", "#ff79c6", "#8be9fd", "#f8f8f2",
        ),
        ansi(
            "#6272a4", "#ff6e6e", "#69ff94", "#ffffa5", "#d6acff", "#ff92df", "#a4ffff", "#ffffff",
        ),
        ansi(
            "#1a1c24", "#bf4040", "#3cbb5d", "#b4bc69", "#8d6ebb", "#bf5a95", "#68aebe", "#bbbbbb",
        ),
        "#9a9a9a",
        "#bd93f9",
        "#44475a",
        "#f8f8f2",
        "#8be9fd",
    )
}

fn opennex_light_terminal() -> TerminalThemeConfig {
    config(
        "#4f5b66",
        "#fdf6e3",
        ansi(
            "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5",
        ),
        ansi(
            "#586e75", "#cb4b16", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#fdf6e3",
        ),
        ansi(
            "#053440", "#a22824", "#667600", "#886800", "#1d6a9c", "#a02963", "#207a72", "#b6b3a6",
        ),
        "#657b83",
        "#148f82",
        "#d2dae0",
        "#20252a",
        "#268bd2",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_returns_valid_terminal_colors() {
        for template in templates() {
            let colors = terminal_colors(template.id)
                .unwrap_or_else(|| panic!("template '{}' returned no colors", template.id));
            assert!(!colors.foreground.as_hex().is_empty());
        }
    }

    #[test]
    fn unknown_template_returns_none() {
        assert!(terminal_colors("nonexistent").is_none());
    }

    #[test]
    fn templates_have_unique_ids() {
        let ids: std::collections::HashSet<_> = templates().iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), templates().len());
    }
}

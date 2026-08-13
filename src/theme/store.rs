use std::path::{Path, PathBuf};

use crate::theme::model::{ThemeDefinition, ThemeError};

const EMBEDDED_THEME_JSON: [&str; 5] = [
    include_str!("../../assets/themes/opennex-dark.json"),
    include_str!("../../assets/themes/opennex-light.json"),
    include_str!("../../assets/themes/solarized-dark.json"),
    include_str!("../../assets/themes/gruvbox-dark.json"),
    include_str!("../../assets/themes/dracula.json"),
];

/// Parse and validate a theme document. No unvalidated theme may leave the store API.
pub fn parse_theme(json: &str) -> Result<ThemeDefinition, ThemeError> {
    let theme: ThemeDefinition = serde_json::from_str(json)?;
    theme.validate()?;
    Ok(theme)
}

/// Every theme shipped with the application binary.
pub fn embedded_themes() -> Result<Vec<ThemeDefinition>, ThemeError> {
    EMBEDDED_THEME_JSON.iter().map(|json| parse_theme(json)).collect()
}

/// The canonical default theme used on first launch and on missing IDs.
pub fn default_theme() -> Result<ThemeDefinition, ThemeError> {
    embedded_themes()?
        .into_iter()
        .find(|theme| theme.id == "opennex-dark")
        .ok_or(ThemeError::MissingDefault)
}

/// Whether `id` matches an embedded (read-only) theme.
pub fn is_embedded_id(id: &str) -> bool {
    embedded_themes()
        .map(|themes| themes.iter().any(|theme| theme.id == id))
        .unwrap_or(false)
}

/// Resolve a theme by ID, preferring user themes, then embedded, then default.
pub fn load_theme(user_dir: &Path, id: &str) -> Result<ThemeDefinition, ThemeError> {
    if let Ok(user) = load_user_theme(user_dir, id) {
        return Ok(user);
    }
    if let Some(theme) = embedded_themes()?.into_iter().find(|t| t.id == id) {
        return Ok(theme);
    }
    log::warn!("theme '{id}' not found, falling back to default theme");
    default_theme()
}

/// File-system location for user themes inside the application data directory.
pub fn themes_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("themes")
}

fn load_user_theme(dir: &Path, id: &str) -> Result<ThemeDefinition, ThemeError> {
    let path = dir.join(format!("{id}.json"));
    if !path.exists() {
        return Err(ThemeError::Io(format!("theme file not found: {}", path.display())));
    }
    let json = std::fs::read_to_string(&path)?;
    parse_theme(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_embedded_theme_is_valid_and_has_a_unique_id() {
        let themes = embedded_themes().unwrap();
        assert_eq!(themes.len(), 5);
        let ids: std::collections::HashSet<_> = themes.iter().map(|t| &t.id).collect();
        assert_eq!(ids.len(), themes.len());
        assert!(themes.iter().all(|theme| theme.validate().is_ok()));
    }

    #[test]
    fn default_embedded_theme_is_opennex_dark() {
        assert_eq!(default_theme().unwrap().id, "opennex-dark");
    }

    #[test]
    fn parse_theme_rejects_invalid_json_and_invalid_themes() {
        assert!(parse_theme("{not-json}").is_err());
        let mut theme = default_theme().unwrap();
        theme.format_version = 99;
        let json = serde_json::to_string(&theme).unwrap();
        assert!(parse_theme(&json).is_err());
    }
}

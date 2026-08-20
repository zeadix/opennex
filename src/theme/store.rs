use std::path::{Path, PathBuf};

use crate::theme::model::{ThemeDefinition, ThemeError};

const EMBEDDED_THEME_JSON: [&str; 11] = [
    include_str!("../../assets/themes/opennex-dark.json"),
    include_str!("../../assets/themes/opennex-light.json"),
    include_str!("../../assets/themes/opennex-noir.json"),
    include_str!("../../assets/themes/solarized-dark.json"),
    include_str!("../../assets/themes/gruvbox-dark.json"),
    include_str!("../../assets/themes/dracula.json"),
    include_str!("../../assets/themes/nord.json"),
    include_str!("../../assets/themes/tokyo-night.json"),
    include_str!("../../assets/themes/one-dark.json"),
    include_str!("../../assets/themes/monokai-pro.json"),
    include_str!("../../assets/themes/catppuccin-mocha.json"),
];

/// Parse and validate a theme document. No unvalidated theme may leave the store API.
pub fn parse_theme(json: &str) -> Result<ThemeDefinition, ThemeError> {
    let theme: ThemeDefinition = serde_json::from_str(json)?;
    theme.validate()?;
    Ok(theme)
}

/// Every theme shipped with the application binary.
pub fn embedded_themes() -> Result<Vec<ThemeDefinition>, ThemeError> {
    EMBEDDED_THEME_JSON
        .iter()
        .map(|json| parse_theme(json))
        .collect()
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

/// Discover all valid user themes under `dir`, sorted by lowercase display name.
///
/// Malformed files are logged and skipped so one bad file cannot break the picker.
pub fn load_user_themes(dir: &Path) -> Result<Vec<ThemeDefinition>, ThemeError> {
    let mut themes = Vec::new();
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(themes),
        Err(err) => return Err(ThemeError::Io(err.to_string())),
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match std::fs::read_to_string(&path)
            .and_then(|s| parse_theme(&s).map_err(std::io::Error::other))
        {
            Ok(theme) => themes.push(theme),
            Err(err) => {
                log::warn!("skipping invalid theme file {}: {err}", path.display());
            }
        }
    }
    themes.sort_by_key(|t| t.name.to_lowercase());
    Ok(themes)
}

/// Persist a user theme using a temp file + backup + rename for crash safety.
///
/// Writes `<id>.json.tmp`, syncs it, moves the existing target to
/// `<id>.json.bak`, renames the temp into place, then removes the backup.
/// On failure the backup is restored.
pub fn save_user_theme(dir: &Path, theme: &ThemeDefinition) -> Result<(), ThemeError> {
    std::fs::create_dir_all(dir)?;
    let target = dir.join(format!("{}.json", theme.id));
    let tmp = dir.join(format!("{}.json.tmp", theme.id));
    let backup = dir.join(format!("{}.json.bak", theme.id));

    // Stamp creation time on first save (imports and legacy files).
    let mut theme = theme.clone();
    if theme.created_at == 0 {
        theme.created_at = now_unix();
    }
    let json = serde_json::to_string_pretty(&theme)?;
    {
        let mut file = std::fs::File::create(&tmp)?;
        use std::io::Write;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
    }

    let had_existing = target.exists();
    if had_existing {
        let _ = std::fs::remove_file(&backup);
        std::fs::rename(&target, &backup)?;
    }

    if let Err(err) = std::fs::rename(&tmp, &target) {
        if had_existing {
            let _ = std::fs::rename(&backup, &target);
        }
        return Err(ThemeError::Io(err.to_string()));
    }

    let _ = std::fs::remove_file(&backup);
    Ok(())
}

/// Read a theme file from disk, validate it, and persist a copy under `dir`.
///
/// Validates before creating any files. On ID collision with an existing user
/// theme or an embedded theme, a free `id-N` variant is chosen.
pub fn import_theme_file(dir: &Path, source: &Path) -> Result<ThemeDefinition, ThemeError> {
    let json = std::fs::read_to_string(source)?;
    import_theme_json(dir, &json)
}

/// Import a theme from an in-memory JSON document.
pub fn import_theme_json(dir: &Path, json: &str) -> Result<ThemeDefinition, ThemeError> {
    let mut theme = parse_theme(json)?;
    theme.id = find_free_import_id(dir, &theme.id);
    save_user_theme(dir, &theme)?;
    Ok(theme)
}

/// Export a theme to an explicit destination path.
pub fn export_theme_file(theme: &ThemeDefinition, destination: &Path) -> Result<(), ThemeError> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(theme)?;
    std::fs::write(destination, json)?;
    Ok(())
}

/// Delete a user theme file by ID. Embedded themes cannot be deleted.
pub fn delete_user_theme(dir: &Path, id: &str) -> Result<(), ThemeError> {
    if is_embedded_id(id) {
        return Err(ThemeError::InvalidField {
            field: "id".into(),
            reason: "embedded themes cannot be deleted".into(),
        });
    }
    let path = dir.join(format!("{id}.json"));
    if path.exists() {
        std::fs::remove_file(&path)?;
    } else {
        return Err(ThemeError::Io(format!(
            "theme file not found: {}",
            path.display()
        )));
    }
    Ok(())
}

/// Rename a user theme by updating its display name. ID and filename stay stable.
pub fn rename_user_theme(
    dir: &Path,
    id: &str,
    new_name: &str,
) -> Result<ThemeDefinition, ThemeError> {
    let mut theme = load_theme(dir, id)?;
    theme.name = new_name.to_string();
    theme.validate()?;
    save_user_theme(dir, &theme)?;
    Ok(theme)
}

/// Create a copy of a theme with a new auto-generated ID and display name.
pub fn copy_theme(
    dir: &Path,
    source_id: &str,
    display_name: &str,
) -> Result<ThemeDefinition, ThemeError> {
    let mut theme = load_theme(dir, source_id)?;
    let new_id = find_free_import_id(dir, &theme.id);
    theme.id = new_id;
    theme.name = display_name.to_string();
    theme.created_at = now_unix();
    save_user_theme(dir, &theme)?;
    Ok(theme)
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Pick a non-colliding ID for a theme by appending `-2`, `-3`, ...
pub fn find_free_id(dir: &Path, requested: &str) -> String {
    find_free_import_id(dir, requested)
}

/// Pick a non-colliding ID for an imported theme by appending `-2`, `-3`, ...
fn find_free_import_id(dir: &Path, requested: &str) -> String {
    let taken_ids: std::collections::HashSet<String> = load_user_themes(dir)
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.id)
        .collect();
    let embedded_ids: std::collections::HashSet<String> = embedded_themes()
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.id)
        .collect();
    if !taken_ids.contains(requested) && !embedded_ids.contains(requested) {
        return requested.to_string();
    }
    for suffix in 2..u32::MAX {
        let candidate = format!("{requested}-{suffix}");
        if !taken_ids.contains(&candidate) && !embedded_ids.contains(&candidate) {
            return candidate;
        }
    }
    format!("{requested}-imported")
}

fn load_user_theme(dir: &Path, id: &str) -> Result<ThemeDefinition, ThemeError> {
    let path = dir.join(format!("{id}.json"));
    if !path.exists() {
        return Err(ThemeError::Io(format!(
            "theme file not found: {}",
            path.display()
        )));
    }
    let json = std::fs::read_to_string(&path)?;
    parse_theme(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let dir =
                std::env::temp_dir().join(format!("opennex-theme-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn every_embedded_theme_is_valid_and_has_a_unique_id() {
        let themes = embedded_themes().unwrap();
        assert_eq!(themes.len(), EMBEDDED_THEME_JSON.len());
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

    #[test]
    fn save_user_theme_writes_json_file_atomically() {
        let dir = TempDir::new();
        let theme = default_theme().unwrap();
        save_user_theme(dir.path(), &theme).unwrap();
        let path = dir.path().join("opennex-dark.json");
        assert!(path.exists());
        let reloaded = load_user_theme(dir.path(), "opennex-dark").unwrap();
        // created_at is stamped on first save; compare with it ignored.
        let mut expected = theme.clone();
        expected.created_at = reloaded.created_at;
        assert!(reloaded.created_at > 0);
        assert_eq!(reloaded, expected);
        assert!(!dir.path().join("opennex-dark.json.tmp").exists());
        assert!(!dir.path().join("opennex-dark.json.bak").exists());
    }

    #[test]
    fn load_user_themes_ignores_malformed_files_and_sorts_by_name() {
        let dir = TempDir::new();
        let mut a = default_theme().unwrap();
        a.id = "zebra".into();
        a.name = "Zebra".into();
        let mut b = default_theme().unwrap();
        b.id = "alpha".into();
        b.name = "Alpha".into();
        save_user_theme(dir.path(), &a).unwrap();
        save_user_theme(dir.path(), &b).unwrap();
        std::fs::write(dir.path().join("broken.json"), "{not-json}").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "ignore me").unwrap();

        let themes = load_user_themes(dir.path()).unwrap();
        assert_eq!(themes.len(), 2);
        assert_eq!(themes[0].name, "Alpha");
        assert_eq!(themes[1].name, "Zebra");
    }

    #[test]
    fn import_collision_creates_a_new_id_without_overwriting() {
        let dir = TempDir::new();
        let theme = default_theme().unwrap();
        save_user_theme(dir.path(), &theme).unwrap();
        let json = serde_json::to_string(&theme).unwrap();
        let imported = import_theme_json(dir.path(), &json).unwrap();
        assert_eq!(imported.id, "opennex-dark-2");
        assert!(dir.path().join("opennex-dark.json").exists());
        assert!(dir.path().join("opennex-dark-2.json").exists());
    }

    #[test]
    fn import_collision_increments_until_free_id_found() {
        let dir = TempDir::new();
        let theme = default_theme().unwrap();
        for id_suffix in &["", "-2", "-3"] {
            let mut t = theme.clone();
            t.id = format!("opennex-dark{id_suffix}");
            save_user_theme(dir.path(), &t).unwrap();
        }
        let json = serde_json::to_string(&theme).unwrap();
        let imported = import_theme_json(dir.path(), &json).unwrap();
        assert_eq!(imported.id, "opennex-dark-4");
    }

    #[test]
    fn failed_import_leaves_existing_files_unchanged() {
        let dir = TempDir::new();
        assert!(import_theme_json(dir.path(), "{not-json}").is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn export_theme_file_round_trips() {
        let dir = TempDir::new();
        let theme = default_theme().unwrap();
        let dest = dir.path().join("exported.json");
        export_theme_file(&theme, &dest).unwrap();
        let reloaded = parse_theme(&std::fs::read_to_string(&dest).unwrap()).unwrap();
        assert_eq!(reloaded, theme);
    }

    #[test]
    fn load_theme_falls_back_to_embedded_then_default() {
        let dir = TempDir::new();
        let via_embedded = load_theme(dir.path(), "dracula").unwrap();
        assert_eq!(via_embedded.id, "dracula");
        let via_default = load_theme(dir.path(), "does-not-exist").unwrap();
        assert_eq!(via_default.id, "opennex-dark");
    }
}

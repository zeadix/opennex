# Cross-Platform Visual Theme Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a versioned, cross-platform OpenNex visual theme system with embedded defaults, safe user persistence, live editing, and portable import/export.

**Architecture:** Replace the split hard-coded app and terminal palettes with one serializable `ThemeDefinition` owned by a focused theme module. Keep behavioral preferences in `AppSettings`, store user themes under the application data directory, and pass a validated runtime theme to egui and egui_term. Embedded JSON assets are the release defaults; user files override only by selected ID and never modify shell configuration.

**Tech Stack:** Rust 2021, serde/serde_json, egui 0.31, eframe 0.31, local egui_term, rfd 0.15, anyhow

---

## File Map

- Create `assets/themes/opennex-dark.json`: canonical embedded default theme.
- Create `assets/themes/opennex-light.json`: complete light application and terminal theme.
- Create `assets/themes/solarized-dark.json`, `assets/themes/gruvbox-dark.json`, `assets/themes/dracula.json`: migrate current terminal presets into complete themes.
- Create `src/theme/model.rs`: serialized schema, color type, validation, and runtime conversion.
- Create `src/theme/store.rs`: embedded theme registry, user theme discovery, import/export, ID collision handling, and atomic writes.
- Create `src/theme/ui.rs`: settings editor and preview widgets, keeping `app.rs` integration small.
- Replace `src/theme.rs` with `src/theme/mod.rs`: public theme API and egui application.
- Delete `src/terminal_theme.rs`: its palettes move to versioned JSON assets.
- Modify `src/app.rs`: settings migration, active/draft themes, UI integration, and terminal theme propagation.
- Modify `egui_term_local/src/theme.rs`: accept validated palette plus cursor, selection, and link colors.
- Modify `egui_term_local/src/view.rs`: render configurable cursor, selection, and hovered links.
- Modify `src/i18n.rs` and `locales/*.yaml`: theme editor and import/export messages.
- Modify `src/lib.rs`, `src/main.rs`, and `project.md`: module registration and feature status.

### Task 1: Define and validate the versioned theme schema

**Files:**
- Create: `src/theme/model.rs`
- Create: `src/theme/mod.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write failing schema tests in `src/theme/model.rs`**

Add tests first for valid color parsing, alpha colors, invalid input, unknown format versions, numeric ranges, and serialization round trips:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_color_accepts_rgb_and_rgba() {
        assert_eq!(ThemeColor::parse("#61AFEF").unwrap().to_array(), [0x61, 0xaf, 0xef, 0xff]);
        assert_eq!(ThemeColor::parse("#11223380").unwrap().to_array(), [0x11, 0x22, 0x33, 0x80]);
    }

    #[test]
    fn theme_color_rejects_invalid_values() {
        assert!(ThemeColor::parse("61afef").is_err());
        assert!(ThemeColor::parse("#xyzxyz").is_err());
    }

    #[test]
    fn validation_rejects_unknown_version_and_out_of_range_sizes() {
        let mut theme = ThemeDefinition::opennex_dark_for_test();
        theme.format_version = 2;
        assert!(matches!(theme.validate(), Err(ThemeError::UnsupportedVersion(2))));
        theme.format_version = 1;
        theme.typography.terminal_font_size = 100.0;
        assert!(matches!(theme.validate(), Err(ThemeError::InvalidField { .. })));
    }

    #[test]
    fn theme_round_trips_without_data_loss() {
        let theme = ThemeDefinition::opennex_dark_for_test();
        let json = serde_json::to_string_pretty(&theme).unwrap();
        assert_eq!(serde_json::from_str::<ThemeDefinition>(&json).unwrap(), theme);
    }
}
```

- [ ] **Step 2: Run the schema tests and verify RED**

Run: `cargo test --lib theme::model::tests -- --nocapture`

Expected: compilation fails because `ThemeColor`, `ThemeDefinition`, and `ThemeError` are not defined.

- [ ] **Step 3: Implement the minimal serializable model**

Define these public types with `Debug`, `Clone`, `PartialEq`, `Serialize`, and `Deserialize`:

```rust
pub const THEME_FORMAT_VERSION: u32 = 1;

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
}

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

pub struct TypographyTheme {
    pub terminal_font_families: Vec<String>,
    pub terminal_font_size: f32,
    pub cell_spacing: f32,
    pub menu_font_size: f32,
}
```

Implement `ThemeColor` as a validated string newtype with custom serde deserialization, `to_egui()`, and canonical lowercase output. Implement `ThemeDefinition::validate()` with these exact rules: version equals `1`; `id` contains only ASCII lowercase letters, digits, `-`, or `_`; name is non-empty and at most 80 Unicode scalar values; author is at most 80; font list is non-empty with non-empty entries; font sizes are `8.0..=32.0`; spacing is `0.5..=2.0`.

- [ ] **Step 4: Register the module and verify GREEN**

Move the current contents of `src/theme.rs` into `src/theme/mod.rs` unchanged, then add `pub mod model;` and re-export the model's public types. Do not declare `store` or `ui` until their files exist. Keep `ThemeMode`, `palette`, and the old `apply_egui_theme(ctx, ThemeMode)` API compiling until Task 5 migrates all callers.

Run: `cargo test --lib theme::model::tests -- --nocapture`

Expected: all model tests pass.

- [ ] **Step 5: Commit the schema**

```bash
git add src/theme src/lib.rs src/main.rs
git commit -m "feat: define versioned visual theme schema"
```

### Task 2: Add and validate embedded cross-platform themes

**Files:**
- Create: `assets/themes/opennex-dark.json`
- Create: `assets/themes/opennex-light.json`
- Create: `assets/themes/solarized-dark.json`
- Create: `assets/themes/gruvbox-dark.json`
- Create: `assets/themes/dracula.json`
- Create: `src/theme/store.rs`
- Modify: `src/theme/mod.rs`

- [ ] **Step 1: Write failing embedded registry tests**

```rust
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
}
```

- [ ] **Step 2: Run the registry tests and verify RED**

Run: `cargo test --lib theme::store::tests -- --nocapture`

Expected: compilation fails because the registry and theme assets do not exist.

- [ ] **Step 3: Create five complete JSON theme assets**

Use the schema from Task 1. Migrate exact ANSI values from the existing `src/terminal_theme.rs`. Use the existing dark and light `ThemePalette` values from `src/theme.rs` for `opennex-dark` and `opennex-light`; derive application surfaces for the other dark themes while retaining readable contrast. Every file must define every field, use `format_version: 1`, and use IDs matching filenames.

The typography block in every built-in theme is:

```json
"typography": {
  "terminal_font_families": ["monospace"],
  "terminal_font_size": 14.0,
  "cell_spacing": 1.0,
  "menu_font_size": 14.0
}
```

- [ ] **Step 4: Implement the embedded registry**

```rust
const EMBEDDED_THEME_JSON: [&str; 5] = [
    include_str!("../../assets/themes/opennex-dark.json"),
    include_str!("../../assets/themes/opennex-light.json"),
    include_str!("../../assets/themes/solarized-dark.json"),
    include_str!("../../assets/themes/gruvbox-dark.json"),
    include_str!("../../assets/themes/dracula.json"),
];

pub fn embedded_themes() -> Result<Vec<ThemeDefinition>, ThemeError> {
    EMBEDDED_THEME_JSON.iter().map(|json| parse_theme(json)).collect()
}

pub fn default_theme() -> Result<ThemeDefinition, ThemeError> {
    embedded_themes()?
        .into_iter()
        .find(|theme| theme.id == "opennex-dark")
        .ok_or(ThemeError::MissingDefault)
}
```

`parse_theme` must deserialize then call `validate`; no unvalidated theme may leave the store API.

- [ ] **Step 5: Verify assets and registry**

Run: `cargo test --lib theme::store::tests -- --nocapture`

Expected: both registry tests pass.

- [ ] **Step 6: Commit embedded themes**

```bash
git add assets/themes src/theme
git commit -m "feat: embed complete OpenNex themes"
```

### Task 3: Implement user theme storage, safe import, and export

**Files:**
- Modify: `src/theme/store.rs`

- [ ] **Step 1: Write failing storage tests using unique temporary directories**

Add a small test helper that creates a path under `std::env::temp_dir()` using `uuid::Uuid::new_v4()` and deletes it at test end. Cover discovery, malformed files, collision renaming, and round-trip export:

```rust
#[test]
fn import_collision_creates_a_new_id_without_overwriting() {
    let dir = temp_theme_dir();
    let theme = default_theme().unwrap();
    save_user_theme(&dir, &theme).unwrap();
    let imported = import_theme_json(&dir, &serde_json::to_string(&theme).unwrap()).unwrap();
    assert_eq!(imported.id, "opennex-dark-2");
    assert!(dir.join("opennex-dark.json").exists());
    assert!(dir.join("opennex-dark-2.json").exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_import_leaves_existing_files_unchanged() {
    let dir = temp_theme_dir();
    std::fs::create_dir_all(&dir).unwrap();
    assert!(import_theme_json(&dir, "{not-json}").is_err());
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    std::fs::remove_dir_all(dir).unwrap();
}
```

- [ ] **Step 2: Run storage tests and verify RED**

Run: `cargo test --lib theme::store::tests -- --nocapture`

Expected: compilation fails for missing storage functions.

- [ ] **Step 3: Implement storage APIs**

Implement these exact public functions:

```rust
pub fn themes_dir(app_data_dir: &Path) -> PathBuf;
pub fn load_user_themes(dir: &Path) -> Result<Vec<ThemeDefinition>, ThemeError>;
pub fn load_theme(dir: &Path, id: &str) -> Result<ThemeDefinition, ThemeError>;
pub fn save_user_theme(dir: &Path, theme: &ThemeDefinition) -> Result<(), ThemeError>;
pub fn import_theme_file(dir: &Path, source: &Path) -> Result<ThemeDefinition, ThemeError>;
pub fn import_theme_json(dir: &Path, json: &str) -> Result<ThemeDefinition, ThemeError>;
pub fn export_theme_file(theme: &ThemeDefinition, destination: &Path) -> Result<(), ThemeError>;
```

`load_theme` searches user themes first, then embedded themes, then returns `default_theme()` while logging the missing ID. `save_user_theme` serializes to `<id>.json.tmp`, calls `sync_all`, moves an existing target to `<id>.json.bak`, renames the temporary file into place, removes the backup after success, and restores the backup after failure. Import validates before creating files and uses `-2`, `-3`, etc. for collisions. Discovery ignores non-JSON files, reports malformed JSON through logs, and returns the remaining valid themes sorted by lowercase display name.

- [ ] **Step 4: Run storage tests and verify GREEN**

Run: `cargo test --lib theme::store::tests -- --nocapture`

Expected: all theme storage tests pass and no `.tmp` files remain.

- [ ] **Step 5: Commit theme persistence**

```bash
git add src/theme/store.rs
git commit -m "feat: persist and transfer user themes safely"
```

### Task 4: Extend egui_term rendering styles

**Files:**
- Modify: `egui_term_local/src/theme.rs`
- Modify: `egui_term_local/src/view.rs`

- [ ] **Step 1: Write failing terminal style tests**

In `egui_term_local/src/theme.rs`, test that the extra visual colors survive construction and that selection colors do not depend on cell foreground/background:

```rust
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
```

- [ ] **Step 2: Run local dependency tests and verify RED**

Run: `cargo test --manifest-path egui_term_local/Cargo.toml theme::tests -- --nocapture`

Expected: compilation fails because `TerminalVisualColors` and accessors are absent.

- [ ] **Step 3: Add `TerminalVisualColors` and preserve default behavior**

Add the struct, a `Default` implementation derived from the current palette behavior, accessors, and the new constructor. Retain a compatibility constructor named `from_palette(Box<ColorPalette>)` only for internal callers still being migrated in this branch; remove it in Task 5 after all callers use the full constructor.

- [ ] **Step 4: Update renderer semantics**

In `TerminalView::show`:

- For selected cells, set `bg = selection_bg` and `fg = selection_text`; do not swap cell colors.
- Draw hovered hyperlink underline with `link_color`.
- Draw the focused blinking cursor with `cursor_color`.
- Preserve ANSI inverse behavior by swapping only when `is_inverse`.
- Keep application cursor mode behavior unchanged for glyph inversion.

Extract this pure helper for testability:

```rust
fn resolved_cell_colors(
    theme: &TerminalTheme,
    mut fg: Color32,
    mut bg: Color32,
    inverse: bool,
    selected: bool,
) -> (Color32, Color32) {
    if inverse {
        std::mem::swap(&mut fg, &mut bg);
    }
    if selected {
        (theme.selection_text_color(), theme.selection_bg_color())
    } else {
        (fg, bg)
    }
}
```

- [ ] **Step 5: Verify terminal rendering tests**

Run: `cargo test --manifest-path egui_term_local/Cargo.toml --lib`

Expected: all local egui_term tests pass.

- [ ] **Step 6: Commit renderer support**

```bash
git add egui_term_local/src/theme.rs egui_term_local/src/view.rs
git commit -m "feat: theme terminal cursor selection and links"
```

### Task 5: Migrate application settings and runtime theme application

**Files:**
- Modify: `src/theme/mod.rs`
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Delete: `src/terminal_theme.rs`

- [ ] **Step 1: Write failing settings migration tests in `src/app.rs`**

Preserve a private legacy deserialization path and test old settings explicitly:

```rust
#[test]
fn legacy_visual_settings_migrate_without_touching_behavior_settings() {
    let json = r#"{
        "max_history": 777,
        "language": "de",
        "theme": "dark",
        "terminal_theme": "gruvbox",
        "bg_color": [1, 2, 3],
        "fg_color": [4, 5, 6]
    }"#;
    let settings = deserialize_settings(json).unwrap();
    assert_eq!(settings.max_history, 777);
    assert_eq!(settings.language, "de");
    assert_eq!(settings.theme_id, "gruvbox-dark");
}

#[test]
fn default_settings_select_embedded_default_theme() {
    assert_eq!(AppSettings::default().theme_id, "opennex-dark");
    assert!(AppSettings::default().apply_theme_typography);
}
```

- [ ] **Step 2: Run migration tests and verify RED**

Run: `cargo test --lib app::tests::legacy_visual_settings_migrate_without_touching_behavior_settings -- --nocapture`

Expected: compilation fails because the new settings fields and deserializer are absent.

- [ ] **Step 3: Replace visual fields in `AppSettings`**

Remove `theme`, `terminal_theme`, `bg_color`, `fg_color`, `menu_bg_color`, `menu_fg_color`, `lock_color`, `font_family`, `font_size`, `cell_spacing`, and `menu_font_size` after migration is in place. Add:

```rust
#[serde(default = "default_theme_id")]
theme_id: String,
#[serde(default = "default_true")]
apply_theme_typography: bool,
```

Implement `deserialize_settings` by first deserializing a serde-compatible migration struct with optional legacy fields. Map `light` to `opennex-light`, `one-dark` to `opennex-dark`, `solarized` to `solarized-dark`, `gruvbox` to `gruvbox-dark`, and `dracula` to `dracula`. Preserve all behavior fields. Do not write a synthetic theme from legacy RGB overrides; document this one-time limitation in `project.md` because the existing settings only cover a small subset of the complete schema.

- [ ] **Step 4: Add active and draft theme state**

Add to `App`:

```rust
active_theme: crate::theme::ThemeDefinition,
theme_edit: crate::theme::ThemeDefinition,
available_themes: Vec<crate::theme::ThemeDefinition>,
theme_message: Option<Result<String, String>>,
```

At startup, ensure the data directory exists before loading user themes. Load `settings.theme_id`, falling back to `opennex-dark`; populate embedded plus user themes with user IDs taking precedence. Apply application visuals and theme typography before constructing terminal UI.

- [ ] **Step 5: Convert theme data to egui and egui_term**

Implement:

```rust
pub fn apply_egui_theme(ctx: &egui::Context, theme: &ThemeDefinition);
pub fn terminal_theme(theme: &ThemeDefinition) -> egui_term::TerminalTheme;
```

`apply_egui_theme` maps every `AppTheme` field to the existing egui visuals. `terminal_theme` builds `ColorPalette` and `TerminalVisualColors` from `TerminalThemeConfig`. Change `TerminalTabViewer` to borrow or clone one validated `TerminalTheme` derived from `active_theme`; remove string-based `get_theme`. Remove the temporary compatibility constructor from Task 4.

- [ ] **Step 6: Resolve font candidates cross-platform**

Add a pure helper and tests:

```rust
fn resolve_font_family(candidates: &[String], installed: &[String]) -> String {
    candidates
        .iter()
        .find(|candidate| candidate.as_str() == "monospace" || installed.iter().any(|name| name == *candidate))
        .cloned()
        .unwrap_or_else(|| "monospace".to_string())
}
```

Apply typography only when `apply_theme_typography` is true. Use existing embedded multilingual fonts as final fallback and never persist an absolute font path in a theme.

- [ ] **Step 7: Run migration and application tests**

Run: `cargo test --lib app::tests -- --nocapture`

Run: `cargo test --lib theme:: -- --nocapture`

Expected: settings migration, font fallback, model, and store tests pass.

- [ ] **Step 8: Commit runtime migration**

```bash
git add src/app.rs src/theme src/lib.rs src/main.rs src/terminal_theme.rs
git commit -m "refactor: unify app and terminal theme runtime"
```

### Task 6: Build the theme editor with live preview

**Files:**
- Create: `src/theme/ui.rs`
- Modify: `src/theme/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Write failing pure UI-state tests**

Keep filesystem and state transitions outside egui closures. Test built-in edit cloning, cancel restoration, and optional typography application:

```rust
#[test]
fn editing_builtin_theme_creates_user_copy_metadata() {
    let source = store::default_theme().unwrap();
    let draft = editable_copy(&source, &[source.clone()]);
    assert_ne!(draft.id, source.id);
    assert_eq!(draft.name, "OpenNex Dark Copy");
}

#[test]
fn cancel_preview_restores_saved_theme() {
    let saved = store::default_theme().unwrap();
    let mut state = ThemeEditorState::new(saved.clone());
    state.draft.app.accent = ThemeColor::parse("#ff0000").unwrap();
    state.cancel();
    assert_eq!(state.draft, saved);
}
```

- [ ] **Step 2: Run UI-state tests and verify RED**

Run: `cargo test --lib theme::ui::tests -- --nocapture`

Expected: compilation fails for missing editor state.

- [ ] **Step 3: Implement focused editor helpers**

Expose `ThemeEditorState`, `ThemeEditorAction`, `editable_copy`, `show_theme_selector`, `show_app_color_editor`, `show_terminal_color_editor`, `show_typography_editor`, and `show_terminal_preview`. Keep file dialogs in `app.rs`; UI helpers return actions and never access the filesystem.

The preview must render a fixed ANSI sample containing normal/bright swatches, prompt text, a success line, a warning line, an error line, selected text, cursor, and link. Use fixed dimensions so color changes do not shift layout.

- [ ] **Step 4: Integrate live preview and apply/cancel behavior**

Replace the current appearance controls and terminal-theme combo. Selecting or editing a draft calls `apply_egui_theme(ctx, &theme_edit)` immediately and uses `theme_edit` for terminal rendering while the settings window is open. “关闭” restores `active_theme`; “应用” validates, saves custom themes when needed, updates `theme_id`, persists settings, updates terminal font sizes, and copies draft to active.

Built-in themes show read-only metadata and an “创建副本” action before fields become editable. Do not allow overwriting embedded IDs.

- [ ] **Step 5: Verify editor behavior tests**

Run: `cargo test --lib theme::ui::tests -- --nocapture`

Run: `cargo test --lib app::tests -- --nocapture`

Expected: editor-state and application tests pass.

- [ ] **Step 6: Commit the editor**

```bash
git add src/theme/ui.rs src/theme/mod.rs src/app.rs
git commit -m "feat: add live visual theme editor"
```

### Task 7: Add native import/export and localized feedback

**Files:**
- Modify: `src/app.rs`
- Modify: `src/i18n.rs`
- Modify: `locales/zh.yaml`
- Modify: `locales/zh-TW.yaml`
- Modify: `locales/en.yaml`
- Modify: `locales/de.yaml`
- Modify: `locales/fr.yaml`
- Modify: `locales/ja.yaml`
- Modify: `locales/it.yaml`
- Modify: `locales/ko.yaml`
- Modify: `locales/hi.yaml`

- [ ] **Step 1: Write failing i18n completeness assertions**

Extend `SettingsAppearanceTexts` with explicit fields for theme selection, author, create copy, import, export, reset, apply typography, import success, export success, invalid theme, unsupported version, and save failure. In the existing locale tests, assert each of the nine embedded YAML files deserializes and these fields are non-empty.

- [ ] **Step 2: Run locale tests and verify RED**

Run: `cargo test --lib i18n::tests -- --nocapture`

Expected: locale deserialization fails because the new keys are missing.

- [ ] **Step 3: Add all nine translations**

Use concise native labels. Keep technical extension text `.opennex-theme.json` untranslated. Ensure `zh_default()` includes the same fields so malformed external locale data still has complete fallback text.

- [ ] **Step 4: Wire native dialogs and transactional actions**

Import dialog:

```rust
rfd::FileDialog::new()
    .add_filter("OpenNex Theme", &["json"])
    .pick_file()
```

Export dialog:

```rust
rfd::FileDialog::new()
    .add_filter("OpenNex Theme", &["json"])
    .set_file_name(format!("{}.opennex-theme.json", self.theme_edit.id))
    .save_file()
```

Import calls `store::import_theme_file`, refreshes available themes, selects the imported theme, and previews it only after success. Export validates the draft then calls `store::export_theme_file`. Surface success/error in the settings window; log technical causes. Dialog cancellation is a no-op.

- [ ] **Step 5: Run i18n and store regression tests**

Run: `cargo test --lib i18n::tests -- --nocapture`

Run: `cargo test --lib theme::store::tests -- --nocapture`

Expected: all locale and transfer tests pass.

- [ ] **Step 6: Commit import/export UI**

```bash
git add src/app.rs src/i18n.rs locales
git commit -m "feat: import and export visual themes"
```

### Task 8: Update project status and perform full verification

**Files:**
- Modify: `project.md`
- Modify: `README.md`

- [ ] **Step 1: Update project documentation**

Add the completed feature to `project.md`: versioned visual themes, five embedded themes, user theme directory, cross-platform import/export, font fallback, and the explicit exclusion of shell semantic highlighting. Update the project structure with `assets/themes/` and `src/theme/`.

Add a concise README section documenting:

- Themes work on Windows, macOS, and Linux.
- Exported files contain visual settings only.
- Missing fonts fall back automatically.
- Shell prompt and command/file semantic colors remain controlled by the shell and CLI programs.

- [ ] **Step 2: Run formatting and static checks**

Run: `cargo fmt --all -- --check`

Expected: exits successfully with no diff.

Run: `cargo clippy --all-targets --all-features -- -D warnings`

Expected: exits successfully with no warnings. Fix only warnings introduced or exposed by this work; record unrelated pre-existing warnings instead of broad refactors.

- [ ] **Step 3: Run all tests**

Run: `cargo test --all --all-features`

Expected: all root and local dependency tests pass.

- [ ] **Step 4: Verify debug and release builds**

Run: `cargo build`

Expected: debug build succeeds.

Run: `cargo build --release`

Expected: release build succeeds and embedded themes are present without runtime asset files.

- [ ] **Step 5: Perform manual cross-platform-focused checks on the available host**

Run: `cargo run`

Verify: switch each embedded theme; edit a copied theme; preview and cancel; apply and restart; import an exported file; import malformed JSON without state change; choose a missing font candidate and observe fallback; inspect ANSI normal/bright colors, selection, cursor, and links in a terminal. Confirm no `.bashrc`, `.zshrc`, PowerShell Profile, or `LS_COLORS` changes occur.

- [ ] **Step 6: Commit documentation and final fixes**

```bash
git add README.md project.md src assets egui_term_local locales
git commit -m "docs: document cross-platform visual themes"
```

## Release Gate

Do not tag a release from this plan alone. Before release, run CI on Windows, macOS, and Linux and manually verify one import/export round trip per platform. The same exported fixture must parse on all three jobs. Version bump and tag selection remain a separate release decision.

use crate::theme::model::{AnsiColors, ThemeColor, ThemeDefinition};
use crate::theme::store;
use egui::Color32;

/// Actions returned by the theme editor UI, handled by `app.rs`.
#[derive(Debug, Clone)]
pub enum ThemeAction {
    /// User selected a different theme from the dropdown.
    SelectTheme(String),
    /// User clicked a subtab.
    SelectSubtab(ThemeEditorSubtab),
    /// User wants to create a new theme based on the current one.
    NewTheme,
    /// User wants to duplicate the current theme.
    CopyTheme,
    /// User wants to rename the current user theme.
    RenameTheme(String),
    /// User wants to delete the current user theme.
    DeleteTheme,
    /// User wants to import a theme file.
    ImportTheme,
    /// User wants to export the current theme.
    ExportTheme,
    /// User applied a terminal palette template (only terminal colors change).
    /// Draft was modified — triggers live preview.
    DraftModified,
}

/// State for the popup dialogs used by the theme editor.
#[derive(Debug, Clone, Default)]
pub struct ThemeDialogState {
    pub show_copy_dialog: bool,
    pub show_new_dialog: bool,
    pub show_rename_dialog: bool,
    pub show_delete_confirm: bool,
    pub show_switch_confirm: bool,
    pub name_input: String,
    pub pending_switch_target: String,
    /// Tracks whether we already requested focus for the current dialog.
    /// Reset to false when all dialogs are closed.
    pub focus_requested: bool,
}

/// Subtab inside the theme editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeEditorSubtab {
    #[default]
    UiAppearance,
    Terminal,
    AnsiPalette,
}

/// Render only the theme editor BODY for one subtab — no theme selector,
/// no management buttons, no subtab tab row. Used by the unified theme
/// settings page which draws its own section headings.
/// Localized color names for the theme editor.
#[derive(Clone, Default)]
pub struct ColorLabels {
    pub app_bg: String,
    pub sidebar: String,
    pub panel: String,
    pub input_bg: String,
    pub text: String,
    pub weak_text: String,
    pub accent: String,
    pub warning: String,
    pub danger: String,
    pub hover: String,
    pub active: String,
    pub selection_bg: String,
    pub selection_text: String,
    pub border: String,
    pub lock: String,
    pub window_shadow: String,
    pub tab_highlight: String,
    pub fg: String,
    pub bg: String,
    pub cursor: String,
    pub selection_term_bg: String,
    pub selection_term_text: String,
    pub link: String,
}

/// Localized labels for the theme editor sections.
#[derive(Clone, Default)]
pub struct ThemeEditorLabels {
    pub system_ui: String,
    pub terminal: String,
    pub ui_font: String,
    pub ui_font_size: String,
    pub terminal_font: String,
    pub terminal_font_size: String,
    pub cell_spacing: String,
    pub terminal_padding: String,
    pub colors: ColorLabels,
}

pub fn show_theme_editor_body(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available: &[ThemeDefinition],
    available_fonts: &[String],
    is_builtin: bool,
    has_unsaved: bool,
    subtab: ThemeEditorSubtab,
    dialog: &mut ThemeDialogState,
    labels: &ThemeEditorLabels,
) -> Vec<ThemeAction> {
    let mut actions = Vec::new();
    let _ = (available, has_unsaved, dialog);
    match subtab {
        ThemeEditorSubtab::UiAppearance => {
            show_ui_appearance_editor(ui, draft, available_fonts, is_builtin, &mut actions, labels);
        }
        ThemeEditorSubtab::Terminal => {
            show_terminal_section_editor(
                ui,
                draft,
                available_fonts,
                is_builtin,
                &mut actions,
                labels,
            );
        }
        ThemeEditorSubtab::AnsiPalette => {
            show_ansi_palette_editor(ui, draft, is_builtin, &mut actions);
        }
    }
    actions
}

/// Render the full theme management section.
#[allow(dead_code)]
pub fn show_theme_section(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available: &[ThemeDefinition],
    available_fonts: &[String],
    is_builtin: bool,
    has_unsaved: bool,
    subtab: ThemeEditorSubtab,
    dialog: &mut ThemeDialogState,
) -> Vec<ThemeAction> {
    let mut actions = Vec::new();
    let labels = ThemeEditorLabels {
        system_ui: "System UI".into(),
        terminal: "Terminal".into(),
        ui_font: "UI 字体: ".into(),
        ui_font_size: "UI 字号: ".into(),
        terminal_font: "终端字体: ".into(),
        terminal_font_size: "终端字号: ".into(),
        cell_spacing: "间距: ".into(),
        terminal_padding: "终端内边距: ".into(),
        colors: Default::default(),
    };

    actions.extend(show_theme_selector(ui, draft, available, has_unsaved));
    actions.extend(show_theme_buttons(ui, is_builtin, dialog));
    actions.extend(show_import_export(ui));

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                matches!(subtab, ThemeEditorSubtab::UiAppearance),
                crate::theme::ui_appearance_text(),
            )
            .clicked()
        {
            actions.push(ThemeAction::SelectSubtab(ThemeEditorSubtab::UiAppearance));
        }
        if ui
            .selectable_label(
                matches!(subtab, ThemeEditorSubtab::Terminal),
                crate::theme::terminal_appearance_text(),
            )
            .clicked()
        {
            actions.push(ThemeAction::SelectSubtab(ThemeEditorSubtab::Terminal));
        }
        if ui
            .selectable_label(
                matches!(subtab, ThemeEditorSubtab::AnsiPalette),
                crate::theme::ansi_palette_text(),
            )
            .clicked()
        {
            actions.push(ThemeAction::SelectSubtab(ThemeEditorSubtab::AnsiPalette));
        }
    });
    ui.add_space(4.0);

    match subtab {
        ThemeEditorSubtab::UiAppearance => {
            show_ui_appearance_editor(
                ui,
                draft,
                available_fonts,
                is_builtin,
                &mut actions,
                &labels,
            );
        }
        ThemeEditorSubtab::Terminal => {
            show_terminal_section_editor(
                ui,
                draft,
                available_fonts,
                is_builtin,
                &mut actions,
                &labels,
            );
        }
        ThemeEditorSubtab::AnsiPalette => {
            show_ansi_palette_editor(ui, draft, is_builtin, &mut actions);
        }
    }

    actions
}

fn show_theme_selector(
    ui: &mut egui::Ui,
    draft: &ThemeDefinition,
    available: &[ThemeDefinition],
    has_unsaved: bool,
) -> Vec<ThemeAction> {
    let mut actions = Vec::new();
    let current_id = draft.id.clone();

    ui.horizontal(|ui| {
        ui.label(crate::theme::theme_label_text());
        egui::ComboBox::from_id_salt("theme_dropdown")
            .selected_text(
                available
                    .iter()
                    .find(|t| t.id == current_id)
                    .map(|t| t.name.as_str())
                    .unwrap_or("—"),
            )
            .show_ui(ui, |ui| {
                for theme in available {
                    let suffix = if store::is_embedded_id(&theme.id) {
                        ""
                    } else {
                        ""
                    };
                    let label = format!("{}{}", theme.name, suffix);
                    if ui
                        .selectable_label(theme.id == current_id, &label)
                        .clicked()
                    {
                        actions.push(ThemeAction::SelectTheme(theme.id.clone()));
                    }
                }
            });
        if has_unsaved {
            ui.colored_label(
                Color32::from_rgb(0xd9, 0xa4, 0x41),
                crate::theme::unsaved_text(),
            );
        }
    });
    let _ = has_unsaved;
    actions
}

fn show_theme_buttons(
    ui: &mut egui::Ui,
    is_builtin: bool,
    dialog: &mut ThemeDialogState,
) -> Vec<ThemeAction> {
    let mut actions = Vec::new();
    ui.horizontal(|ui| {
        if ui.button(crate::theme::new_theme_text()).clicked() {
            dialog.show_new_dialog = true;
            dialog.name_input.clear();
        }
        if ui.button(crate::theme::copy_theme_text()).clicked() {
            dialog.show_copy_dialog = true;
            dialog.name_input.clear();
        }
        if !is_builtin {
            if ui.button(crate::theme::rename_theme_text()).clicked() {
                dialog.show_rename_dialog = true;
                dialog.name_input.clear();
            }
            if ui.button(crate::theme::delete_theme_text()).clicked() {
                dialog.show_delete_confirm = true;
            }
        }
    });
    actions
}

fn show_import_export(ui: &mut egui::Ui) -> Vec<ThemeAction> {
    let mut actions = Vec::new();
    ui.horizontal(|ui| {
        if ui.button(crate::theme::import_theme_text()).clicked() {
            actions.push(ThemeAction::ImportTheme);
        }
        if ui.button(crate::theme::export_theme_text()).clicked() {
            actions.push(ThemeAction::ExportTheme);
        }
    });
    actions
}

fn show_ui_appearance_editor(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available_fonts: &[String],
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
    labels: &ThemeEditorLabels,
) {
    ui.strong(egui::RichText::new(&labels.system_ui).size(12.0));
    ui.add_space(4.0);
    // Row 1: font + size (inline)
    ui.horizontal(|ui| {
        let current = draft
            .app
            .ui_font_families
            .first()
            .cloned()
            .unwrap_or_default();
        egui::ComboBox::from_id_salt("ui_font_select")
            .selected_text(if current.is_empty() {
                "system-ui"
            } else {
                &current
            })
            .width(140.0)
            .show_ui(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for font in available_fonts {
                            if ui.selectable_label(current == *font, font).clicked() {
                                draft.app.ui_font_families = vec![font.clone()];
                                actions.push(ThemeAction::DraftModified);
                            }
                        }
                    });
            });
        ui.add(
            egui::DragValue::new(&mut draft.app.ui_font_size)
                .range(8.0..=32.0)
                .prefix(&labels.ui_font_size),
        )
        .on_hover_text(crate::theme::ui_font_size_label());
    });

    ui.add_space(6.0);

    // Color grid: 3 columns × ~6 rows of compact color cells.
    // Each row: 3 swatch+hex combos using minimal vertical space.
    let c = &labels.colors;
    let mut pairs: [(&mut ThemeColor, &str, &'static str); 17] = [
        (&mut draft.app.app_bg, c.app_bg.as_str(), "app_bg"),
        (&mut draft.app.sidebar, c.sidebar.as_str(), "sidebar"),
        (&mut draft.app.panel, c.panel.as_str(), "panel"),
        (&mut draft.app.input_bg, c.input_bg.as_str(), "input_bg"),
        (&mut draft.app.text, c.text.as_str(), "text"),
        (&mut draft.app.weak_text, c.weak_text.as_str(), "weak_text"),
        (&mut draft.app.accent, c.accent.as_str(), "accent"),
        (&mut draft.app.warning, c.warning.as_str(), "warning"),
        (&mut draft.app.danger, c.danger.as_str(), "danger"),
        (&mut draft.app.hover, c.hover.as_str(), "hover"),
        (&mut draft.app.active, c.active.as_str(), "active"),
        (
            &mut draft.app.selection_bg,
            c.selection_bg.as_str(),
            "app_sel_bg",
        ),
        (
            &mut draft.app.selection_text,
            c.selection_text.as_str(),
            "app_sel_text",
        ),
        (&mut draft.app.border, c.border.as_str(), "border"),
        (&mut draft.app.lock, c.lock.as_str(), "lock"),
        (
            &mut draft.app.window_shadow,
            c.window_shadow.as_str(),
            "shadow",
        ),
        (
            &mut draft.app.tab_highlight,
            c.tab_highlight.as_str(),
            "tab_hl",
        ),
    ];
    for (color, label, key) in pairs {
        compact_color_cell(ui, color, label, true, actions, key);
    }
}

/// Terminal subtab: base colors, palette template, font size, cell spacing.
fn show_terminal_section_editor(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available_fonts: &[String],
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
    labels: &ThemeEditorLabels,
) {
    ui.strong(egui::RichText::new(&labels.terminal).size(12.0));
    ui.add_space(4.0);
    // Row 1: terminal font + size + cell spacing (inline)
    ui.horizontal(|ui| {
        let current = draft
            .typography
            .terminal_font_families
            .first()
            .cloned()
            .unwrap_or_default();
        egui::ComboBox::from_id_salt("terminal_font_select")
            .selected_text(if current.is_empty() {
                "monospace"
            } else {
                &current
            })
            .width(140.0)
            .show_ui(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for font in available_fonts {
                            if ui.selectable_label(current == *font, font).clicked() {
                                draft.typography.terminal_font_families = vec![font.clone()];
                                actions.push(ThemeAction::DraftModified);
                            }
                        }
                    });
            });
        ui.add(
            egui::DragValue::new(&mut draft.typography.terminal_font_size)
                .range(8.0..=32.0)
                .prefix(labels.terminal_font_size.clone()),
        );
        ui.add_space(8.0);
        ui.add(
            egui::DragValue::new(&mut draft.typography.cell_spacing)
                .range(0.5..=2.0)
                .prefix(labels.cell_spacing.clone()),
        );
        ui.add_space(8.0);
        ui.add(
            egui::DragValue::new(&mut draft.typography.terminal_padding)
                .range(0.0..=32.0)
                .prefix(labels.terminal_padding.clone()),
        );
    });

    ui.add_space(6.0);

    // Base colors in 3-column grid.
    let c = &labels.colors;
    let mut pairs: [(&mut ThemeColor, &str, &'static str); 6] = [
        (&mut draft.terminal.foreground, c.fg.as_str(), "term_fg"),
        (&mut draft.terminal.background, c.bg.as_str(), "term_bg"),
        (&mut draft.terminal.cursor, c.cursor.as_str(), "term_cursor"),
        (
            &mut draft.terminal.selection_bg,
            c.selection_term_bg.as_str(),
            "term_sel_bg",
        ),
        (
            &mut draft.terminal.selection_text,
            c.selection_term_text.as_str(),
            "term_sel_text",
        ),
        (&mut draft.terminal.link, c.link.as_str(), "term_link"),
    ];
    for (color, label, key) in pairs {
        compact_color_cell(ui, color, label, true, actions, key);
    }
}

/// ANSI palette subtab: three rows of 8 swatches each (normal/bright/dim).
fn show_ansi_palette_editor(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    let mut groups: [(&str, &mut AnsiColors); 3] = [
        ("普通", &mut draft.terminal.normal),
        ("明亮", &mut draft.terminal.bright),
        ("暗淡", &mut draft.terminal.dim),
    ];
    for (label, colors) in groups.iter_mut() {
        ui.horizontal(|ui| {
            ui.weak(egui::RichText::new(*label).small());
        });
        ui.horizontal(|ui| {
            let black = &mut colors.black;
            let red = &mut colors.red;
            let green = &mut colors.green;
            let yellow = &mut colors.yellow;
            let blue = &mut colors.blue;
            let magenta = &mut colors.magenta;
            let cyan = &mut colors.cyan;
            let white = &mut colors.white;
            let mut slots: [(&mut ThemeColor, &str); 8] = [
                (black, "黑"),
                (red, "红"),
                (green, "绿"),
                (yellow, "黄"),
                (blue, "蓝"),
                (magenta, "紫"),
                (cyan, "青"),
                (white, "白"),
            ];
            for (color, name) in slots.iter_mut() {
                let _ = compact_color_cell(ui, color, name, true, actions, "ansi");
            }
        });
        ui.add_space(4.0);
    }
    let _ = color_dragvalue_row; // suppress unused warning
}

fn ansi_palette_grid(
    ui: &mut egui::Ui,
    colors: &mut AnsiColors,
    label: String,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    ui.label(&label);
    ui.horizontal(|ui| {
        let slots: [(&mut ThemeColor, &str); 8] = [
            (&mut colors.black, crate::theme::black_label()),
            (&mut colors.red, crate::theme::red_label()),
            (&mut colors.green, crate::theme::green_label()),
            (&mut colors.yellow, crate::theme::yellow_label()),
            (&mut colors.blue, crate::theme::blue_label()),
            (&mut colors.magenta, crate::theme::magenta_label()),
            (&mut colors.cyan, crate::theme::cyan_label()),
            (&mut colors.white, crate::theme::white_label()),
        ];
        for (color, _) in slots {
            let _ = label;
            color_swatch(ui, color, is_builtin, actions);
        }
    });
    ui.add_space(2.0);
}

fn color_swatch(
    ui: &mut egui::Ui,
    color: &mut ThemeColor,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    let mut srgb = [
        color.to_array()[0],
        color.to_array()[1],
        color.to_array()[2],
    ];
    let prev = srgb;
    ui.add_enabled_ui(!is_builtin, |ui| {
        egui::widgets::color_picker::color_edit_button_srgb(ui, &mut srgb);
    });
    if srgb != prev {
        *color = ThemeColor::from_rgb_opaque(srgb[0], srgb[1], srgb[2]);
        actions.push(ThemeAction::DraftModified);
    }
}

/// Compact color cell as a two-column table row: the label column and the
/// value column each left-align within their column, so every row's label
/// and swatch line up. The hex code is rendered INSIDE the swatch,
/// horizontally and vertically centered.
fn compact_color_cell(
    ui: &mut egui::Ui,
    color: &mut ThemeColor,
    label: &str,
    enabled: bool,
    actions: &mut Vec<ThemeAction>,
    // Stable widget-id key: must be unique per row and NOT derived from the
    // (translatable) label — different rows can share a translation in some
    // languages ("Selection text"), which would clash egui widget ids.
    key: &'static str,
) {
    let mut srgb = [
        color.to_array()[0],
        color.to_array()[1],
        color.to_array()[2],
    ];
    let prev = srgb;
    let row_h = 22.0;
    let (rect, _) = ui.allocate_exact_size(
        // Table width: half of the available width (50% reduction).
        egui::vec2(ui.available_width() * 0.5, row_h),
        egui::Sense::hover(),
    );
    // --- Column 1: label, left-aligned within a fixed 40% column. ---
    let label_col_w = rect.width() * 0.4;
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(12.0),
            ui.visuals().text_color(),
        )
    });
    ui.painter().galley(
        egui::pos2(rect.min.x, rect.center().y - galley.size().y / 2.0),
        galley,
        ui.visuals().text_color(),
    );

    // --- Column 2: swatch with the hex code inside (centered). ---
    let swatch_w = 110.0;
    let swatch = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + label_col_w, rect.center().y - row_h / 2.0),
        egui::vec2(swatch_w, row_h),
    );
    let sw_bg = egui::Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
    let resp = if enabled {
        ui.interact(
            swatch,
            egui::Id::new(("color_swatch", key)),
            egui::Sense::click(),
        )
    } else {
        ui.interact(
            swatch,
            egui::Id::new(("color_swatch", key)),
            egui::Sense::hover(),
        )
    };
    ui.painter().rect_filled(swatch, 2.0, sw_bg);
    ui.painter().rect_stroke(
        swatch,
        2.0,
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color),
        egui::StrokeKind::Inside,
    );
    // Hex text inside the swatch: pick black or white for contrast.
    let lum = 0.299 * srgb[0] as f32 + 0.587 * srgb[1] as f32 + 0.114 * srgb[2] as f32;
    let hex_color = if lum > 128.0 {
        egui::Color32::BLACK
    } else {
        egui::Color32::WHITE
    };
    let hex_galley =
        ui.fonts(|f| f.layout_no_wrap(color.as_hex(), egui::FontId::monospace(10.0), hex_color));
    ui.painter().galley(
        swatch.center() - hex_galley.size() / 2.0,
        hex_galley,
        hex_color,
    );

    // Clicking the swatch toggles a color picker popup anchored below it.
    let popup_id = egui::Id::new(("color_popup", key));
    if enabled {
        egui::popup_below_widget(
            ui,
            popup_id,
            &resp,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                let mut c32 = egui::Color32::from_rgb(srgb[0], srgb[1], srgb[2]);
                if egui::widgets::color_picker::color_picker_color32(
                    ui,
                    &mut c32,
                    egui::color_picker::Alpha::Opaque,
                ) {
                    srgb = [c32.r(), c32.g(), c32.b()];
                }
            },
        );
    }
    if srgb != prev {
        *color = ThemeColor::from_rgb_opaque(srgb[0], srgb[1], srgb[2]);
        actions.push(ThemeAction::DraftModified);
    }
}
#[allow(dead_code)]
fn color_row(
    ui: &mut egui::Ui,
    color: &mut ThemeColor,
    label: String,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    ui.horizontal(|ui| {
        ui.label(&label);
        let mut srgb = [
            color.to_array()[0],
            color.to_array()[1],
            color.to_array()[2],
        ];
        let prev = srgb;
        ui.add_enabled_ui(!is_builtin, |ui| {
            egui::widgets::color_picker::color_edit_button_srgb(ui, &mut srgb);
        });
        if srgb != prev {
            *color = ThemeColor::from_rgb_opaque(srgb[0], srgb[1], srgb[2]);
            actions.push(ThemeAction::DraftModified);
        }
    });
}

fn color_dragvalue_row(
    ui: &mut egui::Ui,
    value: &mut f32,
    label: String,
    min: f32,
    max: f32,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    ui.horizontal(|ui| {
        ui.label(&label);
        let prev = *value;
        ui.add_enabled(!is_builtin, egui::DragValue::new(value).range(min..=max));
        if *value != prev {
            actions.push(ThemeAction::DraftModified);
        }
    });
}

/// Text strings for the theme editor, centralized for i18n.
/// These are inline constants; full i18n wiring happens in Task 7.
pub mod texts {
    pub fn heading() -> String {
        "主题".into()
    }
    pub fn theme_label() -> String {
        "当前主题:".into()
    }
    pub fn unsaved() -> String {
        "● 未保存".into()
    }
    pub fn new_theme() -> String {
        "新建".into()
    }
    pub fn copy_theme() -> String {
        "复制".into()
    }
    pub fn rename_theme() -> String {
        "重命名".into()
    }
    pub fn delete_theme() -> String {
        "删除".into()
    }
    pub fn import_theme() -> String {
        "导入".into()
    }
    pub fn export_theme() -> String {
        "导出".into()
    }

    pub fn ui_appearance() -> String {
        "UI 外观".into()
    }
    pub fn ui_font() -> String {
        "UI 字体:".into()
    }
    pub fn ui_font_size() -> String {
        "UI 字号:".into()
    }
    pub fn base_colors() -> String {
        "基础颜色".into()
    }
    pub fn app_bg() -> String {
        "主背景:".into()
    }
    pub fn sidebar() -> String {
        "侧栏:".into()
    }
    pub fn panel() -> String {
        "面板:".into()
    }
    pub fn input_bg() -> String {
        "输入框:".into()
    }
    pub fn text_colors() -> String {
        "文字颜色".into()
    }
    pub fn text() -> String {
        "普通文字:".into()
    }
    pub fn weak_text() -> String {
        "弱化文字:".into()
    }
    pub fn status_colors() -> String {
        "状态颜色".into()
    }
    pub fn accent() -> String {
        "强调:".into()
    }
    pub fn warning() -> String {
        "警告:".into()
    }
    pub fn danger() -> String {
        "危险:".into()
    }
    pub fn interaction_colors() -> String {
        "交互颜色".into()
    }
    pub fn hover() -> String {
        "悬停:".into()
    }
    pub fn active() -> String {
        "激活:".into()
    }
    pub fn selection_bg() -> String {
        "选中背景:".into()
    }
    pub fn selection_text() -> String {
        "选中文字:".into()
    }
    pub fn border() -> String {
        "边框:".into()
    }
    pub fn lock() -> String {
        "锁定遮罩:".into()
    }

    pub fn terminal_appearance() -> String {
        "终端外观".into()
    }
    pub fn terminal_font_size() -> String {
        "终端字号:".into()
    }
    pub fn cell_spacing() -> String {
        "单元格间距:".into()
    }
    pub fn palette_template() -> String {
        "配色模板:".into()
    }
    pub fn apply_template() -> String {
        "应用模板".into()
    }
    pub fn terminal_base_colors() -> String {
        "基础颜色".into()
    }
    pub fn fg() -> String {
        "前景色:".into()
    }
    pub fn bg() -> String {
        "背景色:".into()
    }
    pub fn cursor() -> String {
        "光标:".into()
    }
    pub fn link() -> String {
        "链接:".into()
    }
    pub fn normal() -> String {
        "普通".into()
    }
    pub fn bright() -> String {
        "明亮".into()
    }
    pub fn dim() -> String {
        "暗淡".into()
    }
    pub fn black() -> &'static str {
        "黑"
    }
    pub fn red() -> &'static str {
        "红"
    }
    pub fn green() -> &'static str {
        "绿"
    }
    pub fn yellow() -> &'static str {
        "黄"
    }
    pub fn blue() -> &'static str {
        "蓝"
    }
    pub fn magenta() -> &'static str {
        "紫"
    }
    pub fn cyan() -> &'static str {
        "青"
    }
    pub fn white() -> &'static str {
        "白"
    }

    pub fn copy_dialog_title() -> String {
        "创建主题副本".into()
    }
    pub fn copy_dialog_hint() -> String {
        "输入新主题名称:".into()
    }
    pub fn new_dialog_title() -> String {
        "新建主题".into()
    }
    pub fn new_dialog_hint() -> String {
        "输入新主题名称:".into()
    }
    pub fn rename_dialog_title() -> String {
        "重命名主题".into()
    }
    pub fn delete_confirm() -> String {
        "确认删除此主题？".into()
    }
    pub fn switch_confirm() -> String {
        "当前主题有未保存的修改".into()
    }
    pub fn save_and_switch() -> String {
        "保存并切换".into()
    }
    pub fn discard_and_switch() -> String {
        "放弃并切换".into()
    }
    pub fn cancel() -> String {
        "取消".into()
    }
    pub fn confirm() -> String {
        "确认".into()
    }
    pub fn ok() -> String {
        "确定".into()
    }
    pub fn builtin_readonly() -> String {
        "内置主题只读，编辑将创建副本".into()
    }
    pub fn keep() -> String {
        "保留".into()
    }
    pub fn discard() -> String {
        "放弃修改".into()
    }
    pub fn edit_theme() -> String {
        "编辑主题".into()
    }
    pub fn name_label() -> String {
        "名称:".into()
    }
}

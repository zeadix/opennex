use crate::theme::model::{AnsiColors, ThemeColor, ThemeDefinition};
use crate::theme::{model, palettes, store};
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
    ApplyPaletteTemplate(String),
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

/// Render the full theme management section.
pub fn show_theme_section(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available: &[ThemeDefinition],
    is_builtin: bool,
    has_unsaved: bool,
    subtab: ThemeEditorSubtab,
    dialog: &mut ThemeDialogState,
) -> Vec<ThemeAction> {
    let mut actions = Vec::new();

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
            show_ui_appearance_editor(ui, draft, is_builtin, &mut actions);
        }
        ThemeEditorSubtab::Terminal => {
            show_terminal_section_editor(ui, draft, is_builtin, &mut actions);
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
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
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
                ui.selectable_value(
                    &mut draft.app.ui_font_families,
                    vec!["system-ui".into()],
                    "system-ui",
                );
            });
        ui.add(
            egui::DragValue::new(&mut draft.app.ui_font_size)
                .range(8.0..=32.0)
                .prefix("UI 字号: "),
        )
        .on_hover_text(crate::theme::ui_font_size_label());
    });

    ui.add_space(6.0);

    // Color grid: 3 columns × ~6 rows of compact color cells.
    // Each row: 3 swatch+hex combos using minimal vertical space.
    let mut pairs: [(&mut ThemeColor, &str); 16] = [
        (&mut draft.app.app_bg, "主背景"),
        (&mut draft.app.sidebar, "侧栏"),
        (&mut draft.app.panel, "面板"),
        (&mut draft.app.input_bg, "输入框"),
        (&mut draft.app.text, "文字"),
        (&mut draft.app.weak_text, "弱化文字"),
        (&mut draft.app.accent, "强调"),
        (&mut draft.app.warning, "警告"),
        (&mut draft.app.danger, "危险"),
        (&mut draft.app.hover, "悬停"),
        (&mut draft.app.active, "激活"),
        (&mut draft.app.selection_bg, "选中背景"),
        (&mut draft.app.selection_text, "选中文字"),
        (&mut draft.app.border, "边框"),
        (&mut draft.app.lock, "锁定"),
        (&mut draft.app.window_shadow, "阴影"),
    ];
    for chunk in pairs.chunks_mut(3) {
        ui.horizontal(|ui| {
            for (color, label) in chunk {
                let _ = compact_color_cell(ui, color, label, true, actions);
            }
        });
    }
}

/// Terminal subtab: base colors, palette template, font size, cell spacing.
fn show_terminal_section_editor(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    // Row 1: font size + cell spacing (inline)
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(&mut draft.typography.terminal_font_size)
                .range(8.0..=32.0)
                .prefix("终端字号: "),
        );
        ui.add_space(8.0);
        ui.add(
            egui::DragValue::new(&mut draft.typography.cell_spacing)
                .range(0.5..=2.0)
                .prefix("间距: "),
        );
    });

    ui.add_space(6.0);

    // Palette template
    palette_template_selector(ui, actions);

    ui.add_space(6.0);

    // Base colors in 3-column grid.
    let mut pairs: [(&mut ThemeColor, &str); 6] = [
        (&mut draft.terminal.foreground, "前景"),
        (&mut draft.terminal.background, "背景"),
        (&mut draft.terminal.cursor, "光标"),
        (&mut draft.terminal.selection_bg, "选区背景"),
        (&mut draft.terminal.selection_text, "选区文字"),
        (&mut draft.terminal.link, "链接"),
    ];
    for chunk in pairs.chunks_mut(3) {
        ui.horizontal(|ui| {
            for (color, label) in chunk {
                let _ = compact_color_cell(ui, color, label, true, actions);
            }
        });
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
                let _ = compact_color_cell(ui, color, name, true, actions);
            }
        });
        ui.add_space(4.0);
    }
    let _ = color_dragvalue_row; // suppress unused warning
}

fn palette_template_selector(ui: &mut egui::Ui, actions: &mut Vec<ThemeAction>) {
    ui.horizontal(|ui| {
        ui.label(crate::theme::palette_template_label());
        let templates = palettes::templates();
        let mut selected = 0usize;
        egui::ComboBox::from_id_salt("palette_template_select")
            .selected_text(templates[selected].name)
            .show_ui(ui, |ui| {
                for (i, template) in templates.iter().enumerate() {
                    ui.selectable_value(&mut selected, i, template.name);
                }
            });
        if ui.button(crate::theme::apply_template_text()).clicked() {
            actions.push(ThemeAction::ApplyPaletteTemplate(
                templates[selected].id.to_string(),
            ));
        }
    });
}

#[allow(dead_code)]
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

/// Compact color cell: shows label + color swatch + hex value.
fn compact_color_cell(
    ui: &mut egui::Ui,
    color: &mut ThemeColor,
    label: &str,
    enabled: bool,
    actions: &mut Vec<ThemeAction>,
) {
    let mut srgb = [
        color.to_array()[0],
        color.to_array()[1],
        color.to_array()[2],
    ];
    let prev = srgb;
    let label_width = 64.0;
    ui.add_enabled_ui(enabled, |ui| {
        ui.label(label);
        egui::widgets::color_picker::color_edit_button_srgb(ui, &mut srgb);
        ui.weak(egui::RichText::new(color.as_hex()).small());
        let _ = label_width;
    });
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
}

use crate::theme::model::{AnsiColors, ThemeColor, ThemeDefinition};
use crate::theme::{model, palettes, store};
use egui::Color32;

/// Actions returned by the theme editor UI, handled by `app.rs`.
#[derive(Debug, Clone)]
pub enum ThemeAction {
    /// User selected a different theme from the dropdown.
    SelectTheme(String),
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

/// Render the full theme management section.
///
/// Returns a list of actions for `app.rs` to process. The `draft` is the
/// currently-edited theme; `is_builtin` controls whether edit controls are
/// interactive.
pub fn show_theme_section(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available: &[ThemeDefinition],
    is_builtin: bool,
    has_unsaved: bool,
    dialog: &mut ThemeDialogState,
) -> Vec<ThemeAction> {
    let mut actions = Vec::new();

    ui.heading(crate::theme::heading_text());
    ui.add_space(4.0);

    actions.extend(show_theme_selector(ui, draft, available, has_unsaved));
    actions.extend(show_theme_buttons(ui, is_builtin, dialog));
    actions.extend(show_import_export(ui));

    ui.separator();
    ui.add_space(4.0);

    ui.label(crate::theme::ui_appearance_text());
    ui.add_space(2.0);
    show_ui_appearance_editor(ui, draft, is_builtin, &mut actions);

    ui.separator();
    ui.add_space(4.0);

    ui.label(crate::theme::terminal_appearance_text());
    ui.add_space(2.0);
    show_terminal_appearance_editor(ui, draft, is_builtin, &mut actions);

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
    ui.horizontal(|ui| {
        ui.label(crate::theme::ui_font_label());
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
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut draft.app.ui_font_families,
                    vec!["system-ui".into()],
                    "system-ui (默认)",
                );
            });
    });
    color_dragvalue_row(
        ui,
        &mut draft.app.ui_font_size,
        crate::theme::ui_font_size_label(),
        8.0,
        32.0,
        is_builtin,
        actions,
    );

    ui.add_space(4.0);
    ui.label(crate::theme::base_colors_text());
    color_row(
        ui,
        &mut draft.app.app_bg,
        crate::theme::app_bg_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.sidebar,
        crate::theme::sidebar_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.panel,
        crate::theme::panel_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.input_bg,
        crate::theme::input_bg_label(),
        is_builtin,
        actions,
    );

    ui.add_space(4.0);
    ui.label(crate::theme::text_colors_text());
    color_row(
        ui,
        &mut draft.app.text,
        crate::theme::text_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.weak_text,
        crate::theme::weak_text_label(),
        is_builtin,
        actions,
    );

    ui.add_space(4.0);
    ui.label(crate::theme::status_colors_text());
    color_row(
        ui,
        &mut draft.app.accent,
        crate::theme::accent_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.warning,
        crate::theme::warning_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.danger,
        crate::theme::danger_label(),
        is_builtin,
        actions,
    );

    ui.add_space(4.0);
    ui.label(crate::theme::interaction_colors_text());
    color_row(
        ui,
        &mut draft.app.hover,
        crate::theme::hover_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.active,
        crate::theme::active_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.selection_bg,
        crate::theme::selection_bg_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.selection_text,
        crate::theme::selection_text_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.border,
        crate::theme::border_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.app.lock,
        crate::theme::lock_label(),
        is_builtin,
        actions,
    );
}

fn show_terminal_appearance_editor(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
) {
    color_dragvalue_row(
        ui,
        &mut draft.typography.terminal_font_size,
        crate::theme::terminal_font_size_label(),
        8.0,
        32.0,
        is_builtin,
        actions,
    );
    color_dragvalue_row(
        ui,
        &mut draft.typography.cell_spacing,
        crate::theme::cell_spacing_label(),
        0.5,
        2.0,
        is_builtin,
        actions,
    );

    ui.add_space(4.0);

    palette_template_selector(ui, actions);

    ui.add_space(4.0);
    ui.label(crate::theme::terminal_base_colors_text());
    color_row(
        ui,
        &mut draft.terminal.foreground,
        crate::theme::fg_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.terminal.background,
        crate::theme::bg_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.terminal.cursor,
        crate::theme::cursor_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.terminal.selection_bg,
        crate::theme::selection_bg_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.terminal.selection_text,
        crate::theme::selection_text_label(),
        is_builtin,
        actions,
    );
    color_row(
        ui,
        &mut draft.terminal.link,
        crate::theme::link_label(),
        is_builtin,
        actions,
    );

    ui.add_space(4.0);
    ansi_palette_grid(
        ui,
        &mut draft.terminal.normal,
        crate::theme::normal_label(),
        is_builtin,
        actions,
    );
    ansi_palette_grid(
        ui,
        &mut draft.terminal.bright,
        crate::theme::bright_label(),
        is_builtin,
        actions,
    );
    ansi_palette_grid(
        ui,
        &mut draft.terminal.dim,
        crate::theme::dim_label(),
        is_builtin,
        actions,
    );
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
        *color = ThemeColor::from_rgb_for_test(srgb[0], srgb[1], srgb[2]);
        actions.push(ThemeAction::DraftModified);
    }
}

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
            *color = ThemeColor::from_rgb_for_test(srgb[0], srgb[1], srgb[2]);
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

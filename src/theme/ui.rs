use crate::theme::model::{AnsiColors, ThemeColor, ThemeDefinition};

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
/// Unified three-state (hover/pressed/focus) styling for hand-drawn
/// buttons. Every ad-hoc button in the app routes through these so the
/// feedback layer is consistent (the v0.1.37 UI audit found three
/// divergent hover implementations and zero pressed/focus states).
pub struct ButtonChrome<'a> {
    pub fg: egui::Color32,
    pub hover_bg: egui::Color32,
    pub active_bg: egui::Color32,
    pub focus_ring: egui::Color32,
    pub corner: f32,
    pub _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ButtonChrome<'a> {
    pub fn from_theme(app: &crate::theme::model::AppTheme, text: egui::Color32) -> Self {
        let hover = app.hover.to_egui();
        // Pressed: hover darkened/lightened toward its complement — a
        // simple alpha blend over the hover color reads clearly on both
        // light and dark themes.
        let active = egui::Color32::from_rgba_unmultiplied(
            hover.r(),
            hover.g(),
            hover.b(),
            (hover.a() as f32 * 0.6) as u8,
        );
        Self {
            fg: text,
            hover_bg: hover,
            active_bg: active,
            focus_ring: app.accent.to_egui(),
            corner: 4.0,
            _marker: Default::default(),
        }
    }

    /// Paint the chrome layers for a response; returns the (possibly
    /// pressed-adjusted) foreground color for the label.
    pub fn paint(&self, ui: &egui::Ui, rect: egui::Rect, resp: &egui::Response) -> egui::Color32 {
        if resp.contains_pointer() {
            ui.painter().rect_filled(rect, self.corner, self.hover_bg);
        }
        let fg = if resp.is_pointer_button_down_on() || resp.clicked() {
            // pressed: slightly dim the glyph
            egui::Color32::from_rgba_unmultiplied(
                self.fg.r(),
                self.fg.g(),
                self.fg.b(),
                (self.fg.a() as f32 * 0.75) as u8,
            )
        } else {
            self.fg
        };
        if resp.has_focus() {
            ui.painter().rect_stroke(
                rect,
                self.corner,
                egui::Stroke::new(1.5_f32, self.focus_ring),
                egui::StrokeKind::Inside,
            );
        }
        fg
    }
}

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
    pub heading: String,
    pub current: String,
    pub unsaved: String,
    pub new_theme: String,
    pub copy_theme: String,
    pub rename_theme: String,
    pub delete_theme: String,
    pub import_theme: String,
    pub export_theme: String,
    pub ui_appearance: String,
    pub base_colors: String,
    pub app_bg_label: String,
    pub sidebar_label: String,
    pub panel_label: String,
    pub input_bg_label: String,
    pub text_colors: String,
    pub text_label: String,
    pub weak_text_label: String,
    pub status_colors: String,
    pub accent_label: String,
    pub warning_label: String,
    pub danger_label: String,
    pub interaction_colors: String,
    pub hover_label: String,
    pub active_label: String,
    pub selection_bg_label: String,
    pub selection_text_label: String,
    pub border_label: String,
    pub lock_label: String,
    pub terminal_appearance: String,
    pub palette_template_label: String,
    pub apply_template: String,
    pub terminal_base_colors: String,
    pub fg_label: String,
    pub bg_label: String,
    pub cursor_label: String,
    pub link_label: String,
    pub normal: String,
    pub bright: String,
    pub dim: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub copy_dialog_title: String,
    pub copy_dialog_hint: String,
    pub new_dialog_title: String,
    pub new_dialog_hint: String,
    pub rename_dialog_title: String,
    pub delete_confirm: String,
    pub switch_confirm: String,
    pub save_and_switch: String,
    pub discard_and_switch: String,
    pub builtin_readonly: String,
    pub keep: String,
    pub discard: String,
    pub ui_font_label_short: String,
    pub ui_font_size_label: String,
    pub terminal_font_label_short: String,
    pub terminal_font_size_label: String,
    pub cell_spacing_label: String,
    pub terminal_padding_label: String,
}

#[allow(clippy::too_many_arguments)]
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
            show_ansi_palette_editor(ui, draft, is_builtin, &mut actions, labels);
        }
    }
    actions
}

fn show_ui_appearance_editor(
    ui: &mut egui::Ui,
    draft: &mut ThemeDefinition,
    available_fonts: &[String],
    _is_builtin: bool,
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
                        // Each CONCRETE entry renders in its own font
                        // (the named family registered in rebuild_fonts)
                        // so the list doubles as a live preview. Generic
                        // names have no named family — rendering them
                        // with FontFamily::Name would panic (unbound), so
                        // they use the default font.
                        let generic_entry =
                            |ui: &mut egui::Ui, name: &str, current: &str| -> bool {
                                ui.selectable_label(current == name, name).clicked()
                            };
                        let font_entry = |ui: &mut egui::Ui, font: &str, current: &str| -> bool {
                            let rich = egui::RichText::new(font)
                                .family(egui::FontFamily::Name(std::sync::Arc::from(font)));
                            ui.selectable_label(current == font, rich).clicked()
                        };
                        // Generic default first.
                        if generic_entry(ui, "system-ui", &current) {
                            draft.app.ui_font_families = vec!["system-ui".into()];
                            actions.push(ThemeAction::DraftModified);
                        }
                        for font in available_fonts {
                            // The caller prepends the generic names; skip
                            // them here so they are not listed twice.
                            if font == "system-ui" || font == "monospace" {
                                continue;
                            }
                            if font_entry(ui, font, &current) {
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
        .on_hover_text(labels.ui_font_size.clone());
    });

    ui.add_space(6.0);

    // Color grid: 3 columns × ~6 rows of compact color cells.
    // Each row: 3 swatch+hex combos using minimal vertical space.
    let c = &labels.colors;
    let pairs: [(&mut ThemeColor, &str, &'static str); 17] = [
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
    _is_builtin: bool,
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
                        // Live preview per CONCRETE entry (own font);
                        // the generic default renders with the default font.
                        let generic_entry =
                            |ui: &mut egui::Ui, name: &str, current: &str| -> bool {
                                ui.selectable_label(current == name, name).clicked()
                            };
                        let font_entry = |ui: &mut egui::Ui, font: &str, current: &str| -> bool {
                            let rich = egui::RichText::new(font)
                                .family(egui::FontFamily::Name(std::sync::Arc::from(font)));
                            ui.selectable_label(current == font, rich).clicked()
                        };
                        if generic_entry(ui, "monospace", &current) {
                            draft.typography.terminal_font_families = vec!["monospace".into()];
                            actions.push(ThemeAction::DraftModified);
                        }
                        for font in available_fonts {
                            if font == "system-ui" || font == "monospace" {
                                continue;
                            }
                            if font_entry(ui, font, &current) {
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
    let pairs: [(&mut ThemeColor, &str, &'static str); 6] = [
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
    _is_builtin: bool,
    actions: &mut Vec<ThemeAction>,
    labels: &ThemeEditorLabels,
) {
    let normal = labels.normal.clone();
    let bright = labels.bright.clone();
    let dim = labels.dim.clone();
    let mut groups: [(String, &mut AnsiColors); 3] = [
        (normal, &mut draft.terminal.normal),
        (bright, &mut draft.terminal.bright),
        (dim, &mut draft.terminal.dim),
    ];
    for (label, colors) in groups.iter_mut() {
        ui.horizontal(|ui| {
            ui.weak(egui::RichText::new(label.as_str()).small());
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
            let black_n = labels.black.clone();
            let red_n = labels.red.clone();
            let green_n = labels.green.clone();
            let yellow_n = labels.yellow.clone();
            let blue_n = labels.blue.clone();
            let magenta_n = labels.magenta.clone();
            let cyan_n = labels.cyan.clone();
            let white_n = labels.white.clone();
            let mut slots: [(&mut ThemeColor, String); 8] = [
                (black, black_n),
                (red, red_n),
                (green, green_n),
                (yellow, yellow_n),
                (blue, blue_n),
                (magenta, magenta_n),
                (cyan, cyan_n),
                (white, white_n),
            ];
            for (color, name) in slots.iter_mut() {
                compact_color_cell(ui, color, name, true, actions, "ansi");
            }
        });
        ui.add_space(4.0);
    }
    let _ = color_dragvalue_row; // suppress unused warning
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

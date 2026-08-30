//! The settings window: persistence (settings.json load/save with
//! corruption quarantine), the shared row/group primitives and the four
//! pages (general / shortcuts / lock / theme). The window frame itself
//! still lives in `update`; these are its building blocks.

use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SettingsWindowState {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

pub(crate) fn settings_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub(crate) fn read_settings_from(
    path: &std::path::Path,
    warnings: &mut Vec<String>,
) -> AppSettings {
    if path.exists() {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match deserialize_settings(&content) {
                    Ok(settings) => {
                        let (settings, changed) = normalize_settings(settings);
                        if changed {
                            let _ = save_settings(&settings);
                        }
                        return settings;
                    }
                    Err(e) => {
                        log::error!("settings.json is corrupt: {e}");
                        if crate::persist::quarantine_corrupt_file(path).is_some() {
                            warnings.push("配置文件已损坏，已备份为 settings.json.corrupt，本次以默认配置启动。".into());
                        } else {
                            warnings.push("配置文件已损坏且无法备份，本次以默认配置启动。".into());
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("failed to read settings.json: {e}");
                warnings.push("配置文件读取失败，本次以默认配置启动。".into());
            }
        }
    }
    AppSettings::default()
}

pub(crate) fn save_settings(settings: &AppSettings) -> Result<(), anyhow::Error> {
    // Errors are logged here so the many `let _ = save_settings(..)`
    // call sites never fail in total silence.
    match crate::persist::atomic_write_json(&settings_path(), settings) {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("failed to persist settings.json: {e}");
            Err(e)
        }
    }
}

impl App {
    /// Unity-inspector settings row: label in a 42% column on the left,
    /// control starting at a fixed column and left-aligned. 32px row with
    /// a hairline divider; controls get a uniform 180px width budget.
    pub(crate) fn settings_row(
        &self,
        ui: &mut egui::Ui,
        label: &str,
        add_control: impl FnOnce(&mut egui::Ui),
    ) {
        let avail = ui.available_rect_before_wrap();
        let row_rect = egui::Rect::from_min_size(avail.min, egui::vec2(avail.width(), 32.0));
        let resp = ui.allocate_rect(row_rect, egui::Sense::hover());
        if resp.hovered() {
            let h = self.active_theme.app.hover.to_egui();
            let dim = egui::Color32::from_rgba_unmultiplied(
                (h.r() as f32 * 0.5) as u8,
                (h.g() as f32 * 0.5) as u8,
                (h.b() as f32 * 0.5) as u8,
                36,
            );
            ui.painter().rect_filled(row_rect, 0.0, dim);
        }
        let b = self.active_theme.app.border.to_egui();
        let divider = egui::Color32::from_rgba_unmultiplied(b.r(), b.g(), b.b(), 40);
        ui.painter()
            .hline(row_rect.x_range(), row_rect.bottom(), (1.0, divider));
        let label_x = row_rect.min.x + 10.0;
        ui.painter().text(
            egui::pos2(label_x, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.0),
            self.active_theme.app.text.to_egui(),
        );
        let ctrl_x = row_rect.min.x + row_rect.width() * 0.42;
        let ctrl_rect = egui::Rect::from_min_max(
            egui::pos2(ctrl_x, row_rect.min.y),
            egui::pos2(row_rect.max.x, row_rect.max.y),
        );
        let mut ctrl_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(ctrl_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        add_control(&mut ctrl_ui);
    }
    /// A full-width action row: content starts flush at the row's left
    /// edge (no label column) and there is no divider below. Used for
    /// button groups like the lock page's change/clear password actions.
    pub(crate) fn settings_action_row(
        &self,
        ui: &mut egui::Ui,
        add_controls: impl FnOnce(&mut egui::Ui),
    ) {
        let avail = ui.available_rect_before_wrap();
        let row_rect = egui::Rect::from_min_size(avail.min, egui::vec2(avail.width(), 32.0));
        let resp = ui.allocate_rect(row_rect, egui::Sense::hover());
        if resp.hovered() {
            let h = self.active_theme.app.hover.to_egui();
            let dim = egui::Color32::from_rgba_unmultiplied(
                (h.r() as f32 * 0.5) as u8,
                (h.g() as f32 * 0.5) as u8,
                (h.b() as f32 * 0.5) as u8,
                36,
            );
            ui.painter().rect_filled(row_rect, 0.0, dim);
        }
        let mut ctrl_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(row_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        ctrl_ui.add_space(10.0);
        add_controls(&mut ctrl_ui);
    }
    /// Weak group heading with consistent rhythm (16px above, 6px below).
    pub(crate) fn settings_group(&self, ui: &mut egui::Ui, title: &str) {
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(title)
                .size(10.0)
                .color(self.active_theme.app.weak_text.to_egui()),
        );
        ui.add_space(6.0);
    }
    pub(crate) fn settings_page_general(&mut self, ui: &mut egui::Ui) {
        let t = self.texts.settings.general.clone();
        let b = self.texts.settings.buttons.clone();

        self.settings_group(ui, &b.behavior_section);
        let mut auto_copy = self.settings_edit.auto_copy_selection;
        self.settings_row(ui, &t.auto_copy, |ui| {
            ui.checkbox(&mut auto_copy, "");
        });
        self.settings_edit.auto_copy_selection = auto_copy;

        let mut auto_match = self.settings_edit.auto_match_command;
        self.settings_row(ui, &t.auto_match, |ui| {
            ui.checkbox(&mut auto_match, "");
        });
        self.settings_edit.auto_match_command = auto_match;

        // Edge-smoothing (feathering) switch + level. Off = hard,
        // terminal-crisp edges; the level scales the feathering width in
        // physical pixels (1.0 = default; text glyph AA is unaffected).
        let mut smooth = self.settings_edit.smooth_rendering;
        self.settings_row(ui, &t.smooth_rendering, |ui| {
            ui.checkbox(&mut smooth, "");
        });
        self.settings_edit.smooth_rendering = smooth;
        if smooth {
            let mut level = self.settings_edit.smooth_level;
            self.settings_row(ui, &t.smooth_level, |ui| {
                ui.add(
                    egui::Slider::new(&mut level, 0.0..=2.0)
                        .show_value(true)
                        .text("px"),
                );
            });
            self.settings_edit.smooth_level = level;
        }

        // Red warning banner on PROD-marked SSH host terminals.
        let mut prod_banner = self.settings_edit.ssh_prod_banner;
        self.settings_row(ui, &t.prod_banner, |ui| {
            ui.checkbox(&mut prod_banner, "");
        });
        self.settings_edit.ssh_prod_banner = prod_banner;

        // AI assistant: user-supplied OpenAI-compatible endpoint.
        let ta = self.texts.ai.clone();
        self.settings_group(ui, &ta.settings_section);
        let mut ai_on = self.settings_edit.ai_enabled;
        self.settings_row(ui, &ta.enable, |ui| {
            ui.checkbox(&mut ai_on, "");
        });
        self.settings_edit.ai_enabled = ai_on;
        if ai_on {
            let mut base_url = self.settings_edit.ai_base_url.clone();
            self.settings_row(ui, &ta.base_url, |ui| {
                ui.add_sized([280.0, 20.0], egui::TextEdit::singleline(&mut base_url));
            });
            self.settings_edit.ai_base_url = base_url;

            let mut api_key = self.settings_edit.ai_api_key.clone();
            self.settings_row(ui, &ta.api_key, |ui| {
                ui.add_sized(
                    [280.0, 20.0],
                    egui::TextEdit::singleline(&mut api_key).password(true),
                );
            });
            self.settings_edit.ai_api_key = api_key;

            let mut model = self.settings_edit.ai_model.clone();
            self.settings_row(ui, &ta.model, |ui| {
                ui.add_sized([280.0, 20.0], egui::TextEdit::singleline(&mut model));
            });
            self.settings_edit.ai_model = model;
        }

        self.settings_group(ui, &b.data_section);
        let mut max_h = self.settings_edit.max_history;
        let mut sb = self.settings_edit.scrollback;
        self.settings_row(ui, &t.max_history, |ui| {
            ui.add_sized(
                [180.0, 20.0],
                egui::DragValue::new(&mut max_h).range(10..=10000),
            );
        });
        self.settings_row(ui, &t.scrollback, |ui| {
            ui.add_sized(
                [180.0, 20.0],
                egui::DragValue::new(&mut sb).range(100..=50000),
            );
        });
        self.settings_edit.max_history = max_h;
        self.settings_edit.scrollback = sb;

        // Default shell for NEW terminals (Windows multi-shell support;
        // a single detected shell hides the row on other platforms).
        if self.detected_shells.len() > 1 {
            let mut shell_id = self.settings_edit.default_shell.clone();
            let shells = self.detected_shells.clone();
            self.settings_row(ui, "默认 Shell", |ui| {
                let selected = shells.iter().position(|s| s.id == shell_id).unwrap_or(0);
                let names: Vec<String> = shells.iter().map(shell_display_name).collect();
                let mut chosen = selected;
                egui::ComboBox::from_id_salt("default_shell")
                    .selected_text(&names[selected])
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for (i, name) in names.iter().enumerate() {
                            ui.selectable_value(&mut chosen, i, name.clone());
                        }
                    });
                if chosen != selected {
                    shell_id = shells[chosen].id.to_string();
                }
            });
            self.settings_edit.default_shell = shell_id;
        }

        // Maintenance action: plain button (same style as all other
        // settings buttons), flush left, no divider; clicking opens a
        // confirmation dialog. The favorites-clear button sits beside it.
        let mut clear = false;
        let mut clear_favs = false;
        let clear_label = t.clear_all_history.clone();
        let clear_favs_label = self.texts.terminal.clear_favorites.clone();
        self.settings_action_row(ui, |ui| {
            if ui.button(&clear_label).clicked() {
                clear = true;
            }
            ui.add_space(8.0);
            if ui.button(clear_favs_label).clicked() {
                clear_favs = true;
            }
        });
        if clear {
            self.show_clear_history_confirm = true;
            self.settings_clear_just_opened = true;
        }
        if clear_favs {
            self.show_clear_favorites_confirm = true;
            self.fav_clear_just_opened = true;
        }

        // Group footer: path hints as weak, wrapping small text.
        ui.add_space(8.0);
        ui.weak(egui::RichText::new(format!("{}  {}", t.scene_path, t.templates_path)).small());
    }
    pub(crate) fn settings_page_shortcuts(&mut self, ui: &mut egui::Ui) {
        let texts = self.texts.clone();
        ui.label(&texts.settings.shortcuts.hint);
        ui.add_space(4.0);
        for id in shortcut_hint_ids() {
            let label = shortcut_label_for(&texts, id).to_string();
            let rec = self.binding_recording.clone();
            let binds = self.settings_edit.key_binds.clone();
            let mut clicked = false;
            self.settings_row(ui, &label, |ui| {
                let text = if rec.as_deref() == Some(id) {
                    "…".to_string()
                } else if let Some(b) = binds.get(id) {
                    shortcut_display(b)
                } else {
                    texts.settings.shortcuts.not_set.clone()
                };
                if ui
                    .add(egui::Button::new(text).min_size(egui::vec2(180.0, 0.0)))
                    .clicked()
                {
                    clicked = true;
                }
            });
            if clicked {
                self.binding_recording = Some(id.to_string());
            }
        }
        ui.add_space(6.0);
        if ui
            .button(&texts.settings.shortcuts.reset_defaults)
            .clicked()
        {
            self.settings_edit.key_binds = default_key_binds();
            self.binding_recording = None;
        }
    }
    pub(crate) fn settings_page_lock(&mut self, ui: &mut egui::Ui) {
        let t = self.texts.settings.lock.clone();
        self.settings_group(ui, &t.password_section);
        let mut action: Option<&'static str> = None;
        if self.settings.lock_password.is_empty() {
            // No password yet: only the "set password" button.
            let set_label = t.set_password.clone();
            self.settings_action_row(ui, |ui| {
                if ui.button(&set_label).clicked() {
                    action = Some("set");
                }
            });
        } else {
            // Password exists: change + clear on one row, both flush left,
            // 20px apart, no divider below.
            let ch_label = t.change_password.clone();
            let cl_label = t.clear_password.clone();
            self.settings_action_row(ui, |ui| {
                if ui.button(&ch_label).clicked() {
                    action = Some("change");
                }
                ui.add_space(20.0);
                if ui.button(&cl_label).clicked() {
                    action = Some("clear");
                }
            });
        }
        match action {
            Some("set") => {
                self.pw_popup = Some("set");
                self.pw_set1.clear();
                self.pw_set2.clear();
                self.pw_message.clear();
            }
            Some("change") => {
                self.pw_popup = Some("change");
                self.pw_old.clear();
                self.pw_new1.clear();
                self.pw_new2.clear();
                self.pw_message.clear();
            }
            Some("clear") => {
                self.pw_popup = Some("clear");
                self.pw_clear.clear();
                self.pw_message.clear();
            }
            _ => {}
        }
    }
    /// Theme page: one page with section headings for 选择与管理 /
    /// UI 外观 / 终端.
    pub(crate) fn settings_page_theme(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if let Some(Err(msg)) = &self.theme_message {
            ui.colored_label(self.active_theme.app.danger.to_egui(), msg);
        }
        if let Some(Ok(msg)) = &self.theme_message {
            ui.colored_label(self.active_theme.app.success.to_egui(), msg);
        }

        // The list takes all the remaining height down to the window bottom.
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(ui.available_width());
                self.settings_page_theme_select(ctx, ui);
            },
        );
    }
    pub(crate) fn settings_page_theme_select(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let current = self.settings_edit.theme_id.clone();

        let mut pick: Option<String> = None;
        let mut edit_target: Option<String> = None;
        let mut copy_target: Option<String> = None;
        let mut delete_target: Option<String> = None;

        let text_col = self.active_theme.app.text.to_egui();
        let _weak_col = self.active_theme.app.weak_text.to_egui();
        let sel_bg = self.active_theme.app.active.to_egui();
        let hover_bg = self.active_theme.app.hover.to_egui();
        let accent_col = self.active_theme.app.accent.to_egui();
        let _border_col = self.active_theme.app.border.to_egui();
        let builtin_tag = self.texts.settings.buttons.builtin.clone();

        // Theme list: fills the remaining settings-page height, one preview
        // row per theme. The whole row is a preview: left half rendered with
        // the theme's UI styling (bg/text/font), right half a terminal demo.
        let scroll_to_top = ui.ctx().memory_mut(|m| {
            m.data
                .remove_temp::<bool>(egui::Id::new("theme_list_scroll_top"))
                .unwrap_or(false)
        });
        let scroll_area = egui::ScrollArea::vertical()
            .id_salt("theme_list_scroll")
            .auto_shrink([false, false])
            .max_height(ui.available_height());
        let mut scroll_area = scroll_area;
        if scroll_to_top {
            scroll_area = scroll_area.vertical_scroll_offset(0.0);
        }
        scroll_area.show(ui, |ui| {
            let weak_col = self.active_theme.app.weak_text.to_egui();
            let mut shown_custom_heading = false;
            let mut shown_builtin_heading = false;
            let group_heading = |ui: &mut egui::Ui, title: &str| {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(title).size(10.0).color(weak_col));
                ui.add_space(4.0);
            };
            for theme in self.available_themes.iter() {
                let is_builtin = crate::theme::store::is_embedded_id(&theme.id);
                if !is_builtin && !shown_custom_heading {
                    shown_custom_heading = true;
                    group_heading(ui, &self.texts.settings.buttons.user_group);
                }
                if is_builtin && !shown_builtin_heading {
                    shown_builtin_heading = true;
                    group_heading(ui, &self.texts.settings.buttons.builtin_group);
                }
                let selected = theme.id == current;
                // Entry = title line above a shortened preview strip.
                let title_h = 22.0;
                let preview_h = 36.0;
                let row_h = title_h + preview_h;
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), row_h),
                    egui::Sense::hover(),
                );
                ui.add_space(4.0); // gap between entries
                                   // Accent bar width (the selected-theme highlight strip).
                let accent_w = 2.0;
                // Clickable/hover region: from just after the accent bar to
                // the terminal preview's right border — the row must not be
                // triggerable beyond the preview blocks.
                let content_left = rect.min.x + accent_w;
                let preview_w_total = (rect.width() - accent_w) * 0.42 * 2.0;
                let clickable_rect = egui::Rect::from_min_max(
                    egui::pos2(content_left, rect.min.y),
                    egui::pos2((content_left + preview_w_total).min(rect.max.x), rect.max.y),
                );
                let hovered = clickable_rect.contains(
                    ui.input(|i| i.pointer.hover_pos())
                        .unwrap_or(rect.min - egui::vec2(1.0, 1.0)),
                );

                // Register the row click FIRST so it sits below the
                // action buttons in the hit-test order.
                let row_resp = ui.interact(
                    clickable_rect,
                    egui::Id::new((&theme.id, "row-click")),
                    egui::Sense::click(),
                );
                if row_resp.clicked() {
                    pick = Some(theme.id.clone());
                }

                // Selected/hover overlay + accent bar.
                if selected {
                    ui.painter().rect_filled(rect, 0.0, sel_bg);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_max(
                            rect.min,
                            egui::pos2(rect.min.x + accent_w, rect.max.y),
                        ),
                        0.0,
                        accent_col,
                    );
                } else if hovered {
                    ui.painter().rect_filled(rect, 0.0, hover_bg);
                }

                // Title line: theme name (+ builtin tag) above the preview,
                // followed by the ANSI palette dots on the same line.
                let name = if is_builtin {
                    format!("{} ({})", theme.name, builtin_tag)
                } else {
                    theme.name.clone()
                };
                let title_cy = rect.min.y + title_h / 2.0;
                let dot_r = 3.0;
                let colors = [
                    theme.terminal.normal.red.to_egui(),
                    theme.terminal.normal.green.to_egui(),
                    theme.terminal.normal.yellow.to_egui(),
                    theme.terminal.normal.blue.to_egui(),
                    theme.terminal.bright.magenta.to_egui(),
                    theme.terminal.bright.cyan.to_egui(),
                    theme.terminal.cursor.to_egui(),
                ];

                // Preview strip area (below the title), shifted right by the
                // accent-bar width so the UI preview no longer covers the
                // selected-theme highlight strip.
                let preview_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x + accent_w, rect.min.y + title_h),
                    rect.max,
                );
                // The action buttons moved up to the title line, so the
                // preview strip no longer reserves a right-hand button
                // column and spans the full row width.
                let content = preview_rect;
                // Preview width: 0.42 of the content width each. Both halves
                // are anchored left.
                let preview_w = content.width() * 0.42;

                // Resolve a FontId for a theme-configured font NAME. The
                // LIST ITEM's theme config must fully decide the preview:
                //  - a registered named font -> that font's own family
                //  - a generic name ("system-ui"/"monospace") -> the CLEAN
                //    default-stack snapshot, NOT the live generic family
                //    (whose head is the ACTIVE theme's font — using it
                //    would make every preview follow the active theme).
                let resolve_font =
                    |name: &str, size: f32, generic: egui::FontFamily| -> egui::FontId {
                        if name.is_empty() || name == "system-ui" || name == "monospace" {
                            let fam = if generic == egui::FontFamily::Monospace {
                                preview_mono_family()
                            } else {
                                preview_prop_family()
                            };
                            return egui::FontId::new(size, fam);
                        }
                        egui::FontId::new(
                            size,
                            egui::FontFamily::Name(std::sync::Arc::from(name.to_owned())),
                        )
                    };
                // Title line: theme name rendered with THIS theme's own
                // UI font (not the active theme's), followed by the ANSI
                // palette dots on the same line.
                let ui_font_name_for_title = theme
                    .app
                    .ui_font_families
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "system-ui".into());
                let title_font = resolve_font(
                    &ui_font_name_for_title,
                    11.0,
                    egui::FontFamily::Proportional,
                );
                let name_galley =
                    ui.fonts(|f| f.layout_no_wrap(name.clone(), title_font, text_col));
                ui.painter().galley(
                    egui::pos2(rect.min.x + 8.0, title_cy - name_galley.size().y / 2.0),
                    name_galley.clone(),
                    text_col,
                );
                {
                    let mut dx = rect.min.x + 8.0 + name_galley.size().x + 10.0;
                    for c in colors {
                        ui.painter()
                            .circle_filled(egui::pos2(dx + dot_r, title_cy), dot_r, c);
                        dx += dot_r * 2.0 + 2.0;
                    }
                }

                // --- Left half: UI-style preview labeled "OpenNex UI:". ---
                let ui_half = egui::Rect::from_min_max(
                    content.min,
                    egui::pos2(content.min.x + preview_w, content.max.y),
                );
                ui.painter()
                    .rect_filled(ui_half, 0.0, theme.app.app_bg.to_egui());
                let ui_font_name = theme
                    .app
                    .ui_font_families
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "system-ui".into());
                // Two lines at FIXED fractional centers (33% / 66% of the
                // block height): changing fonts can no longer shift the
                // lines up or down.
                let label_font_size = theme.app.ui_font_size.min(12.0);
                let meta_font_size = 9.0;
                let line1_cy = ui_half.min.y + ui_half.height() * 0.33;
                let line2_cy = ui_half.min.y + ui_half.height() * 0.66;
                ui.painter().text(
                    egui::pos2(ui_half.min.x + 8.0, line1_cy),
                    egui::Align2::LEFT_CENTER,
                    "OpenNex UI:",
                    resolve_font(
                        &ui_font_name,
                        label_font_size,
                        egui::FontFamily::Proportional,
                    ),
                    theme.app.text.to_egui(),
                );
                ui.painter().text(
                    egui::pos2(ui_half.min.x + 8.0, line2_cy),
                    egui::Align2::LEFT_CENTER,
                    format!("{} {:.0}px", ui_font_name, theme.app.ui_font_size),
                    resolve_font(
                        &ui_font_name,
                        meta_font_size,
                        egui::FontFamily::Proportional,
                    ),
                    theme.app.weak_text.to_egui(),
                );

                // --- Right half: terminal demo labeled "Terminal:". Same
                // width as the UI half (not stretched to the row edge). ---
                let term_half = egui::Rect::from_min_max(
                    egui::pos2(ui_half.max.x, content.min.y),
                    egui::pos2(ui_half.max.x + preview_w, content.max.y),
                );
                let term_bg = theme.terminal.background.to_egui();
                ui.painter().rect_filled(term_half, 0.0, term_bg);
                let term_font_name = theme
                    .typography
                    .terminal_font_families
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "monospace".into());
                let term_size = theme.typography.terminal_font_size.min(12.0);
                let cmd_font =
                    resolve_font(&term_font_name, term_size, egui::FontFamily::Monospace);
                let cmd_galley = ui.fonts(|f| {
                    f.layout_no_wrap(
                        "Terminal:".into(),
                        cmd_font,
                        theme.terminal.foreground.to_egui(),
                    )
                });
                // Fixed fractional line centers (33% / 66%): font changes
                // cannot shift the lines vertically.
                let line1_cy = term_half.min.y + term_half.height() * 0.33;
                let line2_cy = term_half.min.y + term_half.height() * 0.66;
                let cmd_pos =
                    egui::pos2(term_half.min.x + 8.0, line1_cy - cmd_galley.size().y / 2.0);
                ui.painter().galley(
                    cmd_pos,
                    cmd_galley.clone(),
                    theme.terminal.foreground.to_egui(),
                );
                ui.painter().text(
                    egui::pos2(term_half.min.x + 8.0, line2_cy),
                    egui::Align2::LEFT_CENTER,
                    format!(
                        "$ ls -la  {} {:.0}px",
                        term_font_name, theme.typography.terminal_font_size
                    ),
                    resolve_font(&term_font_name, 9.0, egui::FontFamily::Monospace),
                    theme.terminal.dim_foreground.to_egui(),
                );

                // --- Right: per-row action buttons on the TITLE line,
                // vertically centered with the theme name and palette dots;
                // their right edge aligns with the terminal preview's
                // right border.
                let btn_y = title_cy - 9.0;
                let mut x = term_half.max.x;
                let hover_col = self.active_theme.app.hover.to_egui();
                let active_col = self.active_theme.app.active.to_egui();
                let btn = |ui: &mut egui::Ui, x: f32, y: f32, glyph: &str, id: egui::Id| -> bool {
                    let r = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(22.0, 18.0));
                    let resp = ui.interact(r, id, egui::Sense::click());
                    // Three-state feedback (was: static glyph, zero states).
                    if resp.is_pointer_button_down_on() {
                        ui.painter().rect_filled(r, 3.0, active_col);
                    } else if resp.contains_pointer() {
                        ui.painter().rect_filled(r, 3.0, hover_col);
                    }
                    let dim = resp.is_pointer_button_down_on();
                    let col = if dim {
                        egui::Color32::from_rgba_unmultiplied(
                            text_col.r(),
                            text_col.g(),
                            text_col.b(),
                            (text_col.a() as f32 * 0.75) as u8,
                        )
                    } else {
                        text_col
                    };
                    let g = ui.fonts(|f| {
                        f.layout_no_wrap(glyph.to_string(), egui::FontId::proportional(12.0), col)
                    });
                    ui.painter().galley(r.center() - g.size() / 2.0, g, col);
                    resp.clicked()
                };
                // Action buttons laid out right-to-left at fixed offsets:
                // [delete?] [edit?] [new(+)] — 24px pitch, no overlap.
                if !is_builtin
                    && btn(
                        ui,
                        x - 22.0,
                        btn_y,
                        egui_phosphor::regular::TRASH,
                        egui::Id::new((&theme.id, "row-del")),
                    )
                {
                    delete_target = Some(theme.id.clone());
                }
                if !is_builtin {
                    x -= 24.0;
                    if btn(
                        ui,
                        x - 22.0,
                        btn_y,
                        egui_phosphor::regular::PENCIL_SIMPLE,
                        egui::Id::new((&theme.id, "row-ed")),
                    ) {
                        edit_target = Some(theme.id.clone());
                    }
                    x -= 24.0;
                }
                if btn(
                    ui,
                    x - 22.0,
                    btn_y,
                    egui_phosphor::regular::PLUS,
                    egui::Id::new((&theme.id, "row-new")),
                ) {
                    copy_target = Some(theme.id.clone());
                }
            }
        });

        // Handle per-row actions.
        if let Some(id) = pick {
            self.try_switch_theme(ctx, id);
        }
        if let Some(id) = edit_target {
            if let Some(theme) = self.available_themes.iter().find(|t| t.id == id).cloned() {
                self.theme_edit = theme;
                self.theme_edit_origin = Some(id);
                self.theme_editor_open = true;
            }
        }
        if let Some(id) = copy_target {
            let themes_root = crate::theme::store::themes_dir(&app_data_dir());
            let name = format!(
                "{} (copy)",
                self.available_themes
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| t.name.clone())
                    .unwrap_or_default()
            );
            match crate::theme::store::copy_theme(&themes_root, &id, &name) {
                Ok(new_theme) => {
                    self.refresh_themes(&themes_root);
                    self.switch_theme_by_id(ctx, &new_theme.id);
                    // The new theme sits at the top of the list (user themes
                    // first); scroll the list back to the top.
                    ui.ctx().memory_mut(|m| {
                        m.data
                            .insert_temp(egui::Id::new("theme_list_scroll_top"), true)
                    });
                }
                Err(e) => self.theme_message = Some(Err(e.to_string())),
            }
        }
        if let Some(id) = delete_target {
            if let Some(theme) = self.available_themes.iter().find(|t| t.id == id) {
                self.theme_edit = theme.clone();
                self.theme_dialog.show_delete_confirm = true;
            }
        }
    }
}

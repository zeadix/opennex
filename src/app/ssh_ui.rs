//! SSH host book UI (roadmap batch 1): sidebar host section, the
//! create/edit and delete dialogs, and the connection spawn path.

use super::*;

/// Create/edit form state for the SSH host dialog.
pub(crate) struct SshHostDialog {
    /// Some(id) = editing that row; None = creating a new host.
    edit_id: Option<i64>,
    name: String,
    group: String,
    host: String,
    port: u16,
    user: String,
    auth: crate::hosts::SshAuth,
    key_path: String,
    prod: bool,
    /// Set when a confirm attempt failed validation (missing name/host).
    error: bool,
}

impl SshHostDialog {
    fn new() -> Self {
        Self {
            edit_id: None,
            name: String::new(),
            group: String::new(),
            host: String::new(),
            port: 22,
            user: String::new(),
            auth: crate::hosts::SshAuth::default(),
            key_path: String::new(),
            prod: false,
            error: false,
        }
    }

    fn from_host(host: &crate::hosts::SshHost) -> Self {
        Self {
            edit_id: Some(host.id),
            name: host.name.clone(),
            group: host.group.clone(),
            host: host.host.clone(),
            port: host.port,
            user: host.user.clone(),
            auth: host.auth.clone(),
            key_path: host.auth.key_path().unwrap_or("").to_string(),
            prod: host.prod,
            error: false,
        }
    }
}

impl App {
    pub(crate) fn refresh_ssh_hosts(&mut self) {
        self.ssh_hosts = self.history_db.ssh_hosts();
    }
    /// Create and register an SSH session tab for a saved host WITHOUT
    /// docking it — the caller decides where the tab lands. Returns None
    /// (and surfaces a toast) when no ssh client is available.
    pub(crate) fn connect_ssh_host_inner(
        &mut self,
        ctx: &egui::Context,
        host_id: i64,
    ) -> Option<String> {
        let host = self.ssh_hosts.iter().find(|h| h.id == host_id).cloned()?;
        self.terminal_id_counter += 1;
        let Some(instance) = crate::hosts::spawn_ssh_instance(
            ctx,
            self.terminal_id_counter,
            &host,
            self.settings.scrollback,
        ) else {
            self.update_toast = Some((
                self.texts.ssh.ssh_unavailable.clone(),
                std::time::Instant::now() + std::time::Duration::from_secs(8),
            ));
            return None;
        };
        self.tab_counter += 1;
        let tab_id = format!("terminal-{}", self.tab_counter);
        let snapshot = host.ref_snapshot();
        let name = host.name.clone();
        self.terminals.insert(
            tab_id.clone(),
            TerminalData {
                instance,
                name,
                font_size: DEFAULT_FONT_SIZE,
                shell_id: String::new(),
                host: Some(snapshot),
                startup_command: String::new(),
            },
        );
        if self.focused_terminal.is_none() {
            self.focused_terminal = Some(tab_id.clone());
        }
        Some(tab_id)
    }
    /// Sidebar entry point: connect to a host in the ACTIVE workspace's
    /// focused leaf (the leaf holding the current tab).
    fn connect_ssh_host(&mut self, ctx: &egui::Context, host_id: i64) {
        let Some(tab_id) = self.connect_ssh_host_inner(ctx, host_id) else {
            return;
        };
        if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
            tree.push_to_focused_leaf(tab_id.clone());
        }
        self.focused_terminal = Some(tab_id);
    }
    /// Sidebar SSH host section: header + search + one row per host.
    /// A row click connects in the active workspace's focused leaf.
    pub(crate) fn render_ssh_hosts_section(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(2.0);

        // Header: section label left, add button right.
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&self.texts.ssh.section)
                    .size(10.0)
                    .color(self.active_theme.app.weak_text.to_egui()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let add_btn = ui.add(
                    egui::Button::new(egui::RichText::new(egui_phosphor::regular::PLUS).size(11.0))
                        .frame(false),
                );
                if add_btn.clicked() {
                    self.ssh_dialog = Some(SshHostDialog::new());
                    self.ssh_dialog_just_opened = true;
                }
                let _ = add_btn.on_hover_text(&self.texts.ssh.add);
            });
        });

        if self.ssh_hosts.is_empty() {
            ui.label(
                egui::RichText::new(&self.texts.ssh.empty_hint)
                    .size(11.0)
                    .color(self.active_theme.app.weak_text.to_egui()),
            );
            return;
        }

        // Search only earns its space once there is something to search.
        let filter_resp = ui.add(
            egui::TextEdit::singleline(&mut self.ssh_host_filter)
                .hint_text(&self.texts.ssh.search_hint)
                .desired_width(ui.available_width())
                .font(egui::FontId::proportional(12.0)),
        );
        // While the search box owns the keyboard, the terminal view must
        // not steal focus back every frame (keystrokes fell into the PTY).
        self.sidebar_input_focused = filter_resp.has_focus();

        let filter = self.ssh_host_filter.trim().to_lowercase();
        let hosts = self.ssh_hosts.clone();
        let danger = self.active_theme.app.danger.to_egui();
        let text = self.active_theme.app.text.to_egui();
        let weak_text = self.active_theme.app.weak_text.to_egui();
        let hover_fill = ui.visuals().widgets.hovered.weak_bg_fill;
        let mut connect_target = None;
        for host in &hosts {
            if !filter.is_empty()
                && !host.name.to_lowercase().contains(&filter)
                && !host.group.to_lowercase().contains(&filter)
                && !host.host.to_lowercase().contains(&filter)
            {
                continue;
            }
            let row_h = ui.spacing().interact_size.y;
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), row_h),
                egui::Sense::click(),
            );
            ui.painter().rect_filled(
                rect,
                3.0,
                if resp.hovered() {
                    hover_fill
                } else {
                    egui::Color32::TRANSPARENT
                },
            );

            let child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            let icon = if host.prod {
                egui_phosphor::regular::WARNING
            } else {
                egui_phosphor::regular::PLUG
            };
            child.painter().text(
                egui::pos2(rect.min.x + 8.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                icon,
                egui::FontId::proportional(12.0),
                if host.prod { danger } else { weak_text },
            );
            child.painter().text(
                egui::pos2(rect.min.x + 26.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                host.name.clone(),
                egui::FontId::proportional(13.0),
                if host.prod { danger } else { text },
            );
            if resp.clicked() {
                connect_target = Some(host.id);
            }
            // on_hover_text / context_menu consume the response by value.
            let resp = if resp.hovered() {
                resp.on_hover_text(format!(
                    "{} · {}",
                    crate::hosts::ssh_target(host),
                    host.group
                ))
            } else {
                resp
            };
            resp.context_menu(|ui| {
                if ui.button(&self.texts.ssh.connect).clicked() {
                    connect_target = Some(host.id);
                    ui.close_menu();
                }
                if ui.button(&self.texts.ssh.edit).clicked() {
                    self.ssh_dialog = Some(SshHostDialog::from_host(host));
                    self.ssh_dialog_just_opened = true;
                    ui.close_menu();
                }
                if ui.button(&self.texts.ssh.duplicate).clicked() {
                    let mut copy = host.clone();
                    copy.id = 0;
                    copy.name = format!("{} copy", host.name);
                    copy.prod = false;
                    if self.history_db.ssh_host_insert(&copy).is_some() {
                        self.refresh_ssh_hosts();
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(&self.texts.ssh.delete).clicked() {
                    self.ssh_delete_confirm = Some((host.id, host.name.clone()));
                    self.ssh_del_just_opened = true;
                    ui.close_menu();
                }
            });
        }
        if let Some(host_id) = connect_target {
            self.connect_ssh_host(ui.ctx(), host_id);
        }
    }
    /// SSH host create/edit form. Enter confirms (with inline validation),
    /// Esc cancels — same keyboard protocol as the favorite dialogs.
    pub(crate) fn render_ssh_host_dialog(&mut self, ctx: &egui::Context) {
        if self.ssh_dialog.is_none() {
            return;
        }
        // Rising edge: a freshly-opened dialog starts on the safe side.
        if std::mem::take(&mut self.ssh_dialog_just_opened) {
            self.dialog_kb_confirm = false;
        }
        // Unified protocol, BEFORE the Modal so nothing swallows the keys.
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirm = keys.enter || keys.confirm;
        let mut cancel = keys.cancel;
        if keys.close {
            self.ssh_dialog = None;
            return;
        }
        let t = self.texts.ssh.clone();
        let editing = self
            .ssh_dialog
            .as_ref()
            .is_some_and(|d| d.edit_id.is_some());
        let title = if editing {
            t.dialog_edit_title
        } else {
            t.dialog_new_title
        };
        let confirm_id = egui::Id::new("ssh_host_confirm_btn");
        let cancel_id = egui::Id::new("ssh_host_cancel_btn");
        let mut browse_clicked = false;
        egui::Modal::new(egui::Id::new("ssh_host_dialog"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.set_min_width(340.0);
                ui.heading(title);
                ui.add_space(6.0);
                egui::Grid::new("ssh_host_form")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        let field_w = 210.0;
                        ui.label(&t.label_name);
                        ui.add_sized(
                            [field_w, 20.0],
                            egui::TextEdit::singleline(&mut self.ssh_dialog.as_mut().unwrap().name),
                        );
                        ui.end_row();
                        ui.label(&t.label_group);
                        ui.add_sized(
                            [field_w, 20.0],
                            egui::TextEdit::singleline(
                                &mut self.ssh_dialog.as_mut().unwrap().group,
                            ),
                        );
                        ui.end_row();
                        ui.label(&t.label_host);
                        ui.add_sized(
                            [field_w, 20.0],
                            egui::TextEdit::singleline(&mut self.ssh_dialog.as_mut().unwrap().host),
                        );
                        ui.end_row();
                        ui.label(&t.label_port);
                        ui.add_sized(
                            [field_w, 20.0],
                            egui::DragValue::new(&mut self.ssh_dialog.as_mut().unwrap().port)
                                .range(1..=65535),
                        );
                        ui.end_row();
                        ui.label(&t.label_user);
                        ui.add_sized(
                            [field_w, 20.0],
                            egui::TextEdit::singleline(&mut self.ssh_dialog.as_mut().unwrap().user),
                        );
                        ui.end_row();
                        ui.label(&t.label_auth);
                        ui.horizontal(|ui| {
                            let auth = &mut self.ssh_dialog.as_mut().unwrap().auth;
                            ui.radio_value(auth, crate::hosts::SshAuth::Agent, &t.auth_agent);
                            ui.radio_value(auth, crate::hosts::SshAuth::Password, &t.auth_password);
                            ui.radio_value(
                                auth,
                                crate::hosts::SshAuth::Key {
                                    path: String::new(),
                                },
                                &t.auth_key,
                            );
                        });
                        ui.end_row();
                        if self
                            .ssh_dialog
                            .as_ref()
                            .is_some_and(|d| matches!(d.auth, crate::hosts::SshAuth::Key { .. }))
                        {
                            ui.label(&t.label_key_path);
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [field_w - 70.0, 20.0],
                                    egui::TextEdit::singleline(
                                        &mut self.ssh_dialog.as_mut().unwrap().key_path,
                                    ),
                                );
                                if ui.button(&t.browse).clicked() {
                                    browse_clicked = true;
                                }
                            });
                            ui.end_row();
                        }
                        ui.label("");
                        ui.checkbox(&mut self.ssh_dialog.as_mut().unwrap().prod, &t.label_prod);
                        ui.end_row();
                    });
                if self.ssh_dialog.as_ref().is_some_and(|d| d.error) {
                    ui.label(
                        egui::RichText::new(&t.error_required)
                            .size(11.0)
                            .color(self.active_theme.app.danger.to_egui()),
                    );
                }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let (c, x) = Self::dialog_button_row(
                        ui,
                        &mut self.dialog_kb_confirm,
                        confirm_id,
                        cancel_id,
                        &self.texts.theme_editor.dialog_confirm,
                        &self.texts.theme_editor.cancel,
                    );
                    confirm |= c;
                    cancel |= x;
                });
            });
        if browse_clicked {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                if let Some(dlg) = self.ssh_dialog.as_mut() {
                    dlg.key_path = path.to_string_lossy().to_string();
                }
            }
        }
        if confirm || cancel {
            if confirm {
                let Some(dlg) = self.ssh_dialog.take() else {
                    return;
                };
                let name = dlg.name.trim().to_string();
                let host_addr = dlg.host.trim().to_string();
                if name.is_empty() || host_addr.is_empty() {
                    self.ssh_dialog = Some(SshHostDialog { error: true, ..dlg });
                } else {
                    let mut host = crate::hosts::SshHost {
                        id: dlg.edit_id.unwrap_or(0),
                        name,
                        group: dlg.group.trim().to_string(),
                        host: host_addr,
                        port: dlg.port.max(1),
                        user: dlg.user.trim().to_string(),
                        auth: match dlg.auth {
                            crate::hosts::SshAuth::Key { .. } => crate::hosts::SshAuth::Key {
                                path: dlg.key_path.clone(),
                            },
                            other => other,
                        },
                        prod: dlg.prod,
                        sort_key: 0,
                    };
                    match dlg.edit_id {
                        Some(id) => {
                            host.id = id;
                            self.history_db.ssh_host_update(&host);
                        }
                        None => {
                            if let Some(new_id) = self.history_db.ssh_host_insert(&host) {
                                host.id = new_id;
                            }
                        }
                    }
                    self.refresh_ssh_hosts();
                }
            } else {
                self.ssh_dialog = None;
            }
        }
    }
    /// Delete-host confirmation (a stray Enter must NOT kill the entry).
    pub(crate) fn render_ssh_delete_confirm(&mut self, ctx: &egui::Context) {
        let Some((host_id, host_name)) = self.ssh_delete_confirm.clone() else {
            return;
        };
        if std::mem::take(&mut self.ssh_del_just_opened) {
            self.dialog_kb_confirm = false;
        }
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirmed = keys.confirm;
        let mut cancelled = keys.cancel;
        let mut open = true;
        let title = self.texts.ssh.delete_title.clone();
        let body = self.texts.ssh.delete_body.replace("{}", &host_name);
        let mut kb = self.dialog_kb_confirm;
        let inner = egui::Window::new(title)
            .id(egui::Id::new("ssh_delete_confirm_window"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_pos(screen_center(ctx))
            .pivot(egui::Align2::CENTER_CENTER)
            .show(ctx, |ui| {
                ui.label(body);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    Self::dialog_button_row(
                        ui,
                        &mut kb,
                        egui::Id::new("ssh_del_confirm"),
                        egui::Id::new("ssh_del_cancel"),
                        &self.texts.close_confirm.confirm,
                        &self.texts.close_confirm.cancel,
                    )
                })
                .inner
            })
            .and_then(|r| r.inner);
        if let Some((c, x)) = inner {
            confirmed |= c;
            cancelled |= x;
        }
        if keys.close {
            cancelled = true;
        }
        if confirmed {
            self.history_db.ssh_host_delete(host_id);
            self.refresh_ssh_hosts();
            self.ssh_delete_confirm = None;
        }
        if cancelled || !open {
            self.ssh_delete_confirm = None;
        }
    }
}

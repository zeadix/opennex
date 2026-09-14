//! Snippet `{placeholder}` fill-in and per-terminal startup command
//! dialogs (roadmap batch 3).

use super::*;

/// Fill-in state for a snippet with `{placeholder}` tokens: the target
/// tab, the raw template and one input per unique token.
pub(crate) struct SnippetFillState {
    tab: String,
    template: String,
    tokens: Vec<String>,
    values: Vec<String>,
}

/// Field-wise [`App::open_snippet_fill`] for rendering closures that
/// already borrow parts of `self` (egui's disjoint closure captures
/// cannot express a `&mut self` method call there).
#[allow(clippy::too_many_arguments)]
pub(crate) fn open_snippet_fill_fields(
    terminals: &mut HashMap<String, TerminalData>,
    fav_submenu: &mut Option<(i64, egui::Pos2, Vec<String>, Option<usize>)>,
    fav_sub_focused: &mut bool,
    history_menu_just_closed: &mut HashMap<String, bool>,
    snippet_fill: &mut Option<SnippetFillState>,
    snippet_fill_just_opened: &mut bool,
    tab: &str,
    template: String,
) -> bool {
    let tokens = snippet_tokens(&template);
    if tokens.is_empty() {
        return false;
    }
    let values = vec![String::new(); tokens.len()];
    if let Some(td) = terminals.get_mut(tab) {
        td.instance.history_nav = None;
    }
    *fav_submenu = None;
    *fav_sub_focused = false;
    history_menu_just_closed.insert(tab.to_string(), true);
    *snippet_fill = Some(SnippetFillState {
        tab: tab.to_string(),
        template,
        tokens,
        values,
    });
    *snippet_fill_just_opened = true;
    true
}

/// Unique `{placeholder}` tokens of a snippet, left to right. A token is
/// anything except braces between `{` and `}`, trimmed; empty and
/// duplicate tokens are dropped.
pub(crate) fn snippet_tokens(template: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let Some(close_rel) = rest[open + 1..].find('}') else {
            break;
        };
        let token = rest[open + 1..open + 1 + close_rel].trim();
        if !token.is_empty() && !tokens.iter().any(|t| t == token) {
            tokens.push(token.to_string());
        }
        rest = &rest[open + 1 + close_rel + 1..];
    }
    tokens
}

/// Substitute `{token}` occurrences with their values; unknown or
/// unfilled tokens are left verbatim so the user can spot them.
pub(crate) fn snippet_expand(template: &str, values: &[(String, String)]) -> String {
    let mut out = template.to_string();
    for (token, value) in values {
        if !token.is_empty() {
            out = out.replace(&format!("{{{token}}}"), value);
        }
    }
    out
}

/// Verdict of a rendered dialog frame.
pub(crate) enum DialogVerdict {
    Confirmed,
    Cancelled,
    Open,
}

impl App {
    /// UNIFIED confirm-dialog shell (UI style sheet, v0.1.56): a centered
    /// Modal (360x96 min, margin 12) with a heading (danger-tinted when
    /// `danger`), body text and a bottom button strip driven by
    /// `dialog_button_row` + the `dialog_keys` cursor protocol.
    ///
    /// `keys` MUST come from `dialog_keys(ctx, ..)` called by the caller
    /// BEFORE this (so Esc/Enter never leak to the terminal). A backdrop
    /// click counts as cancel. Callers keep ownership of their open-flag
    /// and act on the returned verdict.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn confirm_dialog_shell(
        &mut self,
        ctx: &egui::Context,
        modal_id: &str,
        title: &str,
        body: &str,
        danger: bool,
        confirm_label: &str,
        cancel_label: &str,
        keys: DialogKeysOutcome,
    ) -> DialogVerdict {
        use crate::theme::ui::kit;
        let mut confirmed = keys.confirm;
        let mut cancelled = keys.cancel;
        let mut kb = self.dialog_kb_confirm;
        let title = title.to_string();
        let body = body.to_string();
        let confirm_txt = confirm_label.to_string();
        let cancel_txt = cancel_label.to_string();
        let danger_color = self.active_theme.app.danger.to_egui();
        let text_col = self.active_theme.app.text.to_egui();
        let heading_color = if danger { danger_color } else { text_col };
        let body_color = if danger { danger_color } else { text_col };
        let modal = egui::Modal::new(egui::Id::new(modal_id))
            .frame(
                egui::Frame::window(&ctx.style())
                    .inner_margin(egui::Margin::same(kit::DIALOG_MARGIN)),
            )
            .show(ctx, |ui| {
                ui.set_min_size(egui::vec2(kit::DIALOG_WIDTH, kit::DIALOG_MIN_HEIGHT));
                ui.heading(egui::RichText::new(&title).color(heading_color));
                kit::apply_dialog_spacing(ui);
                egui::TopBottomPanel::bottom(egui::Id::new((modal_id, "footer")))
                    .frame(egui::Frame::new())
                    .exact_height(kit::DIALOG_FOOTER_HEIGHT)
                    .show_inside(ui, |ui| {
                        ui.add_space(20.0);
                        let (c, x) = Self::dialog_button_row(
                            ui,
                            &mut kb,
                            egui::Id::new((modal_id, "confirm")),
                            egui::Id::new((modal_id, "cancel")),
                            &confirm_txt,
                            &cancel_txt,
                        );
                        confirmed |= c;
                        cancelled |= x;
                    });
                ui.label(
                    egui::RichText::new(body)
                        .size(kit::FONT_STRONG)
                        .color(body_color),
                );
            });
        self.dialog_kb_confirm = kb;
        if modal.backdrop_response.clicked() {
            cancelled = true;
        }
        if confirmed {
            DialogVerdict::Confirmed
        } else if cancelled {
            DialogVerdict::Cancelled
        } else {
            DialogVerdict::Open
        }
    }

    /// If `template` contains `{placeholder}` tokens, open the fill dialog
    /// (closing the history menu first) and return true; otherwise return
    /// false so the caller inserts the command directly.
    pub(crate) fn open_snippet_fill(&mut self, tab: &str, template: String) -> bool {
        open_snippet_fill_fields(
            &mut self.terminals,
            &mut self.fav_submenu,
            &mut self.fav_sub_focused,
            &mut self.history_menu_just_closed,
            &mut self.snippet_fill,
            &mut self.snippet_fill_just_opened,
            tab,
            template,
        )
    }
    /// One input per `{token}`; confirm writes the expanded command into
    /// the target terminal WITHOUT executing (same as the history list).
    pub(crate) fn render_snippet_fill_dialog(&mut self, ctx: &egui::Context) {
        if self.snippet_fill.is_none() {
            return;
        }
        if std::mem::take(&mut self.snippet_fill_just_opened) {
            self.dialog_kb_confirm = false;
        }
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, false);
        let mut confirm = keys.enter || keys.confirm;
        let mut cancel = keys.cancel;
        if keys.close {
            self.snippet_fill = None;
            return;
        }
        let title = self.texts.terminal.snippet_fill_title.clone();
        let hint = self.texts.terminal.snippet_fill_hint.clone();
        egui::Modal::new(egui::Id::new("snippet_fill_dialog"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.set_min_width(340.0);
                ui.heading(title);
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(hint)
                        .size(11.0)
                        .color(self.active_theme.app.weak_text.to_egui()),
                );
                ui.add_space(4.0);
                egui::Grid::new("snippet_fill_grid")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        let token_count = self.snippet_fill.as_ref().unwrap().tokens.len();
                        for i in 0..token_count {
                            let token = self.snippet_fill.as_ref().unwrap().tokens[i].clone();
                            ui.label(format!("{{{token}}}:"));
                            ui.add_sized(
                                [220.0, 20.0],
                                egui::TextEdit::singleline(
                                    &mut self.snippet_fill.as_mut().unwrap().values[i],
                                ),
                            );
                            ui.end_row();
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let (c, x) = Self::dialog_button_row(
                        ui,
                        &mut self.dialog_kb_confirm,
                        egui::Id::new("snippet_fill_confirm"),
                        egui::Id::new("snippet_fill_cancel"),
                        &self.texts.theme_editor.dialog_confirm,
                        &self.texts.theme_editor.cancel,
                    );
                    confirm |= c;
                    cancel |= x;
                });
            });
        if cancel {
            self.snippet_fill = None;
        } else if confirm {
            let Some(state) = self.snippet_fill.take() else {
                return;
            };
            let values: Vec<(String, String)> = state
                .tokens
                .iter()
                .cloned()
                .zip(state.values.iter().cloned())
                .collect();
            let line = snippet_expand(&state.template, &values);
            let tab = state.tab;
            if let Some(td) = self.terminals.get_mut(&tab) {
                td.instance.write(line.as_bytes());
            }
            self.history_menu_just_closed.insert(tab, true);
        }
    }
    /// Per-terminal startup command editor. Confirm SAVES and RUNS the
    /// command immediately (parity with scene restore); an empty value
    /// clears it.
    pub(crate) fn render_startup_cmd_dialog(&mut self, ctx: &egui::Context) {
        if self.startup_cmd_dialog.is_none() {
            return;
        }
        if std::mem::take(&mut self.startup_cmd_just_opened) {
            self.dialog_kb_confirm = false;
        }
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, false);
        let mut confirm = keys.enter || keys.confirm;
        let mut cancel = keys.cancel;
        if keys.close {
            self.startup_cmd_dialog = None;
            return;
        }
        let title = self.texts.terminal.startup_cmd_title.clone();
        let hint = self.texts.terminal.startup_cmd_hint.clone();
        egui::Modal::new(egui::Id::new("startup_cmd_dialog"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                ui.heading(title);
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(hint)
                        .size(11.0)
                        .color(self.active_theme.app.weak_text.to_egui()),
                );
                ui.add_space(4.0);
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.startup_cmd_dialog.as_mut().unwrap().1)
                        .desired_width(ui.available_width())
                        .font(egui::FontId::monospace(13.0)),
                );
                resp.request_focus();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let (c, x) = Self::dialog_button_row(
                        ui,
                        &mut self.dialog_kb_confirm,
                        egui::Id::new("startup_cmd_confirm"),
                        egui::Id::new("startup_cmd_cancel"),
                        &self.texts.theme_editor.dialog_confirm,
                        &self.texts.theme_editor.cancel,
                    );
                    confirm |= c;
                    cancel |= x;
                });
            });
        if cancel {
            self.startup_cmd_dialog = None;
        } else if confirm {
            let Some((tab, cmd)) = self.startup_cmd_dialog.take() else {
                return;
            };
            let cmd = cmd.trim().to_string();
            if let Some(td) = self.terminals.get_mut(&tab) {
                td.startup_command = cmd.clone();
                run_startup_command(&mut td.instance, &cmd);
            }
        }
    }
}

#[cfg(test)]
mod snippet_tests {
    use super::{snippet_expand, snippet_tokens};

    #[test]
    fn tokens_are_unique_and_ordered() {
        assert_eq!(
            snippet_tokens("ssh {host} -p {port} && ssh {host}"),
            vec!["host", "port"]
        );
        assert!(snippet_tokens("no placeholders").is_empty());
        // Unclosed brace / empty token are ignored.
        assert!(snippet_tokens("oops {unclosed").is_empty());
        assert!(snippet_tokens("empty {} braces").is_empty());
    }

    #[test]
    fn expand_fills_known_tokens_and_keeps_unknown() {
        let values = vec![
            ("host".to_string(), "10.0.0.1".to_string()),
            ("port".to_string(), "2222".to_string()),
        ];
        assert_eq!(
            snippet_expand("ssh {host} -p {port} # {unknown}", &values),
            "ssh 10.0.0.1 -p 2222 # {unknown}"
        );
    }
}

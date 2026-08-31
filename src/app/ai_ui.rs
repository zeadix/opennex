//! Floating AI assistant panel (v2): multi-turn conversation, terminal
//! context injection, the right-click "AI" actions, PROD-guarded insert
//! & run, and snippet saving.

use super::*;

/// What the terminal right-click "AI" menu asked for. The menu only
/// records the intent (it lives inside the terminal-tab borrow scope);
/// the action itself executes on the App afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiCtxAction {
    ExplainSelection,
    FixSelection,
    TranslateSelection,
    ExplainScreen,
}

/// Cap for terminal content sent to the model.
const MAX_CONTEXT_CHARS: usize = 4000;

/// Turns of history sent with each request (token bound); the system
/// prompt comes on top of these.
const MAX_HISTORY_TURNS: usize = 20;

impl App {
    /// Floating AI panel: drains finished background responses, shows the
    /// transcript and the action buttons. Deferred right-click intents are
    /// picked up here (the dock renders after the dispatch block).
    pub(crate) fn render_ai_panel(&mut self, ctx: &egui::Context) {
        // Drain finished responses even while the window is closed, so a
        // late reply never resurrects stale state on reopen.
        if let Some(rx) = &self.ai_rx {
            while let Ok(outcome) = rx.try_recv() {
                self.ai_busy = false;
                match outcome {
                    Ok(answer) => {
                        self.ai_messages.push(crate::ai::ChatMessage {
                            role: "assistant",
                            content: answer,
                        });
                        self.ai_error = None;
                    }
                    Err(err) => self.ai_error = Some(err),
                }
            }
        }
        if let Some(action) = self.ai_ctx_intent.take() {
            self.ai_context_action(ctx, action);
        }
        if !self.show_ai_panel {
            return;
        }
        let mut open = true;
        egui::Window::new(&self.texts.ai.panel_title)
            .id(egui::Id::new("ai_panel"))
            .open(&mut open)
            .default_width(430.0)
            .default_pos(screen_center(ctx) + egui::vec2(140.0, -120.0))
            .show(ctx, |ui| {
                let t = self.texts.ai.clone();
                let weak = self.active_theme.app.weak_text.to_egui();
                if !self.settings.ai_enabled {
                    ui.label(&t.not_enabled);
                    return;
                }
                // Transcript (multi-turn).
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.ai_messages.is_empty() {
                            ui.label(egui::RichText::new(&t.empty_hint).size(11.0).color(weak));
                        }
                        for msg in &self.ai_messages {
                            if msg.role == "user" {
                                ui.label(
                                    egui::RichText::new(format!("› {}", msg.content))
                                        .size(11.0)
                                        .color(weak),
                                );
                            } else {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(&msg.content).size(12.0))
                                        .wrap(),
                                );
                            }
                            ui.add_space(3.0);
                        }
                        if self.ai_busy {
                            ui.label(egui::RichText::new(&t.generating).size(11.0).color(weak));
                        }
                    });
                if let Some(err) = &self.ai_error {
                    ui.label(
                        egui::RichText::new(format!("{} {err}", t.error_label))
                            .size(11.0)
                            .color(self.active_theme.app.danger.to_egui()),
                    );
                }
                ui.separator();
                ui.add(
                    egui::TextEdit::multiline(&mut self.ai_prompt)
                        .hint_text(&t.prompt_hint)
                        .desired_rows(2)
                        .desired_width(ui.available_width()),
                );
                ui.add_space(4.0);
                let has_answer = self.ai_last_assistant().is_some();
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.ai_busy, egui::Button::new(&t.send))
                        .clicked()
                    {
                        let prompt = self.ai_prompt.clone();
                        self.ai_send(ctx, self.chat_system(), prompt);
                    }
                    if ui
                        .add_enabled(!self.ai_busy, egui::Button::new(&t.explain_selection))
                        .clicked()
                    {
                        self.ai_context_action(ctx, AiCtxAction::ExplainSelection);
                    }
                    let has_answer = self.ai_last_assistant().is_some();
                    if ui
                        .add_enabled(has_answer, egui::Button::new(&t.insert_to_terminal))
                        .clicked()
                    {
                        self.ai_insert_response();
                    }
                    if ui
                        .add_enabled(has_answer, egui::Button::new(&t.insert_run))
                        .clicked()
                    {
                        self.ai_insert_run_response();
                    }
                });
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(has_answer, egui::Button::new(&t.save_snippet))
                        .clicked()
                    {
                        self.ai_save_snippet();
                    }
                    if ui.button(&t.clear).clicked() {
                        self.ai_messages.clear();
                        self.ai_error = None;
                    }
                });
                // The terminal agent lives below the chat (same panel).
                self.render_agent_section(ui);
            });
        if !open {
            self.show_ai_panel = false;
        }
    }

    /// The latest assistant answer, if any.
    fn ai_last_assistant(&self) -> Option<String> {
        self.ai_messages
            .iter()
            .rev()
            .find(|m| m.role == "assistant")
            .map(|m| m.content.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Free-form chat uses the user's UI language as the reply target
    /// (the locale code maps to a language name for the system prompt).
    fn chat_system(&self) -> String {
        let lang = self.ui_language_name();
        format!(
            "You are a senior terminal and ops assistant embedded in the OpenNex terminal \
manager. The user works in a shell. Answer concisely and practically; prefer copy-pasteable \
commands over prose. Reply in {lang}."
        )
    }

    fn ui_language_name(&self) -> &'static str {
        match self.settings.language.as_str() {
            "zh" => "Simplified Chinese",
            "zh-TW" => "Traditional Chinese",
            "de" => "German",
            "fr" => "French",
            "ja" => "Japanese",
            "it" => "Italian",
            "ko" => "Korean",
            "hi" => "Hindi",
            _ => "English",
        }
    }

    /// System prompt for a right-click action (translator targets the UI
    /// language, so it is built per call).
    fn context_system(&self, action: AiCtxAction) -> String {
        match action {
            AiCtxAction::ExplainSelection | AiCtxAction::ExplainScreen => {
                crate::ai::EXPLAIN_SYSTEM.to_string()
            }
            AiCtxAction::FixSelection => crate::ai::FIX_SYSTEM.to_string(),
            AiCtxAction::TranslateSelection => {
                let lang = self.ui_language_name();
                format!(
                    "You are a translator embedded in the OpenNex terminal manager. Translate \
the user's content (inside a code block) into {lang}. Reply with ONLY the translation — \
preserve technical terms, identifiers and formatting."
                )
            }
        }
    }

    /// Terminal context silently attached to every request: where the
    /// shell is, which machine it is on, what ran last. Screen CONTENT is
    /// only sent when the user explicitly asks for it (selection/screen).
    fn ai_context_preamble(&self, tab: &str) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(td) = self.terminals.get(tab) {
            if !td.instance.cwd.is_empty() {
                parts.push(format!("cwd: {}", td.instance.cwd));
            }
            if let Some(host) = &td.host {
                parts.push(format!(
                    "host: {}{}",
                    host.addr,
                    if host.prod { " (PRODUCTION)" } else { "" }
                ));
            }
        }
        if let Some(last) = self.history_db.get(tab, 1).first() {
            parts.push(format!("last command: {last}"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("Terminal context:\n{}\n\n", parts.join("\n"))
        }
    }

    /// Fire a background multi-turn request; the reply lands in `ai_rx`
    /// and is drained on later frames. The user turn is pushed BEFORE the
    /// spawn so a failed request still shows what was asked.
    pub(crate) fn ai_send(&mut self, ctx: &egui::Context, system: String, user: String) {
        if self.ai_busy {
            return;
        }
        let user = user.trim().to_string();
        if user.is_empty() {
            return;
        }
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        let preamble = self.ai_context_preamble(&tab);
        let user_content = if preamble.is_empty() {
            user
        } else {
            format!("{preamble}{user}")
        };
        self.ai_messages.push(crate::ai::ChatMessage {
            role: "user",
            content: user_content,
        });
        let mut messages = vec![crate::ai::ChatMessage {
            role: "system",
            content: system,
        }];
        let history_start = self.ai_messages.len().saturating_sub(MAX_HISTORY_TURNS);
        messages.extend(self.ai_messages[history_start..].iter().cloned());
        let cfg = crate::ai::AiConfig {
            base_url: self.settings.ai_base_url.clone(),
            api_key: self.settings.ai_api_key.clone(),
            model: self.settings.ai_model.clone(),
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.ai_rx = Some(rx);
        self.ai_busy = true;
        std::thread::spawn(move || {
            let _ = tx.send(crate::ai::complete_messages(&cfg, &messages, 60));
        });
        ctx.request_repaint();
    }

    /// Execute a deferred right-click intent against the focused terminal.
    fn ai_context_action(&mut self, ctx: &egui::Context, action: AiCtxAction) {
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        let content = match action {
            AiCtxAction::ExplainSelection
            | AiCtxAction::FixSelection
            | AiCtxAction::TranslateSelection => {
                let sel = self
                    .terminals
                    .get(&tab)
                    .map(|td| td.instance.backend.selectable_content())
                    .unwrap_or_default();
                let sel: String = sel.trim().chars().take(MAX_CONTEXT_CHARS).collect();
                if sel.is_empty() {
                    // No selection: fall back to the visible screen.
                    let screen = self
                        .terminals
                        .get(&tab)
                        .map(|td| td.instance.visible_text(MAX_CONTEXT_CHARS))
                        .unwrap_or_default();
                    let screen: String = screen.trim().chars().take(MAX_CONTEXT_CHARS).collect();
                    if screen.is_empty() {
                        return;
                    }
                    screen
                } else {
                    sel
                }
            }
            AiCtxAction::ExplainScreen => {
                let screen = self
                    .terminals
                    .get(&tab)
                    .map(|td| td.instance.visible_text(MAX_CONTEXT_CHARS))
                    .unwrap_or_default();
                let screen: String = screen.trim().chars().take(MAX_CONTEXT_CHARS).collect();
                if screen.is_empty() {
                    return;
                }
                screen
            }
        };
        let system = self.context_system(action);
        self.show_ai_panel = true;
        self.ai_send(ctx, system, format!("```\n{content}\n```"));
    }

    /// Write the latest answer into the FOCUSED terminal (text only —
    /// the user reviews it before pressing Enter).
    pub(crate) fn ai_insert_response(&mut self) {
        let Some(text) = self.ai_last_assistant() else {
            return;
        };
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        if let Some(td) = self.terminals.get_mut(&tab) {
            td.instance.write(text.as_bytes());
        }
    }

    /// Insert the latest answer AND run it. On a PROD-marked host this
    /// detours through the danger confirm dialog instead.
    pub(crate) fn ai_insert_run_response(&mut self) {
        let Some(text) = self.ai_last_assistant() else {
            return;
        };
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        let is_prod = self
            .terminals
            .get(&tab)
            .and_then(|td| td.host.as_ref())
            .is_some_and(|h| h.prod);
        if is_prod {
            self.ai_exec_confirm = Some((tab, text));
            self.ai_exec_just_opened = true;
        } else {
            self.ai_write_and_run(&tab, &text);
        }
    }

    fn ai_write_and_run(&mut self, tab: &str, text: &str) {
        if let Some(td) = self.terminals.get_mut(tab) {
            td.instance.write(text.as_bytes());
            td.instance.write(b"\r");
        }
    }

    /// Danger confirmation for running an AI command on a PROD host.
    pub(crate) fn render_ai_exec_confirm(&mut self, ctx: &egui::Context) {
        let Some((tab, command)) = self.ai_exec_confirm.clone() else {
            return;
        };
        if std::mem::take(&mut self.ai_exec_just_opened) {
            self.dialog_kb_confirm = false;
        }
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirmed = keys.confirm;
        let mut cancelled = keys.cancel;
        let mut open = true;
        let title = self.texts.ai.exec_confirm_title.clone();
        let host = self
            .terminals
            .get(&tab)
            .and_then(|td| td.host.as_ref())
            .map(|h| h.addr.clone())
            .unwrap_or_default();
        let preview: String = command.chars().take(300).collect();
        let body = self
            .texts
            .ai
            .exec_confirm_body
            .replace("{}", &host)
            .replace("{}", &preview);
        let mut kb = self.dialog_kb_confirm;
        let inner = egui::Window::new(title)
            .id(egui::Id::new("ai_exec_confirm_window"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_pos(screen_center(ctx))
            .pivot(egui::Align2::CENTER_CENTER)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new(&body).color(self.active_theme.app.danger.to_egui()));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    Self::dialog_button_row(
                        ui,
                        &mut kb,
                        egui::Id::new("ai_exec_confirm"),
                        egui::Id::new("ai_exec_cancel"),
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
            self.ai_exec_confirm = None;
            self.ai_write_and_run(&tab, &command);
        }
        if cancelled || !open {
            self.ai_exec_confirm = None;
        }
    }

    /// Save the first non-empty line of the latest answer into the default
    /// snippet folder (the snippets library is for runnable commands).
    pub(crate) fn ai_save_snippet(&mut self) {
        let Some(text) = self.ai_last_assistant() else {
            return;
        };
        let Some(first_line) = text.lines().map(str::trim).find(|l| !l.is_empty()) else {
            return;
        };
        self.history_db.fav_add(first_line);
        self.fav_folders = self.history_db.fav_folders();
        self.update_toast = Some((
            self.texts.ai.saved_toast.clone(),
            std::time::Instant::now() + std::time::Duration::from_secs(5),
        ));
    }
}

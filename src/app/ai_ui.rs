//! Floating AI assistant panel (roadmap batch 2): prompt box, terminal
//! output explanation and one-click insert into the focused terminal.

use super::*;

impl App {
    /// Floating AI panel: drains finished background responses, shows the
    /// transcript and the three actions (send / explain output / insert).
    pub(crate) fn render_ai_panel(&mut self, ctx: &egui::Context) {
        // Drain finished responses even while the window is closed, so a
        // late reply never resurrects stale state on reopen.
        if let Some(rx) = &self.ai_rx {
            while let Ok(outcome) = rx.try_recv() {
                self.ai_busy = false;
                match outcome {
                    Ok(answer) => {
                        self.ai_response = answer;
                        self.ai_error = None;
                    }
                    Err(err) => self.ai_error = Some(err),
                }
            }
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
                if !self.settings.ai_enabled {
                    ui.label(&t.not_enabled);
                    return;
                }
                if !self.ai_response.is_empty() {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.ai_response)
                            .desired_rows(7)
                            .desired_width(ui.available_width()),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(&t.empty_hint)
                            .size(11.0)
                            .color(self.active_theme.app.weak_text.to_egui()),
                    );
                }
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
                ui.horizontal(|ui| {
                    if self.ai_busy {
                        ui.label(
                            egui::RichText::new(&t.generating)
                                .size(11.0)
                                .color(self.active_theme.app.weak_text.to_egui()),
                        );
                    }
                    if ui
                        .add_enabled(!self.ai_busy, egui::Button::new(&t.send))
                        .clicked()
                    {
                        let prompt = self.ai_prompt.clone();
                        self.ai_send(ctx, crate::ai::CHAT_SYSTEM, prompt);
                    }
                    if ui
                        .add_enabled(!self.ai_busy, egui::Button::new(&t.explain_selection))
                        .clicked()
                    {
                        self.ai_explain_selection(ctx);
                    }
                    if ui
                        .add_enabled(
                            !self.ai_busy && !self.ai_response.trim().is_empty(),
                            egui::Button::new(&t.insert_to_terminal),
                        )
                        .clicked()
                    {
                        self.ai_insert_response();
                    }
                    if ui.button(&t.clear).clicked() {
                        self.ai_response.clear();
                        self.ai_error = None;
                    }
                });
            });
        if !open {
            self.show_ai_panel = false;
        }
    }
    /// Fire a background chat-completion request; the reply lands in
    /// `ai_rx` and is drained on later frames.
    fn ai_send(&mut self, ctx: &egui::Context, system: &'static str, user: String) {
        if self.ai_busy || user.trim().is_empty() {
            return;
        }
        let cfg = crate::ai::AiConfig {
            base_url: self.settings.ai_base_url.clone(),
            api_key: self.settings.ai_api_key.clone(),
            model: self.settings.ai_model.clone(),
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.ai_rx = Some(rx);
        self.ai_busy = true;
        std::thread::spawn(move || {
            let _ = tx.send(crate::ai::complete(&cfg, system, &user, 60));
        });
        ctx.request_repaint();
    }
    /// Ask the model to explain the focused terminal's selected (or all
    /// visible) output.
    fn ai_explain_selection(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        let Some(td) = self.terminals.get(&tab) else {
            return;
        };
        let content = td.instance.backend.selectable_content();
        let content: String = content.trim().chars().take(4000).collect();
        if content.is_empty() {
            return;
        }
        self.ai_send(
            ctx,
            crate::ai::EXPLAIN_SYSTEM,
            format!("```\n{content}\n```"),
        );
    }
    /// Write the current answer into the FOCUSED terminal (text only —
    /// the user reviews it before pressing Enter).
    fn ai_insert_response(&mut self) {
        let text = self.ai_response.trim().to_string();
        if text.is_empty() {
            return;
        }
        if let Some(tab) = self.focused_terminal.clone() {
            if let Some(td) = self.terminals.get_mut(&tab) {
                td.instance.write(text.as_bytes());
            }
        }
    }
}

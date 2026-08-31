//! The terminal agent state machine and its panel UI (v1): one goal,
//! one terminal, one command at a time, behind the safety gate.
//!
//! Loop: Thinking (model call) -> Gate (classify + approve) -> Executing
//! (write + \\r) -> Waiting (prompt_seq / screen stability / timeout) ->
//! Observing (screen snapshot fed back) -> Thinking ... until done,
//! step cap, timeout or user stop.

use super::*;
use crate::app::agent::{ApprovalMode, GateDecision, WaitSignal};

/// Status of one completed step in the transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepStatus {
    Running,
    Ok,
    Denied,
    Cancelled,
}

/// One transcript row.
#[derive(Debug, Clone)]
pub(crate) struct AgentStep {
    pub command: String,
    /// Short reason the model gave for this command.
    pub reason: String,
    pub status: StepStatus,
    /// First screen line(s) after the command finished (observation).
    pub outcome: String,
}

/// What the loop is doing right now.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AgentPhase {
    /// Model call in flight.
    Thinking,
    /// A command is waiting for user confirmation (the confirm dialog
    /// owns the UI; the tick does not advance).
    WaitingConfirm,
    /// Command written; waiting for completion.
    Executing,
    /// Completion wait exceeded the timeout: user picks continue/stop.
    TimedOut,
    /// The model asked the user something.
    NeedInput(String),
    /// Goal achieved (model sent `done`).
    Done,
    /// Terminal error / stop.
    Failed(String),
}

/// Completion-wait state for the Executing phase.
#[derive(Debug, Clone)]
pub(crate) struct AgentWait {
    pub started: std::time::Instant,
    /// prompt_seq snapshot before the command ran.
    pub prompt_snapshot: u64,
    /// Whether the shell integration is present (prompt_seq ever > 0).
    pub has_prompt_signal: bool,
    /// Last visible-screen hash for the stability heuristic.
    pub screen_hash: u64,
    pub stable_since: Option<std::time::Instant>,
}

/// The full run state, owned by `App.agent`.
pub(crate) struct AgentRun {
    pub goal: String,
    pub tab: String,
    pub phase: AgentPhase,
    pub steps: Vec<AgentStep>,
    /// Model conversation (system + goal + step observations).
    pub chat: Vec<crate::ai::ChatMessage>,
    pub rx: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    pub pending_confirm: Option<String>,
    /// Set by the global stop shortcut / stop button; consumed by tick.
    pub request_stop: bool,
    pub wait: Option<AgentWait>,
    /// Consecutive parse failures (fallback to manual input after 2).
    pub parse_failures: u32,
    /// Screen snapshot observed after the LAST command (fed to the model).
    pub last_observation: String,
}

/// Completion-wait parameters.
const SCREEN_STABLE_MS: u128 = 600;
const STEP_TIMEOUT_MS: u128 = 30_000;
/// Screen tail sent to the model after each command.
const OBSERVE_CHARS: usize = 2000;

fn hash_text(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

/// Ctrl+Shift+. stops the running agent. Consumed BEFORE the modal
/// arbiter so it works even while a confirm dialog is open.
pub(crate) fn agent_stop_shortcut_hit(
    ctx: &egui::Context,
    binds: &HashMap<String, ShortcutBinding>,
) -> bool {
    let Some(b) = binds.get("stop_agent") else {
        return false;
    };
    let Some(key) = binding_to_key(b) else {
        return false;
    };
    ctx.input_mut(|i| {
        i.consume_key(
            egui::Modifiers {
                ctrl: b.ctrl,
                shift: b.shift,
                alt: b.alt,
                ..Default::default()
            },
            key,
        )
    })
}

impl App {
    /// Effective approval mode for the current agent (PROD hosts are
    /// always downgraded to Manual).
    pub(crate) fn agent_effective_mode(&self) -> ApprovalMode {
        let mode = ApprovalMode::from_str(&self.settings.agent_approval_mode);
        let prod = self
            .agent
            .as_ref()
            .and_then(|a| self.terminals.get(&a.tab))
            .and_then(|td| td.host.as_ref())
            .is_some_and(|h| h.prod);
        if prod {
            ApprovalMode::Manual
        } else {
            mode
        }
    }

    /// The state machine tick: drains the model reply, advances the gate,
    /// and watches command completion. Runs every frame at the top of
    /// `update`; waiting phases schedule their own repaints.
    pub(crate) fn agent_tick(&mut self, ctx: &egui::Context) {
        // Read the phase, then drop the borrow: the transition methods
        // take &mut self and cannot run while `agent` is borrowed.
        let phase = self.agent.as_ref().map(|a| a.phase.clone());
        let Some(phase) = phase else {
            return;
        };
        {
            let Some(agent) = self.agent.as_mut() else {
                return;
            };
            // Stop request wins over everything.
            if agent.request_stop {
                agent.phase = AgentPhase::Failed("stopped by user".into());
                agent.request_stop = false;
                agent.rx = None;
                agent.pending_confirm = None;
            }
        }
        match phase {
            AgentPhase::Thinking => {
                let mut reply = None;
                let mut err = None;
                if let Some(agent) = self.agent.as_mut() {
                    if let Some(rx) = &agent.rx {
                        if let Ok(outcome) = rx.try_recv() {
                            agent.rx = None;
                            match outcome {
                                Ok(r) => reply = Some(r),
                                Err(e) => err = Some(e),
                            }
                        }
                    }
                }
                if let Some(e) = err {
                    if let Some(agent) = self.agent.as_mut() {
                        agent.phase = AgentPhase::Failed(e);
                    }
                } else if let Some(r) = reply {
                    self.agent_handle_reply(ctx, r);
                }
            }
            AgentPhase::Executing => {
                let done = self.agent_check_completion();
                if done {
                    // Observe, feed back to the model, keep looping.
                    self.agent_observe_and_continue(ctx);
                }
            }
            AgentPhase::TimedOut => {
                // The panel shows continue/stop buttons; the tick is
                // passive here.
            }
            AgentPhase::WaitingConfirm
            | AgentPhase::Done
            | AgentPhase::NeedInput(_)
            | AgentPhase::Failed(_) => {}
        }
        // Schedule a wake-up while waiting, so idle screens still tick.
        if matches!(
            self.agent.as_ref().map(|a| &a.phase),
            Some(AgentPhase::Executing | AgentPhase::Thinking)
        ) {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
    }

    /// The model replied: parse the action, run the gate, transition.
    fn agent_handle_reply(&mut self, ctx: &egui::Context, reply: String) {
        let Some(action) = crate::app::agent::parse_action(&reply) else {
            let Some(agent) = self.agent.as_mut() else {
                return;
            };
            agent.parse_failures += 1;
            if agent.parse_failures >= 2 {
                agent.phase = AgentPhase::NeedInput(
                    "the model kept replying in a format I could not parse".into(),
                );
                agent.rx = None;
            } else {
                // Nudge the model: append a corrective user turn.
                agent.chat.push(crate::ai::ChatMessage {
                    role: "user",
                    content: "Your last reply was not valid JSON with an action field. \
Respond with ONLY the JSON object as instructed."
                        .into(),
                });
                agent.phase = AgentPhase::Thinking;
                self.agent_spawn_model_call();
            }
            return;
        };
        let max_steps = self.settings.agent_max_steps;
        let mode = self.agent_effective_mode();
        let mut do_write = false;
        let mut do_respawn = false;
        {
            let Some(agent) = self.agent.as_mut() else {
                return;
            };
            agent.parse_failures = 0;
            match action.action.as_str() {
                "done" => {
                    agent.phase = AgentPhase::Done;
                }
                "say" => {
                    agent.phase = AgentPhase::NeedInput(action.reason.clone());
                }
                "run" => {
                    let command = action.command.trim().to_string();
                    if command.is_empty() {
                        agent.phase =
                            AgentPhase::NeedInput("the model sent an empty command".into());
                        return;
                    }
                    if agent.steps.len() >= max_steps {
                        agent.phase =
                            AgentPhase::Failed(format!("step limit reached ({max_steps} steps)"));
                        return;
                    }
                    let class = crate::app::agent::classify_command(&command);
                    let prod = self
                        .terminals
                        .get(&agent.tab)
                        .and_then(|td| td.host.as_ref())
                        .is_some_and(|h| h.prod);
                    let decision = crate::app::agent::gate(mode, class, prod);
                    agent.steps.push(AgentStep {
                        command: command.clone(),
                        reason: action.reason.clone(),
                        status: StepStatus::Running,
                        outcome: String::new(),
                    });
                    match decision {
                        GateDecision::Auto => {
                            agent.phase = AgentPhase::Executing;
                            do_write = true;
                        }
                        GateDecision::Confirm => {
                            agent.pending_confirm = Some(command);
                            agent.phase = AgentPhase::WaitingConfirm;
                        }
                        GateDecision::Deny => {
                            // Tell the model the command was refused.
                            agent.chat.push(crate::ai::ChatMessage {
                                role: "user",
                                content: format!(
                                    "The safety gate DENIED your command: `{command}`. \
Try a read-only alternative."
                                ),
                            });
                            agent.steps.last_mut().unwrap().status = StepStatus::Denied;
                            agent.phase = AgentPhase::Thinking;
                            do_respawn = true;
                        }
                    }
                }
                _ => unreachable!("parse_action filters action strings"),
            }
        }
        if do_write {
            self.agent_write_and_execute();
        }
        if do_respawn {
            self.agent_spawn_model_call();
        }
        ctx.request_repaint();
    }

    /// Write the command into the agent's terminal and enter Executing.
    fn agent_write_and_execute(&mut self) {
        let Some(agent) = self.agent.as_mut() else {
            return;
        };
        let tab = agent.tab.clone();
        let Some(command) = agent.steps.last().map(|s| s.command.clone()) else {
            return;
        };
        let has_signal = self
            .terminals
            .get(&tab)
            .map(|td| td.instance.prompt_seq > 0)
            .unwrap_or(false);
        let prompt_snapshot = self
            .terminals
            .get(&tab)
            .map(|td| td.instance.prompt_seq)
            .unwrap_or(0);
        if let Some(td) = self.terminals.get_mut(&tab) {
            td.instance.write(command.as_bytes());
            td.instance.write(b"\r");
        }
        agent.wait = Some(AgentWait {
            started: std::time::Instant::now(),
            prompt_snapshot,
            has_prompt_signal: has_signal,
            screen_hash: 0,
            stable_since: None,
        });
        agent.phase = AgentPhase::Executing;
    }

    /// Completion check for the Executing phase: prompt_seq growth when
    /// the shell integration is live, otherwise screen stability; both
    /// bounded by the step timeout.
    fn agent_check_completion(&mut self) -> bool {
        let Some(agent) = self.agent.as_mut() else {
            return false;
        };
        let tab = agent.tab.clone();
        let Some(wait) = agent.wait.as_mut() else {
            return false;
        };
        let Some(td) = self.terminals.get_mut(&tab) else {
            return false;
        };
        let signal = crate::app::agent::wait_signal(
            if wait.has_prompt_signal {
                td.instance.prompt_seq
            } else {
                0
            },
            wait.started.elapsed().as_millis(),
            STEP_TIMEOUT_MS,
        );
        match signal {
            WaitSignal::PromptSeq => {
                td.instance.poll_cwd();
                td.instance.prompt_seq > wait.prompt_snapshot
            }
            WaitSignal::ScreenStable => {
                let screen = td.instance.visible_text_refreshed(OBSERVE_CHARS);
                let hash = hash_text(&screen);
                if hash != wait.screen_hash {
                    wait.screen_hash = hash;
                    wait.stable_since = None;
                    false
                } else if wait.stable_since.is_none() {
                    wait.stable_since = Some(std::time::Instant::now());
                    false
                } else {
                    wait.stable_since.unwrap().elapsed().as_millis() >= SCREEN_STABLE_MS
                }
            }
            WaitSignal::Timeout => {
                agent.wait = None;
                agent.phase = AgentPhase::TimedOut;
                false
            }
        }
    }

    /// Command finished: snapshot the screen as the observation, feed it
    /// back to the model, and either continue or stop.
    fn agent_observe_and_continue(&mut self, ctx: &egui::Context) {
        let Some(agent) = self.agent.as_mut() else {
            return;
        };
        let tab = agent.tab.clone();
        let screen = self
            .terminals
            .get_mut(&tab)
            .map(|td| td.instance.visible_text_refreshed(OBSERVE_CHARS))
            .unwrap_or_default();
        // Skip re-observing an unchanged screen (the prompt line only).
        let changed = screen != agent.last_observation;
        agent.last_observation = screen.clone();
        let last_step = agent.steps.last_mut().unwrap();
        last_step.status = StepStatus::Ok;
        last_step.outcome = screen.lines().next().unwrap_or("").to_string();
        if changed {
            agent.chat.push(crate::ai::ChatMessage {
                role: "user",
                content: format!(
                    "Command finished. The screen now shows:\n```\n{screen}\n```\n\
Continue toward the goal, or send {{\"action\":\"done\"}} when it is achieved."
                ),
            });
        } else {
            agent.chat.push(crate::ai::ChatMessage {
                role: "user",
                content: "Command finished (screen unchanged). Continue toward the goal, or \
send {\"action\":\"done\"} when it is achieved."
                    .into(),
            });
        }
        agent.wait = None;
        agent.phase = AgentPhase::Thinking;
        self.agent_spawn_model_call();
        ctx.request_repaint();
    }

    /// Fire the next model call for the agent (independent rx/busy).
    fn agent_spawn_model_call(&mut self) {
        let Some(agent) = self.agent.as_mut() else {
            return;
        };
        let cfg = crate::ai::AiConfig {
            base_url: self.settings.ai_base_url.clone(),
            api_key: self.settings.ai_api_key.clone(),
            model: self.settings.ai_model.clone(),
        };
        let messages = agent.chat.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        agent.rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(crate::ai::complete_messages(&cfg, &messages, 60));
        });
    }

    /// Start an agent run for `goal` on the FOCUSED terminal.
    pub(crate) fn agent_start(&mut self, ctx: &egui::Context, goal: String) {
        let goal = goal.trim().to_string();
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };
        if goal.is_empty() || self.agent.is_some() {
            return;
        }
        let mut chat = vec![crate::ai::ChatMessage {
            role: "system",
            content: crate::app::agent::AGENT_SYSTEM.to_string(),
        }];
        let prod = self
            .terminals
            .get(&tab)
            .and_then(|td| td.host.as_ref())
            .is_some_and(|h| h.prod);
        chat.push(crate::ai::ChatMessage {
            role: "user",
            content: if prod {
                format!(
                    "GOAL: {goal}\n\nThis is a PRODUCTION host. Prefer read-only commands and \
ask before anything destructive."
                )
            } else {
                format!("GOAL: {goal}")
            },
        });
        self.agent = Some(AgentRun {
            goal,
            tab,
            phase: AgentPhase::Thinking,
            steps: Vec::new(),
            chat,
            rx: None,
            pending_confirm: None,
            request_stop: false,
            wait: None,
            parse_failures: 0,
            last_observation: String::new(),
        });
        self.agent_spawn_model_call();
        ctx.request_repaint();
    }

    /// The user picked "continue" after a step timeout: re-observe and
    /// keep looping (the command may still be running; the screen
    /// snapshot makes that visible to the model).
    pub(crate) fn agent_continue_after_timeout(&mut self, ctx: &egui::Context) {
        let Some(agent) = self.agent.as_mut() else {
            return;
        };
        agent.phase = AgentPhase::Executing;
        agent.wait = Some(AgentWait {
            started: std::time::Instant::now(),
            prompt_snapshot: 0,
            has_prompt_signal: false,
            screen_hash: 0,
            stable_since: None,
        });
        ctx.request_repaint();
    }

    /// Full stop (button or shortcut): clear the run.
    pub(crate) fn agent_stop(&mut self) {
        if let Some(agent) = self.agent.as_mut() {
            agent.request_stop = true;
        }
    }

    /// Confirm dialog for a pending command (dialog_keys protocol).
    pub(crate) fn render_agent_confirm(&mut self, ctx: &egui::Context) {
        let Some(agent) = self.agent.as_ref() else {
            return;
        };
        let Some(command) = agent.pending_confirm.clone() else {
            return;
        };
        // Safe-side rising edge (a stray Enter must not run anything).
        if std::mem::take(&mut self.agent_confirm_just_opened) {
            self.dialog_kb_confirm = false;
        }
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirmed = keys.confirm;
        let mut cancelled = keys.cancel;
        let mut open = true;
        let preview: String = command.chars().take(300).collect();
        let body = self.texts.ai.agent_confirm_body.replace("{}", &preview);
        let mut kb = self.dialog_kb_confirm;
        let inner = egui::Window::new(&self.texts.ai.agent_confirm_title)
            .id(egui::Id::new("agent_confirm_window"))
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_pos(screen_center(ctx))
            .pivot(egui::Align2::CENTER_CENTER)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new(&body).color(self.active_theme.app.warning.to_egui()));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    Self::dialog_button_row(
                        ui,
                        &mut kb,
                        egui::Id::new("agent_confirm_yes"),
                        egui::Id::new("agent_confirm_no"),
                        &self.texts.ai.agent_confirm_run,
                        &self.texts.ai.agent_confirm_cancel,
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
            let Some(agent) = self.agent.as_mut() else {
                return;
            };
            agent.pending_confirm = None;
            agent.phase = AgentPhase::Executing;
            self.agent_write_and_execute();
        }
        if cancelled || !open {
            // Cancelled: tell the model the user declined this step.
            let Some(agent) = self.agent.as_mut() else {
                return;
            };
            agent.pending_confirm = None;
            agent.chat.push(crate::ai::ChatMessage {
                role: "user",
                content: format!(
                    "The user DECLINED your command `{command}`. Propose a different, \
safer approach."
                ),
            });
            if let Some(step) = agent.steps.last_mut() {
                step.status = StepStatus::Cancelled;
            }
            agent.phase = AgentPhase::Thinking;
            self.agent_spawn_model_call();
        }
    }

    /// The agent section of the AI panel.
    pub(crate) fn render_agent_section(&mut self, ui: &mut egui::Ui) {
        let t = self.texts.ai.clone();
        let weak = self.active_theme.app.weak_text.to_egui();
        let accent = self.active_theme.app.accent.to_egui();
        ui.separator();
        ui.label(egui::RichText::new(&t.agent_section).size(10.0).color(weak));
        match self.agent.as_mut() {
            None => {
                // Goal input + approval mode + start.
                ui.add(
                    egui::TextEdit::multiline(&mut self.agent_goal)
                        .hint_text(&t.agent_goal_hint)
                        .desired_rows(2)
                        .desired_width(ui.available_width()),
                );
                ui.horizontal(|ui| {
                    ui.label(&t.agent_approval);
                    let mut mode = ApprovalMode::from_str(&self.settings.agent_approval_mode);
                    egui::ComboBox::from_id_salt("agent_approval")
                        .selected_text(self.agent_mode_label(mode))
                        .show_ui(ui, |ui| {
                            for m in [
                                ApprovalMode::Allowlist,
                                ApprovalMode::Manual,
                                ApprovalMode::FullAuto,
                            ] {
                                ui.selectable_value(&mut mode, m, self.agent_mode_label(m));
                            }
                        });
                    if ApprovalMode::from_str(&self.settings.agent_approval_mode) != mode {
                        self.settings.agent_approval_mode = mode.as_str().to_string();
                    }
                });
                let can_start = self.settings.ai_enabled && !self.agent_goal.trim().is_empty();
                if ui
                    .add_enabled(can_start, egui::Button::new(&t.agent_start))
                    .clicked()
                {
                    let goal = self.agent_goal.clone();
                    self.agent_start(ui.ctx(), goal);
                }
            }
            Some(agent) => {
                // Running/finished: goal + phase + transcript + controls.
                let goal = agent.goal.clone();
                let tab = agent.tab.clone();
                let phase = agent.phase.clone();
                let steps = agent.steps.clone();
                ui.label(egui::RichText::new(&goal).size(11.0).color(accent));
                let status = match &phase {
                    AgentPhase::Thinking => t.agent_phase_thinking.clone(),
                    AgentPhase::WaitingConfirm => t.agent_phase_waiting.clone(),
                    AgentPhase::Executing => t.agent_phase_executing.clone(),
                    AgentPhase::TimedOut => t.agent_phase_timed_out.clone(),
                    AgentPhase::NeedInput(r) => format!("{}: {r}", t.agent_phase_need_input),
                    AgentPhase::Done => t.agent_phase_done.clone(),
                    AgentPhase::Failed(e) => format!("{}: {e}", t.agent_phase_failed),
                };
                ui.label(egui::RichText::new(status).size(11.0).color(
                    if matches!(phase, AgentPhase::Failed(_)) {
                        self.active_theme.app.danger.to_egui()
                    } else {
                        weak
                    },
                ));
                egui::ScrollArea::vertical()
                    .max_height(140.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for step in &steps {
                            let mark = match step.status {
                                StepStatus::Running => "…",
                                StepStatus::Ok => "✓",
                                StepStatus::Denied => "✕",
                                StepStatus::Cancelled => "↺",
                            };
                            ui.label(
                                egui::RichText::new(format!("{mark} {}", step.command))
                                    .size(11.0)
                                    .monospace(),
                            );
                            if !step.outcome.is_empty() {
                                let tail = if step.reason.is_empty() {
                                    step.outcome.clone()
                                } else {
                                    format!("{}  [{}]", step.outcome, step.reason)
                                };
                                ui.label(
                                    egui::RichText::new(format!("   {tail}"))
                                        .size(10.0)
                                        .color(weak),
                                );
                            }
                        }
                    });
                ui.horizontal(|ui| {
                    if matches!(phase, AgentPhase::TimedOut)
                        && ui.button(&t.agent_continue).clicked()
                    {
                        self.agent_continue_after_timeout(ui.ctx());
                    }
                    let finished = matches!(
                        phase,
                        AgentPhase::Done | AgentPhase::Failed(_) | AgentPhase::NeedInput(_)
                    );
                    let label = if finished {
                        &t.agent_close
                    } else {
                        &t.agent_stop
                    };
                    if ui.button(label).clicked() {
                        if finished {
                            self.agent = None;
                        } else {
                            self.agent_stop();
                        }
                    }
                });
                let _ = tab;
            }
        }
    }

    fn agent_mode_label(&self, mode: ApprovalMode) -> String {
        match mode {
            ApprovalMode::Manual => self.texts.ai.agent_manual.clone(),
            ApprovalMode::Allowlist => self.texts.ai.agent_allowlist.clone(),
            ApprovalMode::FullAuto => self.texts.ai.agent_fullauto.clone(),
        }
    }
}

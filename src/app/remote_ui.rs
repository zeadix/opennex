//! Remote phone control: the App-side bridge (snapshot refresh, command
//! execution, server lifecycle) and the desktop panel with the QR code.

use super::*;
use crate::remote::protocol::{
    lan_ip, remote_url, RemoteCommand, RemoteSnapshot, TermInfo, WsInfo,
};
use crate::remote::server::{RemoteServer, RemoteShared};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A live remote session (server + shared state + per-terminal frame
/// sequence numbers).
pub(crate) struct RemoteSession {
    pub server: Option<RemoteServer>,
    pub shared: RemoteShared,
    /// LAN address the phone connects to ("" = unresolved).
    pub ip: String,
    pub last_refresh: Instant,
    pub frame_seq: HashMap<String, u64>,
    /// Last serialized ANSI per tab - the change detector.
    pub last_ansi: HashMap<String, String>,
    pub panel_visible: bool,
}

/// A WS "frame" message: the FrameMsg JSON plus the tab id.
fn frame_json(tab: &str, frame: &crate::remote::ansi::FrameMsg) -> String {
    let body = serde_json::to_string(frame).unwrap_or_else(|_| "{}".into());
    // Splice the tab id in front of the closing brace.
    let mut out = body;
    if let Some(pos) = out.rfind('}') {
        out.insert_str(
            pos,
            &format!(
                ",\"tab\":{}",
                serde_json::to_string(tab).unwrap_or_default()
            ),
        );
    }
    format!("{{\"t\":\"frame\",{}", &out[1..])
}

impl RemoteSession {
    pub fn url(&self) -> String {
        remote_url(&self.ip, self.shared.port, &self.shared.token)
    }
}

impl App {
    /// Start the remote server (new session token each time). Returns an
    /// error string for the toast path on bind failure.
    pub(crate) fn remote_start(&mut self) -> Result<(), String> {
        if self.remote.is_some() {
            return Ok(());
        }
        let token = uuid::Uuid::new_v4().simple().to_string();
        let port = self.settings.remote_port;
        let (shared, server) = RemoteServer::start(port, token)?;
        self.remote = Some(RemoteSession {
            server: Some(server),
            shared,
            ip: lan_ip().unwrap_or_default(),
            last_refresh: Instant::now() - Duration::from_secs(1),
            frame_seq: HashMap::new(),
            last_ansi: HashMap::new(),
            panel_visible: true,
        });
        Ok(())
    }

    pub(crate) fn remote_stop(&mut self) {
        if let Some(mut session) = self.remote.take() {
            if let Some(server) = session.server.take() {
                server.stop();
            }
        }
    }

    /// Per-frame tick: drain phone commands (terminal writes and focus
    /// changes MUST run on the UI thread) and refresh the shared snapshot
    /// + frames on a ~200ms cadence.
    pub(crate) fn remote_tick(&mut self, ctx: &egui::Context) {
        let Some(session) = self.remote.as_mut() else {
            return;
        };
        // 1. Drain queued commands with only the queue borrow live.
        let commands: Vec<RemoteCommand> = {
            let mut queue = session.shared.commands.lock().unwrap();
            queue.drain(..).collect()
        };
        // 2. Throttle decision: ~40ms cadence while a session runs
        // (real-time for TUI redraws; PTY output drives repaints).
        let refresh_due = session.last_refresh.elapsed() >= Duration::from_millis(40);
        if refresh_due {
            session.last_refresh = Instant::now();
        }
        // 3. Execute commands and refresh (no `session` borrow live).
        for command in commands {
            match command {
                RemoteCommand::Focus { tab } => self.remote_focus(&tab),
                RemoteCommand::Write { tab, data } => {
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        td.instance.write(data.as_bytes());
                    }
                }
                RemoteCommand::Unlock {
                    panel,
                    password,
                    reply,
                } => {
                    let ok = verify_lock_password(&password, &self.settings.lock_password)
                        && panel < self.panels.len();
                    if ok {
                        self.locked_panels.remove(&panel);
                    }
                    let _ = reply.send(ok);
                }
                RemoteCommand::RequestScrollback { tab, reply } => {
                    let ansi = self
                        .terminals
                        .get_mut(&tab)
                        .map(|td| {
                            let content = td.instance.backend.sync();
                            crate::remote::ansi::serialize_scrollback(
                                &content.grid,
                                &self.terminal_theme_cache,
                                500,
                                256 * 1024,
                            )
                        })
                        .unwrap_or_default();
                    let _ = reply.send(ansi);
                }
            }
        }
        if refresh_due {
            self.remote_refresh();
        }
        // Keep the frame cache fresh even when nothing repaints locally.
        ctx.request_repaint_after(Duration::from_millis(250));
    }

    /// Rebuild the shared snapshot and every terminal's frame.
    fn remote_refresh(&mut self) {
        // --- snapshot (workspaces + terminals, locked ones redacted) ---
        let mut snapshot = RemoteSnapshot::default();
        for (i, panel) in self.panels.iter().enumerate() {
            let locked = self.locked_panels.contains(&i);
            let mut ws = WsInfo {
                name: panel.name.clone(),
                locked,
                terminals: Vec::new(),
            };
            if !locked {
                if let Some(tree) = self.dock_states.get(&i) {
                    for (_, tab_id) in tree.iter_all_tabs() {
                        if let Some(td) = self.terminals.get(tab_id) {
                            let (cols, rows) = td.instance.size();
                            ws.terminals.push(TermInfo {
                                id: tab_id.clone(),
                                name: td.name.clone(),
                                host: td.host.as_ref().map(|h| h.addr.clone()).unwrap_or_default(),
                                cwd: td.instance.cwd.clone(),
                                cols,
                                rows,
                            });
                        }
                    }
                }
            }
            snapshot.workspaces.push(ws);
        }
        snapshot.focused = self.focused_terminal.clone();
        if let Some(session) = self.remote.as_ref() {
            *session.shared.snapshot.write().unwrap() = snapshot;
        } else {
            return;
        }

        // --- frames (one per live terminal; unchanged content keeps its
        // seq, changed content bumps seq and broadcasts to WS writers) ---
        let theme = self.terminal_theme_cache.clone();
        let ids: Vec<String> = self.terminals.keys().cloned().collect();
        let mut changed: Vec<(String, crate::remote::ansi::FrameMsg)> = Vec::new();
        let session = self.remote.as_mut().unwrap();
        let mut new_frames = HashMap::new();
        for id in ids {
            if let Some(td) = self.terminals.get_mut(&id) {
                let content = td.instance.backend.sync();
                let seq = session.frame_seq.entry(id.clone()).or_insert(0);
                let frame = crate::remote::ansi::serialize_frame(
                    &content.grid,
                    &theme,
                    &content.terminal_mode,
                    *seq,
                );
                let mut frame = frame;
                let prev = session.last_ansi.get(&id);
                if prev != Some(&frame.d) {
                    *seq = seq.wrapping_add(1);
                    frame.seq = *seq;
                    session.last_ansi.insert(id.clone(), frame.d.clone());
                    changed.push((id.clone(), frame.clone()));
                }
                new_frames.insert(id, frame);
            }
        }
        *session.shared.frames.write().unwrap() = new_frames;
        // Broadcast changed frames; prune dead subscribers.
        if !changed.is_empty() {
            let mut subs = session.shared.subscribers.lock().unwrap();
            subs.retain(|tx| {
                changed.iter().all(|(id, frame)| {
                    let payload = frame_json(id, frame);
                    tx.send(crate::remote::ws::WsOut::Text(payload)).is_ok()
                })
            });
        }
    }

    /// Phone asked to focus a terminal: switch workspace and select the
    /// tab (mirrors the monitor panel's row-click behavior, plus the dock
    /// focus so the NEXT keystroke lands there).
    fn remote_focus(&mut self, tab: &str) {
        let tab = tab.to_string();
        for (idx, tree) in self.dock_states.iter() {
            if tree.find_tab(&tab).is_some() {
                self.active_panel = *idx;
                break;
            }
        }
        self.focused_terminal = Some(tab.to_string());
        if let Some(tree) = self.dock_states.get_mut(&self.active_panel) {
            if let Some(loc) = tree.find_tab(&tab) {
                tree.set_active_tab(loc);
                if let Some((surface, node, _)) = tree.find_tab(&tab) {
                    tree.set_focused_node_and_surface((surface, node));
                }
            }
        }
    }

    /// The remote-control panel: toggle, URL, QR code, status.
    pub(crate) fn render_remote_panel(&mut self, ctx: &egui::Context) {
        let visible = self
            .remote
            .as_ref()
            .map(|s| s.panel_visible)
            .unwrap_or(false)
            || self.show_remote_panel;
        if !visible {
            return;
        }
        let mut open = true;
        egui::Window::new(&self.texts.remote.panel_title)
            .id(egui::Id::new("remote_panel"))
            .open(&mut open)
            .default_width(320.0)
            .default_pos(screen_center(ctx) + egui::vec2(60.0, 40.0))
            .show(ctx, |ui| {
                let t = self.texts.remote.clone();
                let weak = self.active_theme.app.weak_text.to_egui();
                let accent = self.active_theme.app.accent.to_egui();
                match self.remote.as_ref() {
                    None => {
                        ui.label(egui::RichText::new(&t.off_hint).size(11.0).color(weak));
                        if ui.button(&t.start).clicked() {
                            match self.remote_start() {
                                Ok(()) => {}
                                Err(err) => {
                                    self.update_toast = Some((
                                        format!("{}: {err}", t.bind_failed),
                                        Instant::now() + Duration::from_secs(8),
                                    ));
                                }
                            }
                        }
                    }
                    Some(_) => {
                        let url = self.remote.as_ref().unwrap().url();
                        ui.label(egui::RichText::new(&t.on_hint).size(11.0).color(weak));
                        ui.label(egui::RichText::new(&url).size(11.0).color(accent));
                        ui.horizontal(|ui| {
                            if ui.button(&t.copy_url).clicked() {
                                ui.ctx().copy_text(url.clone());
                                self.update_toast = Some((
                                    t.copied.clone(),
                                    Instant::now() + Duration::from_secs(4),
                                ));
                            }
                            if ui.button(&t.stop).clicked() {
                                self.remote_stop();
                            }
                        });
                        ui.add_space(6.0);
                        self.remote_draw_qr(ui, &url);
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&t.security_hint).size(10.0).color(weak));
                    }
                }
            });
        // Persist the panel visibility choice.
        let still_open = open;
        if let Some(session) = self.remote.as_mut() {
            session.panel_visible = still_open;
        }
        if !still_open && self.remote.is_none() {
            self.show_remote_panel = false;
        }
    }

    /// Draw the QR matrix with plain rects (no image dependency).
    fn remote_draw_qr(&mut self, ui: &mut egui::Ui, url: &str) {
        let Ok(code) = qrcode::QrCode::new(url.as_bytes()) else {
            ui.label("QR error");
            return;
        };
        let modules = code.width() as f32;
        let quiet = 2.0;
        let total = modules + quiet * 2.0;
        let size = 220.0;
        let cell = size / total;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
        let origin = rect.min + egui::vec2(quiet * cell, quiet * cell);
        let dark = ui.visuals().strong_text_color();
        let light = ui.visuals().extreme_bg_color;
        ui.painter().rect_filled(rect, 4.0, light);
        // qrcode 0.14: module matrix via to_colors (row-major).
        let colors: Vec<qrcode::Color> = code.to_colors();
        for (i, color) in colors.iter().enumerate() {
            let (row, col) = (i / code.width(), i % code.width());
            let fill = match color {
                qrcode::Color::Dark => dark,
                qrcode::Color::Light => light,
            };
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(origin.x + col as f32 * cell, origin.y + row as f32 * cell),
                egui::vec2(cell, cell),
            );
            ui.painter().rect_filled(cell_rect, 0.0, fill);
        }
    }
}

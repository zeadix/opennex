//! Remote phone control: the App-side bridge (snapshot refresh, command
//! execution, server lifecycle) and the desktop panel with the QR code.

use super::*;
use crate::remote::protocol::{
    lan_ip, remote_url, RemoteCommand, RemoteSnapshot, TermInfo, WsInfo,
};
use crate::remote::server::{RemoteServer, RemoteShared};
use crate::remote::tunnel::{TunnelEvent, TunnelHandle};
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
    /// Public IPv6 for direct cellular access (None = no global route).
    pub ipv6: Option<String>,
    /// Managed cloudflared quick tunnel (None = not started).
    pub tunnel: Option<TunnelHandle>,
    /// Handle-spawn result channel (drained by remote_tick).
    pub tunnel_joiner: Option<std::sync::mpsc::Receiver<Result<TunnelHandle, String>>>,
    /// Latest tunnel status surfaced to the panel.
    pub tunnel_url: Option<String>,
    pub tunnel_error: Option<String>,
    pub tunnel_progress: Option<f32>,
    pub tunnel_starting: bool,
    /// Which address the QR shows.
    pub qr_target: QrTarget,
    /// The tab the phone is currently watching (drives on-demand frame
    /// serialization; None = nobody focused yet).
    pub remote_focus_tab: Option<String>,
    /// QR cache: (url, dark-module matrix) - regenerating the QR per
    /// frame cost ~1.7k painter shapes at 60fps for a static image.
    pub qr_cache: Option<(String, Vec<bool>)>,
    /// Module count per side of the cached QR.
    pub qr_width: usize,
}

/// Which entry URL the QR code shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum QrTarget {
    #[default]
    Lan,
    Ipv6,
    Tunnel,
}

/// Paint the QR module matrix with quiet zone.
fn draw_qr_modules(ui: &mut egui::Ui, modules: &[bool], width: usize) {
    if width == 0 || modules.len() != width * width {
        return;
    }
    let quiet = 2.0;
    let total = width as f32 + quiet * 2.0;
    let size = 220.0;
    let cell = size / total;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let origin = rect.min + egui::vec2(quiet * cell, quiet * cell);
    let dark = ui.visuals().strong_text_color();
    let light = ui.visuals().extreme_bg_color;
    ui.painter().rect_filled(rect, 4.0, light);
    for row in 0..width {
        for col in 0..width {
            if modules[row * width + col] {
                let cell_rect = egui::Rect::from_min_size(
                    egui::pos2(origin.x + col as f32 * cell, origin.y + row as f32 * cell),
                    egui::vec2(cell, cell),
                );
                ui.painter().rect_filled(cell_rect, 0.0, dark);
            }
        }
    }
}

impl RemoteSession {
    pub fn url(&self) -> String {
        remote_url(&self.ip, self.shared.port, &self.shared.token)
    }

    /// IPv6 direct URL (needs the port; bracketed literal).
    pub fn ipv6_url(&self) -> Option<String> {
        self.ipv6
            .as_ref()
            .map(|ip| remote_url(ip, self.shared.port, &self.shared.token))
    }

    /// The URL the QR should show for the current target.
    pub fn qr_url(&self) -> Option<String> {
        match self.qr_target {
            QrTarget::Lan if !self.ip.is_empty() => Some(self.url()),
            QrTarget::Lan => None,
            QrTarget::Ipv6 => self.ipv6_url(),
            QrTarget::Tunnel => self.tunnel_url.clone(),
        }
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
            ipv6: crate::remote::protocol::public_ipv6(),
            tunnel: None,
            tunnel_joiner: None,
            tunnel_url: None,
            tunnel_error: None,
            tunnel_progress: None,
            tunnel_starting: false,
            qr_target: QrTarget::default(),
            remote_focus_tab: None,
            qr_cache: None,
            qr_width: 0,
        });
        Ok(())
    }

    pub(crate) fn remote_stop(&mut self) {
        if let Some(mut session) = self.remote.take() {
            if let Some(server) = session.server.take() {
                server.stop();
            }
            if let Some(mut tunnel) = session.tunnel.take() {
                tunnel.stop();
            }
        }
    }

    /// Start (or download + start) the cloudflared quick tunnel. The
    /// spawn happens on a background thread; the result handle flows
    /// back through a channel drained by remote_tick.
    pub(crate) fn remote_tunnel_start(&mut self) {
        let Some(session) = self.remote.as_mut() else {
            return;
        };
        if session.tunnel.is_some() || session.tunnel_starting {
            return;
        }
        session.tunnel_starting = true;
        session.tunnel_error = None;
        let data_dir = app_data_dir();
        let port = session.shared.port;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(TunnelHandle::start(&data_dir, port));
        });
        session.tunnel_joiner = Some(rx);
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
        // 2. Throttle decision: ~40ms only while a phone is actually
        // subscribed (real-time for TUI redraws); with nobody watching,
        // 500ms snapshot refresh keeps the server responsive without
        // burning CPU on serialization nobody reads.
        let has_subs = !session.shared.subscribers.lock().unwrap().is_empty();
        let cadence = if has_subs { 40 } else { 500 };
        let refresh_due = session.last_refresh.elapsed() >= Duration::from_millis(cadence);
        if refresh_due {
            session.last_refresh = Instant::now();
        }
        // 3. Execute commands and refresh (no `session` borrow live).
        for command in commands {
            match command {
                RemoteCommand::Focus { tab } => {
                    // Track which tab the phone watches (drives on-demand
                    // serialization) before focusing it.
                    if let Some(session) = self.remote.as_mut() {
                        session.remote_focus_tab = Some(tab.clone());
                    }
                    self.remote_focus(&tab);
                }
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
                RemoteCommand::ScrollBottom { tab } => {
                    // Large negative delta scrolls the shared viewport to
                    // the live prompt (desktop and phone see the same
                    // grid - the phone's ⤓ must move BOTH back).
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        td.instance
                            .backend
                            .process_command(egui_term::BackendCommand::Scroll(-1_000_000));
                        td.instance.backend.set_dirty();
                    }
                }
                RemoteCommand::Mouse {
                    tab,
                    btn,
                    col,
                    row,
                    pressed,
                } => {
                    // Forward a phone touch to the TUI application as a
                    // real mouse event (grid coordinates).
                    if let Some(td) = self.terminals.get_mut(&tab) {
                        let button = match btn {
                            0 => crate::remote::remote_mouse_button(0),
                            1 => crate::remote::remote_mouse_button(1),
                            2 => crate::remote::remote_mouse_button(2),
                            32 => crate::remote::remote_mouse_button(32),
                            33 => crate::remote::remote_mouse_button(33),
                            34 => crate::remote::remote_mouse_button(34),
                            35 => crate::remote::remote_mouse_button(35),
                            64 => crate::remote::remote_mouse_button(64),
                            65 => crate::remote::remote_mouse_button(65),
                            _ => crate::remote::remote_mouse_button(99),
                        };
                        let point = alacritty_terminal::index::Point::new(
                            alacritty_terminal::index::Line(row as i32),
                            alacritty_terminal::index::Column(col as usize),
                        );
                        td.instance.backend.process_command(
                            egui_term::BackendCommand::MouseReport(
                                button,
                                egui::Modifiers::NONE,
                                point,
                                pressed,
                            ),
                        );
                    }
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
        // Tunnel plumbing: adopt the spawned handle, surface its events.
        if let Some(session) = self.remote.as_mut() {
            if let Some(joiner) = session.tunnel_joiner.take() {
                match joiner.try_recv() {
                    Ok(Ok(handle)) => {
                        session.tunnel = Some(handle);
                        session.tunnel_starting = false;
                    }
                    Ok(Err(err)) => {
                        session.tunnel_starting = false;
                        session.tunnel_error = Some(err);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        session.tunnel_joiner = Some(joiner);
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        session.tunnel_starting = false;
                    }
                }
            }
            // Drain events with the tunnel borrow live, act afterwards.
            let mut drained: Vec<TunnelEvent> = Vec::new();
            if let Some(tunnel) = session.tunnel.as_mut() {
                while let Ok(event) = tunnel.events.try_recv() {
                    drained.push(event);
                }
            }
            for event in drained {
                match event {
                    TunnelEvent::Downloading(p) => {
                        session.tunnel_error = None;
                        session.tunnel_progress = Some(p);
                    }
                    TunnelEvent::Starting => {
                        session.tunnel_progress = None;
                    }
                    TunnelEvent::Ready(url) => {
                        session.tunnel_url = Some(url);
                        session.qr_target = QrTarget::Tunnel;
                    }
                    TunnelEvent::Failed(err) => {
                        if session.tunnel_url.is_none() {
                            session.tunnel_error = Some(err);
                            // Drop the dead handle so Start can be
                            // pressed again (the guard in
                            // remote_tunnel_start rejects Some).
                            if let Some(mut tunnel) = session.tunnel.take() {
                                tunnel.stop();
                            }
                            session.tunnel_starting = false;
                        }
                    }
                }
            }
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

        // --- frames (on-demand: only the tab the phone watches gets
        // serialized; other tabs keep their last frame in the map.
        // Unchanged content keeps its seq - zero push when idle.) ---
        let theme = self.terminal_theme_cache.clone();
        let mut changed: Vec<(String, crate::remote::ansi::FrameMsg)> = Vec::new();
        let session = self.remote.as_mut().unwrap();
        let watch = session
            .remote_focus_tab
            .clone()
            .or_else(|| self.focused_terminal.clone());
        if let Some(id) = watch {
            if let Some(td) = self.terminals.get_mut(&id) {
                let content = td.instance.backend.sync();
                let seq = session.frame_seq.entry(id.clone()).or_insert(0);
                let mut frame = crate::remote::ansi::serialize_frame(
                    &content.grid,
                    &theme,
                    &content.terminal_mode,
                    *seq,
                );
                let prev = session.last_ansi.get(&id);
                if prev != Some(&frame.d) {
                    *seq = seq.wrapping_add(1);
                    frame.seq = *seq;
                    session.last_ansi.insert(id.clone(), frame.d.clone());
                    changed.push((id.clone(), frame.clone()));
                }
                session.shared.frames.write().unwrap().insert(id, frame);
            }
        }
        // Broadcast changed frames; prune dead subscribers.
        if !changed.is_empty() {
            let mut subs = session.shared.subscribers.lock().unwrap();
            let payloads: Vec<(String, String)> = changed
                .iter()
                .map(|(id, frame)| (id.clone(), crate::remote::ansi::frame_json(id, frame)))
                .collect();
            subs.retain(|tx| {
                payloads.iter().all(|(_, payload)| {
                    tx.send(crate::remote::ws::WsOut::Text(payload.clone()))
                        .is_ok()
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
                        // Address target selector (LAN / IPv6 / tunnel).
                        let ipv6 = self.remote.as_ref().and_then(|s| s.ipv6.clone());
                        let tunnel_state = self
                            .remote
                            .as_ref()
                            .map(|s| (s.tunnel_url.clone(), s.tunnel_starting, s.tunnel_progress))
                            .unwrap_or((None, false, None));
                        ui.label(egui::RichText::new(&t.on_hint).size(11.0).color(weak));
                        ui.horizontal(|ui| {
                            let target = self.remote.as_ref().unwrap().qr_target;
                            let mut pick = target;
                            if ui
                                .selectable_label(target == QrTarget::Lan, &t.addr_lan)
                                .clicked()
                            {
                                pick = QrTarget::Lan;
                            }
                            if let Some(ip) = ipv6.as_ref() {
                                if ui
                                    .selectable_label(
                                        target == QrTarget::Ipv6,
                                        format!("{} {}", t.addr_ipv6, &ip[..ip.len().min(12)]),
                                    )
                                    .clicked()
                                {
                                    pick = QrTarget::Ipv6;
                                }
                            }
                            if tunnel_state.0.is_some()
                                && ui
                                    .selectable_label(target == QrTarget::Tunnel, &t.addr_tunnel)
                                    .clicked()
                            {
                                pick = QrTarget::Tunnel;
                            }
                            if pick != target {
                                self.remote.as_mut().unwrap().qr_target = pick;
                            }
                        });
                        // Tunnel control row.
                        ui.horizontal(|ui| {
                            if tunnel_state.1 {
                                let pct = tunnel_state
                                    .2
                                    .map(|p| format!(" ({:.0}%)", p * 100.0))
                                    .unwrap_or_default();
                                ui.label(
                                    egui::RichText::new(format!("{}{pct}", t.tunnel_starting))
                                        .size(11.0)
                                        .color(weak),
                                );
                            } else if tunnel_state.0.is_some() {
                                ui.label(
                                    egui::RichText::new(&t.tunnel_ready)
                                        .size(11.0)
                                        .color(accent),
                                );
                            } else {
                                if let Some(err) =
                                    self.remote.as_ref().and_then(|s| s.tunnel_error.clone())
                                {
                                    ui.label(
                                        egui::RichText::new(format!("{}: {err}", t.tunnel_failed))
                                            .size(10.0)
                                            .color(self.active_theme.app.danger.to_egui()),
                                    );
                                }
                                if ui.button(&t.tunnel_start).clicked() {
                                    self.remote_tunnel_start();
                                }
                            }
                            if ui.button(&t.copy_url).clicked() {
                                if let Some(url) = self.remote.as_ref().unwrap().qr_url() {
                                    ui.ctx().copy_text(url.clone());
                                    self.update_toast = Some((
                                        t.copied.clone(),
                                        Instant::now() + Duration::from_secs(4),
                                    ));
                                }
                            }
                            if ui.button(&t.stop).clicked() {
                                self.remote_stop();
                            }
                        });
                        // QR for the selected target.
                        if let Some(url) = self.remote.as_ref().unwrap().qr_url() {
                            ui.label(egui::RichText::new(&url).size(11.0).color(accent));
                            ui.add_space(6.0);
                            self.remote_draw_qr(ui, &url);
                        } else if self.remote.as_ref().is_some_and(|s| s.ip.is_empty()) {
                            ui.label(egui::RichText::new(&t.no_addr).size(11.0).color(weak));
                        }
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(&t.security_hint).size(10.0).color(weak));
                        if tunnel_state.0.is_some() {
                            ui.label(
                                egui::RichText::new(&t.tunnel_warning)
                                    .size(10.0)
                                    .color(weak),
                            );
                        }
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

    /// Draw the QR matrix with plain rects (no image dependency). The
    /// matrix is cached per URL so a static QR costs nothing after the
    /// first frame.
    fn remote_draw_qr(&mut self, ui: &mut egui::Ui, url: &str) {
        let cache_hit = self
            .remote
            .as_ref()
            .and_then(|s| s.qr_cache.as_ref())
            .is_some_and(|(cached_url, _)| cached_url == url);
        if !cache_hit {
            let Ok(code) = qrcode::QrCode::new(url.as_bytes()) else {
                ui.label("QR error");
                return;
            };
            let width = code.width();
            let modules: Vec<bool> = code
                .to_colors()
                .into_iter()
                .map(|c| matches!(c, qrcode::Color::Dark))
                .collect();
            if let Some(session) = self.remote.as_mut() {
                session.qr_cache = Some((url.to_string(), modules));
                session.qr_width = width;
            }
        }
        let Some(session) = self.remote.as_ref() else {
            return;
        };
        let Some((_, modules)) = session.qr_cache.as_ref() else {
            return;
        };
        draw_qr_modules(ui, modules, session.qr_width);
    }
}

//! Remote phone control: the App-side bridge (snapshot refresh, command
//! execution, server lifecycle) and the desktop panel with the QR code.

use super::*;
use crate::remote::frp::FrpHandle;
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
    /// LAN panel visible (menu → 局域网控制). While ANY panel is
    /// visible the embedded server stays up; both closed = full stop.
    pub lan_panel_visible: bool,
    /// WAN panel visible (menu → 外网控制): tunnel/frp channels run
    /// only while this panel is open.
    pub wan_panel_visible: bool,
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
    /// Managed frpc relay channel (None = not running).
    pub frp: Option<FrpHandle>,
    /// Index of the profile the running frpc serves (None = none).
    pub frp_index: Option<usize>,
    /// Config snapshot the running frpc was started with - the change
    /// detector for the instant-apply sync.
    pub frp_profile: Option<crate::app::TunnelProfile>,
    /// Spawn-result channel for frpc (drained by remote_tick).
    pub frp_joiner: Option<std::sync::mpsc::Receiver<Result<FrpHandle, String>>>,
    /// Latest frp status surfaced to the panel.
    pub frp_url: Option<String>,
    pub frp_error: Option<String>,
    pub frp_progress: Option<f32>,
    pub frp_starting: bool,
    /// Config snapshot of the last FAILED spawn attempt - blocks an
    /// infinite auto-restart loop until the user edits the profile or
    /// switches targets.
    pub frp_failed_profile: Option<crate::app::TunnelProfile>,
    /// Which address the QR shows.
    pub qr_target: QrTarget,
    /// The tab the phone is currently watching (drives on-demand frame
    /// serialization; None = nobody focused yet).
    pub remote_focus_tab: Option<String>,
    /// QR caches per URL: the LAN and WAN panels can be open at the
    /// same time showing two different codes — a single-entry cache
    /// would thrash between them every frame.
    pub qr_caches: HashMap<String, (Vec<bool>, usize)>,
}

/// Which entry URL the QR code shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum QrTarget {
    #[default]
    Lan,
    Ipv6,
    Tunnel,
    /// A user-configured relay channel (index into settings.remote_tunnels).
    Frp(usize),
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
            // Frp targets resolve through App (needs settings access).
            QrTarget::Frp(_) => None,
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
            lan_panel_visible: false,
            wan_panel_visible: false,
            ipv6: crate::remote::protocol::public_ipv6(),
            tunnel: None,
            tunnel_joiner: None,
            tunnel_url: None,
            tunnel_error: None,
            tunnel_progress: None,
            tunnel_starting: false,
            frp: None,
            frp_index: None,
            frp_profile: None,
            frp_joiner: None,
            frp_url: None,
            frp_error: None,
            frp_progress: None,
            frp_starting: false,
            frp_failed_profile: None,
            qr_target: QrTarget::default(),
            remote_focus_tab: None,
            qr_caches: HashMap::new(),
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
            if let Some(mut frp) = session.frp.take() {
                frp.stop();
            }
        }
    }

    /// Menu → 局域网控制: start the embedded server if needed, then
    /// toggle the LAN panel. Closing the LAST panel stops everything.
    pub(crate) fn remote_toggle_lan(&mut self) {
        if self.remote.is_none() {
            if let Err(err) = self.remote_start() {
                self.update_toast = Some((
                    format!("{}: {err}", self.texts.remote.bind_failed),
                    Instant::now() + Duration::from_secs(8),
                ));
                return;
            }
        }
        let Some(session) = self.remote.as_mut() else {
            return;
        };
        session.lan_panel_visible = !session.lan_panel_visible;
        self.remote_stop_if_idle();
    }

    /// Menu → 外网控制: start the server AND the cloudflared quick
    /// tunnel immediately (no extra "start tunnel" click), then toggle
    /// the WAN panel. frp channels follow the WAN panel's selector.
    pub(crate) fn remote_toggle_wan(&mut self) {
        if self.remote.is_none() {
            if let Err(err) = self.remote_start() {
                self.update_toast = Some((
                    format!("{}: {err}", self.texts.remote.bind_failed),
                    Instant::now() + Duration::from_secs(8),
                ));
                return;
            }
        }
        let opening;
        {
            let Some(session) = self.remote.as_mut() else {
                return;
            };
            session.wan_panel_visible = !session.wan_panel_visible;
            opening = session.wan_panel_visible;
        }
        if opening {
            // Direct start: the menu click IS the start action.
            self.remote_tunnel_start();
        }
        self.remote_stop_if_idle();
    }

    /// Panel X / Esc close: drop the visibility and stop the whole
    /// session when no panel remains (server + tunnel + frp all die).
    /// Closing the WAN panel also stops its channels (tunnel + frp).
    pub(crate) fn remote_close_panel(&mut self, lan: bool) {
        let Some(session) = self.remote.as_mut() else {
            return;
        };
        if lan {
            session.lan_panel_visible = false;
        } else {
            session.wan_panel_visible = false;
            // The WAN channels stop with their panel.
            if let Some(mut tunnel) = session.tunnel.take() {
                tunnel.stop();
            }
            session.tunnel_joiner = None;
            session.tunnel_url = None;
            session.tunnel_error = None;
            session.tunnel_progress = None;
            session.tunnel_starting = false;
        }
        self.remote_stop_if_idle();
    }

    fn remote_stop_if_idle(&mut self) {
        let idle = self
            .remote
            .as_ref()
            .is_some_and(|s| !s.lan_panel_visible && !s.wan_panel_visible);
        if idle {
            self.remote_stop();
        }
    }

    /// The URL the QR should show, including relay-channel targets
    /// resolved from settings.
    pub(crate) fn remote_qr_url(&self) -> Option<String> {
        let session = self.remote.as_ref()?;
        match session.qr_target {
            QrTarget::Frp(i) => self
                .settings
                .remote_tunnels
                .get(i)
                .map(crate::remote::frp::relay_url),
            _ => session.qr_url(),
        }
    }

    /// Tear the frp channel down completely (process + all state).
    fn remote_frp_reset(session: &mut RemoteSession) {
        if let Some(mut frp) = session.frp.take() {
            frp.stop();
        }
        session.frp_joiner = None;
        session.frp_index = None;
        session.frp_profile = None;
        session.frp_url = None;
        session.frp_error = None;
        session.frp_progress = None;
        session.frp_starting = false;
        session.frp_failed_profile = None;
    }

    /// Keep the single active frp relay channel in sync with the WAN
    /// panel's selected target and the settings config (instant apply).
    ///
    /// frp runs ONLY while the WAN panel is open (closing the panel
    /// stops the channel, same as the tunnel). Desired state = the
    /// profile the address selector points at. When the desired profile
    /// differs from what is running (config edited, target switched),
    /// the old frpc is stopped and a new one spawned within the same
    /// frame - no restart needed. A config that just FAILED blocks
    /// auto-respawn until edited or reselected.
    pub(crate) fn remote_sync_frp(&mut self) {
        // Compute the desired channel without holding the session borrow.
        let wan_open = self.remote.as_ref().is_some_and(|s| s.wan_panel_visible);
        let desired = if !wan_open {
            // WAN panel closed: the channel stops with it.
            if let Some(session) = self.remote.as_mut() {
                if session.frp.is_some() || session.frp_joiner.is_some() {
                    Self::remote_frp_reset(session);
                }
            }
            None
        } else {
            let session = self.remote.as_ref().unwrap();
            match session.qr_target {
                QrTarget::Frp(i) => self
                    .settings
                    .remote_tunnels
                    .get(i)
                    .filter(|p| p.enabled && !p.server.trim().is_empty() && p.forward_port != 0)
                    .map(|p| (i, p.clone())),
                _ => None,
            }
        };
        let Some((want_index, want_profile)) = desired else {
            if let Some(session) = self.remote.as_mut() {
                if session.frp.is_some() || session.frp_joiner.is_some() {
                    Self::remote_frp_reset(session);
                }
            }
            return;
        };
        let session = self.remote.as_mut().unwrap();
        // Spawn still in flight: adoption will land with this config.
        if session.frp_joiner.is_some() {
            return;
        }
        // Already running the exact desired config.
        if session.frp.is_some()
            && session.frp_index == Some(want_index)
            && session.frp_profile.as_ref() == Some(&want_profile)
        {
            return;
        }
        // Same config just failed: wait for an edit (or reselect) so a
        // broken relay address cannot spin spawn/kill cycles forever.
        if session.frp.is_none() && session.frp_failed_profile.as_ref() == Some(&want_profile) {
            return;
        }
        // Switching channel or config changed: replace the process.
        if let Some(mut frp) = session.frp.take() {
            frp.stop();
        }
        session.frp_joiner = None;
        session.frp_starting = true;
        session.frp_error = None;
        session.frp_url = None;
        session.frp_progress = None;
        session.frp_failed_profile = None;
        session.frp_index = Some(want_index);
        session.frp_profile = Some(want_profile.clone());
        let data_dir = app_data_dir();
        let local_port = session.shared.port;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(FrpHandle::start(
                &data_dir,
                &want_profile,
                want_index,
                local_port,
            ));
        });
        session.frp_joiner = Some(rx);
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
        // Relay channel sync (instant apply): selection / config changes
        // start, replace or stop the frpc process within this frame.
        self.remote_sync_frp();
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
            // frp plumbing: adopt the spawn result, surface its events.
            if let Some(joiner) = session.frp_joiner.take() {
                match joiner.try_recv() {
                    Ok(Ok(handle)) => {
                        session.frp = Some(handle);
                        session.frp_starting = false;
                        session.frp_failed_profile = None;
                    }
                    Ok(Err(err)) => {
                        session.frp_starting = false;
                        session.frp_error = Some(err);
                        session.frp_failed_profile = session.frp_profile.take();
                        session.frp_index = None;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        session.frp_joiner = Some(joiner);
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        session.frp_starting = false;
                    }
                }
            }
            let mut frp_events: Vec<TunnelEvent> = Vec::new();
            if let Some(frp) = session.frp.as_mut() {
                while let Ok(event) = frp.events.try_recv() {
                    frp_events.push(event);
                }
            }
            for event in frp_events {
                match event {
                    TunnelEvent::Downloading(p) => {
                        session.frp_error = None;
                        session.frp_progress = Some(p);
                    }
                    TunnelEvent::Starting => {
                        session.frp_progress = None;
                    }
                    TunnelEvent::Ready(url) => {
                        session.frp_url = Some(url);
                        session.frp_error = None;
                    }
                    TunnelEvent::Failed(err) => {
                        // A late watchdog after a success is noise; only
                        // real failures (before any URL) tear the
                        // channel down so the user can fix the config.
                        if session.frp_url.is_none() {
                            session.frp_error = Some(err);
                            if let Some(mut frp) = session.frp.take() {
                                frp.stop();
                            }
                            session.frp_failed_profile = session.frp_profile.take();
                            session.frp_index = None;
                            session.frp_starting = false;
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

    /// LAN control panel: fixed LAN address + QR. One of the TWO
    /// independent remote panels (the other is the WAN panel); both can
    /// be open at the same time — the embedded server is shared.
    pub(crate) fn render_lan_panel(&mut self, ctx: &egui::Context) {
        let visible = self.remote.as_ref().is_some_and(|s| s.lan_panel_visible);
        if !visible {
            return;
        }
        let url = self
            .remote
            .as_ref()
            .filter(|s| !s.ip.is_empty())
            .map(|s| s.url());
        let mut open = true;
        egui::Window::new(&self.texts.remote.lan_panel_title)
            .id(egui::Id::new("remote_lan_panel"))
            .open(&mut open)
            .default_width(320.0)
            .default_pos(screen_center(ctx) + egui::vec2(-60.0, 40.0))
            .show(ctx, |ui| {
                let t = self.texts.remote.clone();
                let weak = self.active_theme.app.weak_text.to_egui();
                let accent = self.active_theme.app.accent.to_egui();
                let Some(url) = url.clone() else {
                    ui.label(egui::RichText::new(&t.no_addr).size(11.0).color(weak));
                    return;
                };
                ui.label(egui::RichText::new(&t.on_hint).size(11.0).color(weak));
                ui.label(egui::RichText::new(&url).size(11.0).color(accent));
                ui.add_space(6.0);
                self.remote_draw_qr(ui, &url);
                ui.add_space(4.0);
                ui.label(egui::RichText::new(&t.security_hint).size(10.0).color(weak));
                ui.horizontal(|ui| {
                    if ui.button(&t.copy_url).clicked() {
                        ui.ctx().copy_text(url.clone());
                        self.update_toast =
                            Some((t.copied.clone(), Instant::now() + Duration::from_secs(4)));
                    }
                    if ui.button(&t.stop).clicked() {
                        // Stop EVERYTHING (both panels + channels).
                        self.remote_stop();
                    }
                });
            });
        if !open {
            self.remote_close_panel(true);
        }
    }

    /// WAN control panel: IPv6 direct / cloudflared quick tunnel /
    /// frp relay channels. Selecting a target switches the QR; relay
    /// channels auto-start on selection (instant apply).
    pub(crate) fn render_wan_panel(&mut self, ctx: &egui::Context) {
        let visible = self.remote.as_ref().is_some_and(|s| s.wan_panel_visible);
        if !visible {
            return;
        }
        let mut open = true;
        egui::Window::new(&self.texts.remote.wan_panel_title)
            .id(egui::Id::new("remote_wan_panel"))
            .open(&mut open)
            .default_width(320.0)
            .default_pos(screen_center(ctx) + egui::vec2(60.0, 40.0))
            .show(ctx, |ui| {
                let t = self.texts.remote.clone();
                let weak = self.active_theme.app.weak_text.to_egui();
                let accent = self.active_theme.app.accent.to_egui();
                // Address target selector (IPv6 / tunnel / relay
                // channels — LAN lives in its own panel). Selecting a
                // relay channel auto-starts it within a frame.
                let ipv6 = self.remote.as_ref().and_then(|s| s.ipv6.clone());
                let tunnel_state = self
                    .remote
                    .as_ref()
                    .map(|s| (s.tunnel_url.clone(), s.tunnel_starting, s.tunnel_progress))
                    .unwrap_or((None, false, None));
                let frp_state = self
                    .remote
                    .as_ref()
                    .map(|s| {
                        (
                            s.frp_url.clone(),
                            s.frp_starting,
                            s.frp_progress,
                            s.frp_error.clone(),
                        )
                    })
                    .unwrap_or((None, false, None, None));
                let relay_channels: Vec<(usize, String)> = self
                    .settings
                    .remote_tunnels
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.enabled)
                    .map(|(i, p)| (i, p.name.clone()))
                    .collect();
                ui.horizontal(|ui| {
                    let target = self.remote.as_ref().unwrap().qr_target;
                    let mut pick = target;
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
                    for (i, name) in &relay_channels {
                        if ui
                            .selectable_label(target == QrTarget::Frp(*i), name.clone())
                            .clicked()
                        {
                            pick = QrTarget::Frp(*i);
                        }
                    }
                    if pick != target {
                        self.remote.as_mut().unwrap().qr_target = pick;
                    }
                });
                // Relay-channel status row (only while a relay target is
                // selected; the channel lifecycle is driven by
                // remote_sync_frp, not by buttons).
                if matches!(self.remote.as_ref().unwrap().qr_target, QrTarget::Frp(_)) {
                    ui.horizontal(|ui| {
                        if frp_state.1 {
                            let pct = frp_state
                                .2
                                .map(|p| format!(" ({:.0}%)", p * 100.0))
                                .unwrap_or_default();
                            ui.label(
                                egui::RichText::new(format!("{}{pct}", t.tunnel_starting))
                                    .size(11.0)
                                    .color(weak),
                            );
                        } else if frp_state.0.is_some() {
                            ui.label(
                                egui::RichText::new(&t.tunnel_ready)
                                    .size(11.0)
                                    .color(accent),
                            );
                        } else if let Some(err) = frp_state.3.as_ref() {
                            ui.label(
                                egui::RichText::new(format!("{}: {err}", t.tunnel_failed))
                                    .size(10.0)
                                    .color(self.active_theme.app.danger.to_egui()),
                            );
                            if ui.button(&t.relay_retry).clicked() {
                                // Forget the failure so the sync
                                // respawns the channel.
                                if let Some(session) = self.remote.as_mut() {
                                    session.frp_failed_profile = None;
                                }
                            }
                        }
                    });
                    ui.label(egui::RichText::new(&t.relay_hint).size(10.0).color(weak));
                }
                // Tunnel control row (the quick tunnel auto-starts when
                // the panel opens; this row shows progress and retries).
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
                        if let Some(err) = self.remote.as_ref().and_then(|s| s.tunnel_error.clone())
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
                        if let Some(url) = self.remote_qr_url() {
                            ui.ctx().copy_text(url.clone());
                            self.update_toast =
                                Some((t.copied.clone(), Instant::now() + Duration::from_secs(4)));
                        }
                    }
                });
                // QR for the selected target.
                if let Some(url) = self.remote_qr_url() {
                    ui.label(egui::RichText::new(&url).size(11.0).color(accent));
                    ui.add_space(6.0);
                    self.remote_draw_qr(ui, &url);
                } else {
                    ui.label(
                        egui::RichText::new(&t.tunnel_starting)
                            .size(11.0)
                            .color(weak),
                    );
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
                if ui.button(&t.stop).clicked() {
                    // Stop EVERYTHING (both panels + channels).
                    self.remote_stop();
                }
            });
        if !open {
            self.remote_close_panel(false);
        }
    }

    /// Draw the QR matrix with plain rects (no image dependency).
    /// Caches are keyed per URL: the LAN and WAN panels can show two
    /// different codes in the same frame.
    fn remote_draw_qr(&mut self, ui: &mut egui::Ui, url: &str) {
        let cached = self
            .remote
            .as_ref()
            .and_then(|s| s.qr_caches.get(url))
            .cloned();
        let (modules, width) = match cached {
            Some(hit) => hit,
            None => {
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
                    session
                        .qr_caches
                        .insert(url.to_string(), (modules.clone(), width));
                }
                (modules, width)
            }
        };
        draw_qr_modules(ui, &modules, width);
    }
}

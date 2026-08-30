//! Monitor panel + the process-sampling worker plumbing (roadmap
//! batch 4). Sampling runs off-thread; the panel renders the latest rows.

use super::*;

/// Process-tree sample job: pid roots for (all terminals, active
/// workspace, focused terminal).
/// (all, active-workspace, focused) root pid groups + per-terminal
/// (tab id, root pid) pairs for the monitor panel.
pub(crate) type ProcSampleJob = (Vec<u32>, Vec<u32>, Vec<u32>, Vec<(String, u32)>);

/// Aggregated status-bar metrics, computed OFF the UI thread by the
/// persistent sampler worker and delivered through the egui temp store.
#[derive(Clone, Default)]
pub(crate) struct ProcSampleResult {
    pub(crate) all: (Option<f32>, Option<u64>),
    pub(crate) workspace: (Option<f32>, Option<u64>),
    pub(crate) focused: (Option<f32>, Option<u64>),
    /// (tab id, cpu%, rss bytes) per sampled terminal.
    pub(crate) per_terminal: Vec<(String, Option<f32>, Option<u64>)>,
}

pub(crate) fn proc_sample_id() -> egui::Id {
    egui::Id::new("proc_sample_result")
}

/// Spawn the persistent process sampler worker. It owns the delta
/// sampler (previous snapshot lives across jobs, so CPU% stays correct)
/// and pushes each result into the egui temp store. The full process
/// table scan — tens of ms on busy Windows machines — no longer runs on
/// the UI thread.
pub(crate) fn spawn_proc_sampler(ctx: egui::Context) -> std::sync::mpsc::Sender<ProcSampleJob> {
    let (tx, rx) = std::sync::mpsc::channel::<ProcSampleJob>();
    let _ = std::thread::Builder::new()
        .name("opennex-proc-sampler".into())
        .spawn(move || {
            let mut sampler = crate::proc_stats::ProcSampler::new();
            while let Ok((all_roots, ws_roots, focused_roots, per_terminal)) = rx.recv() {
                let mut result = ProcSampleResult::default();
                result.per_terminal = sampler.refresh_mixed(
                    [&focused_roots, &ws_roots, &all_roots],
                    [
                        &mut result.focused.0,
                        &mut result.workspace.0,
                        &mut result.all.0,
                    ],
                    [
                        &mut result.focused.1,
                        &mut result.workspace.1,
                        &mut result.all.1,
                    ],
                    &per_terminal,
                );
                ctx.memory_mut(|mem| {
                    mem.data.insert_temp(proc_sample_id(), result);
                });
                ctx.request_repaint();
            }
        });
    tx
}

impl App {
    /// Floating monitor: global terminal-tree CPU/MEM with a sparkline,
    /// plus a per-terminal list (process trees) sorted by CPU. Clicking a
    /// row focuses that terminal.
    pub(crate) fn render_monitor_panel(&mut self, ctx: &egui::Context) {
        if !self.show_monitor {
            return;
        }
        let mut open = true;
        egui::Window::new(&self.texts.monitor.title)
            .id(egui::Id::new("monitor_panel"))
            .open(&mut open)
            .default_width(380.0)
            .default_pos(screen_center(ctx) + egui::vec2(180.0, 60.0))
            .show(ctx, |ui| {
                let t = self.texts.monitor.clone();
                let weak = self.active_theme.app.weak_text.to_egui();
                let accent = self.active_theme.app.accent.to_egui();

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&t.global).size(11.0).color(weak));
                    ui.label(format!("{} {}", t.cpu, format_cpu(self.terminal_cpu)));
                    ui.label(format!("{} {}", t.memory, format_memory(self.terminal_mem)));
                });

                // Sparkline of the recent global CPU% samples.
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 48.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
                let history = &self.cpu_history;
                if history.len() >= 2 {
                    let max = 100.0f32.max(history.iter().cloned().fold(0.0f32, f32::max));
                    let last = history.len() - 1;
                    let pts: Vec<egui::Pos2> = history
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let x = rect.left() + rect.width() * (i as f32 / last.max(1) as f32);
                            let y = rect.bottom() - rect.height() * (v / max).clamp(0.0, 1.0);
                            egui::pos2(x, y)
                        })
                        .collect();
                    ui.painter()
                        .add(egui::Shape::line(pts, egui::Stroke::new(1.5, accent)));
                }

                ui.separator();
                let mut rows = self.monitor_rows.clone();
                rows.sort_by(|a, b| {
                    b.1.unwrap_or(0.0)
                        .partial_cmp(&a.1.unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                egui::ScrollArea::vertical()
                    .max_height(260.0)
                    .show(ui, |ui| {
                        for (tab_id, cpu, mem) in &rows {
                            let name = self
                                .terminals
                                .get(tab_id)
                                .map(|td| td.name.clone())
                                .unwrap_or_else(|| tab_id.clone());
                            ui.horizontal(|ui| {
                                if ui.selectable_label(false, &name).clicked() {
                                    for (idx, tree) in self.dock_states.clone() {
                                        if tree.find_tab(tab_id).is_some() {
                                            self.active_panel = idx;
                                            break;
                                        }
                                    }
                                    self.focused_terminal = Some(tab_id.clone());
                                }
                                let frac = (cpu.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
                                ui.add_sized(
                                    [ui.available_width() - 92.0, 14.0],
                                    egui::ProgressBar::new(frac).show_percentage(),
                                );
                                ui.monospace(format_memory(*mem));
                            });
                        }
                        if rows.is_empty() {
                            ui.label(egui::RichText::new(&t.empty_hint).size(11.0).color(weak));
                        }
                    });
            });
        if !open {
            self.show_monitor = false;
        }
    }
}

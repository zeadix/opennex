//! The global command-history / auto-match menu (roadmap batch 4 of the
//! original plan, extracted during the module split): the Alt-menu
//! overlay, its favorites columns, the clear/favorite dialogs and the
//! keyboard-shortcut detection for the Alt toggle.

use super::*;

pub(crate) fn history_menu_shortcut_released(
    ctx: &egui::Context,
    binds: &HashMap<String, ShortcutBinding>,
    state: &mut AltKeyState,
) -> bool {
    let Some(binding) = binds.get("history_menu") else {
        return false;
    };
    if binding.key != "Alt" || binding.ctrl || binding.shift || !binding.alt {
        return false;
    }

    let other_key_pressed = ctx.input(|input| {
        input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.alt
            )
        })
    });
    ctx.input(|input| update_alt_key_state(state, input.modifiers.alt, other_key_pressed))
}

pub(crate) fn toggle_history_menu(
    nav: &mut Option<HistoryNav>,
    entries: Vec<String>,
    favorites: Vec<String>,
) {
    if nav.is_some() {
        *nav = None;
    } else {
        // Open even when the terminal has no history yet: the fixed
        // 10-row list doubles as the entry point to the favorites.
        *nav = Some(HistoryNav {
            entries,
            selected: 0,
            auto_word: None,
            favorites,
            fav_focused: false,
            fav_selected: 0,
        });
    }
}

#[derive(Default)]
pub(crate) struct AltKeyState {
    pub(crate) pressed: bool,
    pub(crate) used_with_other_key: bool,
}

impl App {
    /// Global command-history / auto-match list for the focused terminal.
    /// Rendered as a Foreground layer Area so it is never clipped by the
    /// terminal's rect. Features: outer border matching the UI divider
    /// style, row index numbers (dimmed), alternating row colors from the
    /// theme (menu_bg / menu_alt_bg), wheel scrolling, a footer with the
    /// total entry count and a "clear" button (with confirmation).
    pub(crate) fn render_history_menu(&mut self, ctx: &egui::Context) {
        // The settings window owns the screen while open: the history
        // menu must never float above it.
        if self.show_settings {
            if let Some(tab) = self.focused_terminal.clone() {
                if let Some(td) = self.terminals.get_mut(&tab) {
                    td.instance.history_nav = None;
                }
            }
            self.fav_submenu = None;
            self.fav_sub_focused = false;
            return;
        }
        let Some(tab) = self.focused_terminal.clone() else {
            return;
        };

        // FIRST-frame restoration after a reopen: apply the snapshot
        // BEFORE anything else this frame can reset focus states.
        if self.menu_pending_restore.remove(&tab) {
            let snap = self.menu_cursors.get(&tab).copied();
            if let Some((hist_sel, fav_sel, fav_focused, sub_sel, sub_focused, sub_fid)) = snap {
                if let Some(td) = self.terminals.get_mut(&tab) {
                    if let Some(nav) = td.instance.history_nav.as_mut() {
                        nav.selected = hist_sel.min(nav.entries.len().saturating_sub(1));
                        nav.fav_focused = fav_focused;
                        nav.fav_selected = fav_sel.min(self.fav_folders.len().saturating_sub(1));
                    }
                }
                if sub_focused && sub_fid != 0 {
                    let items = self.history_db.fav_items(sub_fid);
                    if !items.is_empty() {
                        self.fav_sub_focused = true;
                        self.fav_submenu = Some((sub_fid, egui::Pos2::ZERO, items, Some(sub_sel)));
                    }
                }
            }
        }

        let Some(td) = self.terminals.get_mut(&tab) else {
            return;
        };
        let Some(nav) = td.instance.history_nav.clone() else {
            return;
        };

        // Snapshot the cursor/panel state EVERY frame while the MANUAL
        // (Alt) menu is open, so ANY close path (Esc, Enter-send,
        // click-away) persists it for the next open. AUTO-MATCH sessions
        // (auto_word set) share this HistoryNav slot but are a different
        // feature: they must never overwrite the manual menu's snapshot
        // (their fav_focused is false and their transient column state
        // is gone — snapshotting them reset every Alt reopen to the
        // history list).
        if nav.auto_word.is_none() {
            let (sub_sel_snap, sub_fid_snap) = match &self.fav_submenu {
                Some((fid, _, _, sel)) => (sel.unwrap_or(0), *fid),
                None => (0, 0),
            };
            let snap = (
                nav.selected,
                nav.fav_selected,
                nav.fav_focused,
                sub_sel_snap,
                self.fav_sub_focused,
                sub_fid_snap,
            );
            self.menu_cursors.insert(tab.clone(), snap);
        }

        let app = &self.active_theme.app;
        let menu_bg = app.menu_bg.to_egui();
        let menu_alt = app.menu_alt_bg.to_egui();
        let menu_fg = app.menu_fg.to_egui();
        let weak = app.weak_text.to_egui();

        let sel_bg = app.active.to_egui();
        let border = app.sidebar_border.to_egui();
        let font_size = self.active_theme.typography.menu_font_size;

        // Adaptive geometry: height fits the content (capped at 10 rows)
        // so a 2-entry history doesn't float a mostly-empty 10-row menu
        // (v0.1.37 UI audit). Footer shared by both columns, favorites
        // column glued to the right of the main list inside the SAME window.
        let row_h = 20.0f32;
        let max_visible = 10usize;
        let footer_h = 24.0f32;
        let list_w = 250.0f32;
        let fav_w = 200.0f32;
        let visible_rows = nav.entries.len().min(max_visible).max(1);
        // Column height must fit BOTH columns: the main history list and
        // the favorites/folder column (which can hold more rows than the
        // main list has entries — a folder tree with no history would
        // otherwise clip the folders out of view).
        // FIXED 10-row height (v0.1.47): the favorites/folder column must
        // always have room for its full tree regardless of how few history
        // entries exist — an adaptive height clipped it to the main list.
        // The columns scroll internally when content overflows.
        let rows_h = max_visible as f32 * row_h;
        let list_h = rows_h + footer_h;
        // History column header (24px, aligned with the favorites column's
        // "new folder" header): the "clear history" button lives on its
        // right, so the history rows start below it.
        let hist_header_h = 24.0f32;
        let hist_visible_rows = ((rows_h - hist_header_h) / row_h).floor() as usize;
        // The folder column is ALWAYS visible in the manual menu (even
        // with zero legacy favorites or after clear-favorites) — the
        // folders themselves are the feature now.
        let show_favs = nav.auto_word.is_none();
        // The folder's command list renders as a THIRD column in the
        // same window (flush against the folder column, same style) —
        // not a floating popup. The command column is a PERMANENT
        // fixture in the manual menu: always shown, even for an empty
        // favorites list (empty column = no rows, just background).
        let sub_w = 200.0f32;
        let total_w =
            list_w + if show_favs { fav_w } else { 0.0 } + if show_favs { sub_w } else { 0.0 };
        let total = nav.entries.len();
        let fav_total = nav.favorites.len();

        // Independent scroll offsets for the two columns.
        let scroll_id = egui::Id::new(("hist_menu_scroll", tab.as_str()));
        let fav_scroll_id = egui::Id::new(("hist_fav_scroll", tab.as_str()));
        let max_scroll = total.saturating_sub(hist_visible_rows);
        let fav_max_scroll = fav_total.saturating_sub(visible_rows);
        let mut scroll: usize = ctx
            .memory(|m| m.data.get_temp(scroll_id).unwrap_or(0))
            .min(max_scroll);
        let mut fav_scroll: usize = ctx
            .memory(|m| m.data.get_temp(fav_scroll_id).unwrap_or(0))
            .min(fav_max_scroll);

        // Anchor: follow the terminal CURSOR row — below the cursor line
        // when there is room, otherwise above it.
        let anchor_rect = self
            .terminal_view_rects
            .get(&tab)
            .copied()
            .unwrap_or_else(|| {
                let sr = ctx.screen_rect();
                egui::Rect::from_min_max(
                    egui::pos2(sr.min.x + 200.0, sr.min.y + 60.0),
                    egui::pos2(sr.max.x, sr.max.y),
                )
            });
        let (cell_w, cell_h_grid) = td.instance.cell_size();
        let (cursor_col, cursor_row) = td.instance.cursor_position();
        #[allow(unused_variables)]
        let cursor_x = anchor_rect.min.x + cursor_col as f32 * cell_w;
        let cursor_y = anchor_rect.min.y + (cursor_row as f32 + 1.0) * cell_h_grid;
        let screen = ctx.screen_rect();
        let mut pos = if anchor_rect.max.y - cursor_y >= list_h {
            egui::pos2(anchor_rect.min.x + 8.0, cursor_y)
        } else {
            egui::pos2(
                anchor_rect.min.x + 8.0,
                (cursor_y - cell_h_grid - list_h).max(anchor_rect.min.y + 4.0),
            )
        };
        // Clamp onto the screen: in NARROW panes a wrapped prompt pushes
        // the cursor row far down (or the recomputed cursor_y overshoots
        // the pane bottom), which used to place the whole menu OUTSIDE
        // the window — the menu was built (toast) but invisible.
        let total_w_actual = list_w + if show_favs { fav_w } else { 0.0 };
        pos.x = pos.x.clamp(
            screen.min.x + 4.0,
            (screen.max.x - total_w_actual - 4.0).max(screen.min.x + 4.0),
        );
        pos.y = pos.y.clamp(
            screen.min.y + 4.0,
            (screen.max.y - list_h - 4.0).max(screen.min.y + 4.0),
        );

        let mut entry_clicked: Option<usize> = None;
        let mut row_fav_clicked: Option<usize> = None;
        let mut row_del_clicked: Option<usize> = None;
        let mut clear_favs_clicked = false;
        let mut clear_history_clicked = false;
        let mut close_clicked = false;

        egui::Area::new(egui::Id::new(("hist_menu", tab.as_str())))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                // One fixed-size window; everything below is painted on
                // absolutely-computed rects (no nested layout direction
                // can disturb the vertical row stacking).
                let (frame_rect, _) =
                    ui.allocate_exact_size(egui::vec2(total_w, list_h), egui::Sense::hover());
                ui.painter().rect_filled(frame_rect, 0.0, menu_bg);
                // Border is painted LAST (after all rows/scrollbars/footer)
                // so the list UI can never cover it.

                let hover_pos = ui
                    .input(|i| i.pointer.hover_pos())
                    .unwrap_or(egui::pos2(-1.0, -1.0));

                // Wheel: the column under the pointer scrolls.
                let wheel = ui.input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| match e {
                            egui::Event::MouseWheel { delta, .. } => Some(delta.y),
                            _ => None,
                        })
                        .sum::<f32>()
                });
                if wheel != 0.0 {
                    let over_fav = show_favs && hover_pos.x >= frame_rect.min.x + list_w;
                    if over_fav {
                        if wheel > 0.0 {
                            fav_scroll = fav_scroll.saturating_sub(1);
                        } else {
                            fav_scroll = (fav_scroll + 1).min(fav_max_scroll);
                        }
                    } else if wheel > 0.0 {
                        scroll = scroll.saturating_sub(1);
                    } else {
                        scroll = (scroll + 1).min(max_scroll);
                    }
                }

                // ---- History column header: "clear history" button on
                // the right (moved out of the shared footer). ----
                {
                    let htxt = self.texts.terminal.clear_history.clone();
                    let hg = ui.fonts(|f| {
                        f.layout_no_wrap(htxt.clone(), egui::FontId::proportional(11.0), weak)
                    });
                    let hw = hg.size().x + 10.0;
                    let h_rect = egui::Rect::from_min_size(
                        egui::pos2(frame_rect.min.x + list_w - 8.0 - hw, frame_rect.min.y + 4.0),
                        egui::vec2(hw, 16.0),
                    );
                    let hresp = ui.interact(
                        h_rect,
                        egui::Id::new(("hist_clear", tab.as_str())),
                        egui::Sense::click(),
                    );
                    let hcol = if hresp.hovered() {
                        app.danger.to_egui()
                    } else {
                        weak
                    };
                    let hbg = if hresp.contains_pointer() {
                        egui::Color32::from_rgba_unmultiplied(
                            sel_bg.r(),
                            sel_bg.g(),
                            sel_bg.b(),
                            90,
                        )
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(h_rect, 3.0, hbg);
                    let hg2 = ui.fonts(|f| {
                        f.layout_no_wrap(htxt.clone(), egui::FontId::proportional(11.0), hcol)
                    });
                    ui.painter()
                        .galley(h_rect.center() - hg2.size() / 2.0, hg2, hcol);
                    if hresp.clicked() {
                        clear_history_clicked = true;
                    }
                }

                // ---- Empty state: actionable hint instead of a bare
                // empty list (v0.1.37 UI audit P3-12). ----
                if total == 0 {
                    ui.painter().text(
                        egui::pos2(
                            frame_rect.min.x + 12.0,
                            frame_rect.min.y + hist_header_h + row_h * 0.5,
                        ),
                        egui::Align2::LEFT_CENTER,
                        self.texts.terminal.history_empty.clone(),
                        egui::FontId::proportional(11.0),
                        weak,
                    );
                }

                // ---- Main rows ----
                for i in scroll..(scroll + hist_visible_rows).min(total) {
                    let row = egui::Rect::from_min_size(
                        egui::pos2(
                            frame_rect.min.x,
                            frame_rect.min.y + hist_header_h + (i - scroll) as f32 * row_h,
                        ),
                        egui::vec2(list_w, row_h),
                    );
                    let is_sel = i == nav.selected;
                    let row_hovered = row.contains(hover_pos);
                    let row_bg = if is_sel {
                        sel_bg
                    } else if row_hovered {
                        egui::Color32::from_rgba_unmultiplied(
                            sel_bg.r(),
                            sel_bg.g(),
                            sel_bg.b(),
                            90,
                        )
                    } else if i % 2 == 1 {
                        menu_alt
                    } else {
                        menu_bg
                    };
                    ui.painter().rect_filled(row, 0.0, row_bg);
                    ui.painter().text(
                        egui::pos2(row.min.x + 6.0, row.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!("{}", i + 1),
                        egui::FontId::monospace(font_size * 0.85),
                        weak,
                    );
                    let text_x = row.min.x + 34.0;
                    let font_id = egui::FontId::monospace(font_size);
                    let full = ui.fonts(|f| {
                        f.layout_no_wrap(nav.entries[i].clone(), font_id.clone(), menu_fg)
                    });
                    // No ellipsis: overlong text is clipped at the row's
                    // right edge (minus scrollbar/button gutter).
                    let clip_w = row.max.x - 16.0;
                    ui.set_clip_rect(egui::Rect::from_min_max(
                        egui::pos2(text_x, row.min.y),
                        egui::pos2(clip_w, row.max.y),
                    ));
                    ui.painter().galley(
                        egui::pos2(text_x, row.center().y - full.size().y / 2.0),
                        full,
                        menu_fg,
                    );
                    // Restore with a 1px bleed so the later outer border
                    // (stroked on the boundary) keeps its outer half.
                    ui.set_clip_rect(frame_rect.expand(1.0));
                    let resp = ui.interact(
                        row,
                        egui::Id::new(("hist_row", tab.as_str(), i)),
                        egui::Sense::click_and_drag(),
                    );
                    if resp.clicked() {
                        entry_clicked = Some(i);
                    }
                    // Drag a history entry onto a favorites FOLDER to
                    // copy it there (history keeps its entry). The drag
                    // must START on this row (press origin inside it,
                    // outside the trailing action gutter); once armed it
                    // follows the pointer regardless of hover.
                    let press = ui.input(|i| i.pointer.press_origin());
                    let starts_here =
                        press.is_some_and(|p| row.contains(p) && p.x <= row.max.x - 70.0);
                    let primary_down = ui.input(|i| i.pointer.primary_down());
                    if starts_here
                        && primary_down
                        && self.hist_drag_cmd.is_none()
                        && self.fav_item_drag.is_none()
                        && self.fav_drag_src.is_none()
                    {
                        if let Some(cmd) = nav.entries.get(i).cloned() {
                            self.hist_drag_cmd = Some(cmd);
                        }
                    }
                    let _ = resp;
                    // Release anywhere ends a history drag; if a folder
                    // target was registered, COPY the command in.
                    if self.hist_drag_cmd.is_some() && ui.input(|i| i.pointer.any_released()) {
                        if let (Some(cmd), Some(fid)) =
                            (self.hist_drag_cmd.take(), self.hist_drop_folder.take())
                        {
                            self.history_db.fav_add_to(fid, &cmd);
                            self.fav_folders = self.history_db.fav_folders();
                        }
                        self.hist_drag_cmd = None;
                        self.hist_drop_folder = None;
                    }
                    // Per-row actions (manual menu only): hover or
                    // keyboard selection shows icon buttons (star = favorite,
                    // trash = delete) on the right.
                    if nav.auto_word.is_none() && (row_hovered || is_sel) {
                        let icon_font = egui::FontId::proportional(13.0);
                        let del_glyph = egui_phosphor::regular::TRASH.to_string();
                        let star_glyph = egui_phosphor::regular::STAR.to_string();
                        let del_g = ui.fonts(|f| {
                            f.layout_no_wrap(del_glyph.clone(), icon_font.clone(), weak)
                        });
                        let star_g = ui.fonts(|f| {
                            f.layout_no_wrap(star_glyph.clone(), icon_font.clone(), weak)
                        });
                        let pad = 6.0;
                        let del_rect = egui::Rect::from_min_max(
                            egui::pos2(row.max.x - 8.0 - del_g.size().x, row.center().y - 8.0),
                            egui::pos2(row.max.x - 8.0, row.center().y + 8.0),
                        );
                        let star_rect = egui::Rect::from_min_max(
                            egui::pos2(
                                del_rect.min.x - pad - star_g.size().x,
                                row.center().y - 8.0,
                            ),
                            egui::pos2(del_rect.min.x - pad, row.center().y + 8.0),
                        );
                        let del_resp = ui.interact(
                            del_rect,
                            egui::Id::new(("hist_row_del", tab.as_str(), i)),
                            egui::Sense::click(),
                        );
                        let star_resp = ui.interact(
                            star_rect,
                            egui::Id::new(("hist_row_star", tab.as_str(), i)),
                            egui::Sense::click(),
                        );
                        let del_col = if del_resp.hovered() {
                            app.danger.to_egui()
                        } else {
                            weak
                        };
                        let star_col = if star_resp.hovered() {
                            app.accent.to_egui()
                        } else {
                            weak
                        };
                        let del_g2 = ui.fonts(|f| {
                            f.layout_no_wrap(del_glyph.clone(), icon_font.clone(), del_col)
                        });
                        let star_g2 = ui.fonts(|f| {
                            f.layout_no_wrap(star_glyph.clone(), icon_font.clone(), star_col)
                        });
                        ui.painter().galley(
                            del_rect.center() - del_g2.size() / 2.0,
                            del_g2,
                            del_col,
                        );
                        ui.painter().galley(
                            star_rect.center() - star_g2.size() / 2.0,
                            star_g2,
                            star_col,
                        );
                        if del_resp.clicked() {
                            row_del_clicked = Some(i);
                        }
                        if star_resp.clicked() {
                            row_fav_clicked = Some(i);
                        }
                    }
                }

                // ---- Main scrollbar ----
                if total > hist_visible_rows {
                    let sb_track = egui::Rect::from_min_max(
                        egui::pos2(
                            frame_rect.min.x + list_w - 6.0,
                            frame_rect.min.y + hist_header_h,
                        ),
                        egui::pos2(frame_rect.min.x + list_w, frame_rect.min.y + rows_h),
                    );
                    let track_h = rows_h - hist_header_h;
                    let thumb_h = (hist_visible_rows as f32 / total as f32 * track_h).max(16.0);
                    let scrollable = (track_h - thumb_h).max(1.0);
                    let thumb_y = frame_rect.min.y
                        + hist_header_h
                        + scrollable * (scroll as f32 / max_scroll as f32);
                    let sb_col =
                        egui::Color32::from_rgba_unmultiplied(weak.r(), weak.g(), weak.b(), 110);
                    let track_col =
                        egui::Color32::from_rgba_unmultiplied(weak.r(), weak.g(), weak.b(), 40);
                    ui.painter().rect_filled(sb_track, 0.0, track_col);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(sb_track.min.x, thumb_y),
                            egui::vec2(sb_track.width(), thumb_h),
                        ),
                        0.0,
                        sb_col,
                    );
                    let sb_resp = ui.interact(
                        sb_track,
                        egui::Id::new(("hist_sb", tab.as_str())),
                        egui::Sense::click_and_drag(),
                    );
                    if sb_resp.dragged() {
                        let dy = ui.input(|i| i.pointer.delta().y);
                        let lines = dy * max_scroll as f32 / scrollable;
                        scroll = (scroll as f32 + lines)
                            .round()
                            .clamp(0.0, max_scroll as f32)
                            as usize;
                    }
                }

                // ---- Favorites column: folder tree (v0.1.46) ----
                if show_favs || !self.fav_folders.is_empty() {
                    let fx0 = frame_rect.min.x + list_w;
                    let col_w = total_w_actual - list_w;
                    let t = &self.texts.terminal;
                    // Fresh snapshot of folders+items for rendering; the
                    // DB is the source of truth, mutations refresh below.
                    let folders = self.fav_folders.clone();
                    let mut folder_items: Vec<Vec<String>> = Vec::new();
                    for (fid, _) in &folders {
                        folder_items.push(self.history_db.fav_items(*fid));
                    }
                    let rows: Vec<(i64, usize, String)> = folders
                        .iter()
                        .map(|(fid, _)| (*fid, 0, String::new()))
                        .collect();
                    let row_h: f32 = 20.0;
                    // The rows paint below a 24px "new folder" header,
                    // so usable height is rows_h - 24 — without this the
                    // overflow check misses by one row and the scrollbar
                    // never appears for exactly-overflowing lists.
                    let header_h = 24.0f32;
                    let col_usable_h = (rows_h - header_h).max(row_h);
                    let content_h = rows.len() as f32 * row_h;
                    let max_col_scroll =
                        ((content_h - col_usable_h) / row_h).ceil().max(0.0) as usize;
                    let col_scroll_id = egui::Id::new(("hist_favcol_scroll", tab.as_str()));
                    let mut col_scroll: usize = ctx
                        .memory(|m| m.data.get_temp(col_scroll_id).unwrap_or(0))
                        .min(max_col_scroll);

                    let col_full_rect = egui::Rect::from_min_size(
                        egui::pos2(fx0, frame_rect.min.y),
                        egui::vec2(col_w, rows_h + footer_h),
                    );
                    self.fav_column_rect = col_full_rect;
                    ui.painter().rect_filled(col_full_rect, 0.0, menu_bg);

                    // --- "New folder" button at the column top ---
                    let new_txt = t.fav_new_folder.clone();
                    let ng = ui.fonts(|f| {
                        f.layout_no_wrap(
                            format!("+ {new_txt}"),
                            egui::FontId::proportional(11.0),
                            menu_fg,
                        )
                    });
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(fx0 + 6.0, frame_rect.min.y + 4.0),
                        egui::vec2(ng.size().x + 12.0, 16.0),
                    );
                    let nresp = ui.interact(
                        btn_rect,
                        egui::Id::new(("fav_new_folder", tab.as_str())),
                        egui::Sense::click(),
                    );
                    let nbg = if nresp.contains_pointer() {
                        egui::Color32::from_rgba_unmultiplied(
                            sel_bg.r(),
                            sel_bg.g(),
                            sel_bg.b(),
                            90,
                        )
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(btn_rect, 3.0, nbg);
                    ui.painter()
                        .galley(btn_rect.center() - ng.size() / 2.0, ng, menu_fg);
                    if nresp.clicked() {
                        self.fav_name_dialog = Some((None, String::new()));
                        self.fav_name_just_opened = true;
                    }

                    // --- "Clear favorites" button, right-aligned in the
                    // column header (moved out of the shared footer). ---
                    {
                        let ctxt = t.clear_favorites.clone();
                        let cg = ui.fonts(|f| {
                            f.layout_no_wrap(ctxt.clone(), egui::FontId::proportional(11.0), weak)
                        });
                        let cw = cg.size().x + 10.0;
                        let c_rect = egui::Rect::from_min_size(
                            egui::pos2(fx0 + col_w - 8.0 - cw, frame_rect.min.y + 4.0),
                            egui::vec2(cw, 16.0),
                        );
                        let cresp = ui.interact(
                            c_rect,
                            egui::Id::new(("hist_clear_favs", tab.as_str())),
                            egui::Sense::click(),
                        );
                        let ccol = if cresp.hovered() {
                            app.danger.to_egui()
                        } else {
                            weak
                        };
                        let cbg = if cresp.contains_pointer() {
                            egui::Color32::from_rgba_unmultiplied(
                                sel_bg.r(),
                                sel_bg.g(),
                                sel_bg.b(),
                                90,
                            )
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        ui.painter().rect_filled(c_rect, 3.0, cbg);
                        let cg2 = ui.fonts(|f| {
                            f.layout_no_wrap(ctxt.clone(), egui::FontId::proportional(11.0), ccol)
                        });
                        ui.painter()
                            .galley(c_rect.center() - cg2.size() / 2.0, cg2, ccol);
                        if cresp.clicked() {
                            clear_favs_clicked = true;
                        }
                    }

                    // --- Folder & item rows ---
                    let mut y = frame_rect.min.y + 24.0 - col_scroll as f32 * row_h;
                    self.fav_folder_rects.clear();
                    let mut folder_row_index: Vec<(i64, egui::Rect)> = Vec::new();
                    let pointer_down = ui.input(|i| i.pointer.primary_down());
                    let pointer_released = ui.input(|i| i.pointer.any_released());

                    for row in &rows {
                        if y + row_h < frame_rect.min.y + 24.0 {
                            y += row_h;
                            continue;
                        }
                        if y > frame_rect.min.y + rows_h {
                            break;
                        }
                        let (fid, kind, _cmd) = row.clone();
                        let is_header = kind == 0;
                        let fidx = folders.iter().position(|(id, _)| *id == fid).unwrap_or(0);
                        let row_rect =
                            egui::Rect::from_min_size(egui::pos2(fx0, y), egui::vec2(col_w, row_h));
                        let row_hover = row_rect.contains(hover_pos);

                        if is_header {
                            // Folder header row
                            let name = folders[fidx].1.clone();
                            let count = folder_items[fidx].len();
                            let has_submenu = self
                                .fav_submenu
                                .as_ref()
                                .is_some_and(|(sid, _, _, _)| *sid == fid);
                            let expanded = has_submenu;
                            let is_sel = nav.fav_focused && nav.fav_selected == fidx;
                            let bg = if is_sel {
                                sel_bg
                            } else if row_hover {
                                egui::Color32::from_rgba_unmultiplied(
                                    sel_bg.r(),
                                    sel_bg.g(),
                                    sel_bg.b(),
                                    90,
                                )
                            } else {
                                menu_bg
                            };
                            ui.painter().rect_filled(row_rect, 0.0, bg);
                            let hg = ui.fonts(|f| {
                                f.layout_no_wrap(
                                    "≡".to_string(),
                                    egui::FontId::proportional(11.0),
                                    weak,
                                )
                            });
                            ui.painter().galley(
                                egui::pos2(
                                    row_rect.min.x + 6.0,
                                    row_rect.center().y - hg.size().y / 2.0,
                                ),
                                hg,
                                weak,
                            );
                            let caret = if expanded { "▾" } else { "▸" };
                            let label = format!("{caret} {name} ({count})");
                            let fnt = egui::FontId::proportional(font_size * 0.9);
                            let g = ui.fonts(|f| f.layout_no_wrap(label, fnt, menu_fg));
                            ui.set_clip_rect(egui::Rect::from_min_max(
                                egui::pos2(row_rect.min.x, row_rect.min.y),
                                egui::pos2(row_rect.max.x - 60.0, row_rect.max.y),
                            ));
                            ui.painter().galley(
                                egui::pos2(
                                    row_rect.min.x + 20.0,
                                    row_rect.center().y - g.size().y / 2.0,
                                ),
                                g,
                                menu_fg,
                            );
                            ui.set_clip_rect(frame_rect.expand(1.0));

                            let mut assemble_clicked = false;
                            let mut add_cmd_clicked = false;
                            let mut rename_clicked = false;
                            let mut delete_clicked = false;
                            let is_default_folder =
                                name == crate::history_db::HistoryDb::DEFAULT_FAVORITE_FOLDER;
                            let hresp = ui.interact(
                                row_rect,
                                egui::Id::new(("fav_folder", tab.as_str(), fid)),
                                egui::Sense::click_and_drag(),
                            );
                            // Right-click context menu (hover-button
                            // actions + clear/delete folder).
                            hresp.context_menu(|menu_ui| {
                                let t = &self.texts.terminal;
                                if menu_ui.button(t.fav_menu_assemble.clone()).clicked() {
                                    assemble_clicked = true;
                                    menu_ui.close_menu();
                                }
                                if menu_ui.button(t.fav_btn_add_cmd.clone()).clicked() {
                                    add_cmd_clicked = true;
                                    menu_ui.close_menu();
                                }
                                if menu_ui.button(t.fav_btn_rename.clone()).clicked() {
                                    rename_clicked = true;
                                    menu_ui.close_menu();
                                }
                                menu_ui.separator();
                                if menu_ui.button(t.fav_clear_folder.clone()).clicked() {
                                    self.history_db.fav_folder_clear(fid);
                                    self.fav_folders = self.history_db.fav_folders();
                                    if self
                                        .fav_submenu
                                        .as_ref()
                                        .is_some_and(|(sid, _, _, _)| *sid == fid)
                                    {
                                        // Keep the column open, now empty.
                                        self.fav_submenu =
                                            Some((fid, egui::Pos2::ZERO, Vec::new(), None));
                                    }
                                    menu_ui.close_menu();
                                }
                                if !is_default_folder
                                    && menu_ui
                                        .button(
                                            egui::RichText::new(t.fav_btn_delete.clone())
                                                .color(self.active_theme.app.danger.to_egui()),
                                        )
                                        .clicked()
                                {
                                    self.fav_delete_confirm = Some((fid, folders[fidx].1.clone()));
                                    self.fav_del_just_opened = true;
                                    menu_ui.close_menu();
                                }
                            });
                            folder_row_index.push((fid, row_rect));
                            self.fav_folder_rects.push(row_rect);
                            // A HISTORY entry dragged over a folder row:
                            // highlight + register the drop target; the
                            // drop COPIES the command into this folder.
                            if row_hover
                                && self.hist_drag_cmd.is_some()
                                && self.fav_item_drag.is_none()
                            {
                                self.hist_drop_folder = Some(fid);
                                ui.painter().rect_filled(
                                    row_rect,
                                    0.0,
                                    egui::Color32::from_rgba_unmultiplied(
                                        sel_bg.r(),
                                        sel_bg.g(),
                                        sel_bg.b(),
                                        120,
                                    ),
                                );
                            }
                            // During a submenu ITEM drag, hovering another
                            // folder does NOT switch the submenu — the
                            // folder becomes the cross-folder drop target.
                            if row_hover {
                                if let Some((src_fid, _src_idx)) = self.fav_item_drag {
                                    if fid != src_fid {
                                        self.fav_item_drop = Some((fid, usize::MAX, true));
                                        // Highlight the whole folder row as
                                        // the move target.
                                        ui.painter().rect_filled(
                                            row_rect,
                                            0.0,
                                            egui::Color32::from_rgba_unmultiplied(
                                                sel_bg.r(),
                                                sel_bg.g(),
                                                sel_bg.b(),
                                                120,
                                            ),
                                        );
                                    }
                                }
                            }
                            // Hover over a folder row IMMEDIATELY opens
                            // its floating command list (native-menu feel;
                            // user requirement — no click needed).
                            if row_hover
                                && self.fav_item_drag.is_none()
                                && self.hist_drag_cmd.is_none()
                                && self
                                    .fav_submenu
                                    .as_ref()
                                    .is_none_or(|(sid, _, _, _)| *sid != fid)
                            {
                                let items = folder_items[fidx].clone();
                                // Always open the column — even an empty
                                // folder shows an empty command list now
                                // (the column is a permanent fixture).
                                self.fav_submenu = Some((fid, egui::Pos2::ZERO, items, None));
                            }
                            // hover 3 buttons: assemble / rename / delete
                            if row_hover && self.fav_drag_src.is_none() {
                                let mut bx = row_rect.max.x - 8.0;
                                let is_default_folder =
                                    name == crate::history_db::HistoryDb::DEFAULT_FAVORITE_FOLDER;
                                // ICON buttons (user request): PENCIL = rename,
                                // LIGHTNING = assemble, PLUS = add command,
                                // TRASH = delete (default folder: no trash).
                                let act_font = egui::FontId::proportional(13.0);
                                // Painted right-to-left, so the VISUAL order
                                // (left→right) is: 合并(asm) 添加(add)
                                // 修改(ren) 删除(del).
                                let mut btns: Vec<(&str, &str, bool)> = vec![
                                    (egui_phosphor::regular::PENCIL_SIMPLE, "ren", false),
                                    (egui_phosphor::regular::PLUS, "add", false),
                                    (egui_phosphor::regular::LIGHTNING, "asm", false),
                                ];
                                if !is_default_folder {
                                    // Visual rightmost = delete; protected
                                    // default folder shows none.
                                    btns.insert(0, (egui_phosphor::regular::TRASH, "del", true));
                                }
                                for (icon, id_ext, is_danger) in &btns {
                                    let lg = ui.fonts(|f| {
                                        f.layout_no_wrap(icon.to_string(), act_font.clone(), weak)
                                    });
                                    let pad = 4.0;
                                    let brect = egui::Rect::from_min_max(
                                        egui::pos2(bx - lg.size().x, row_rect.center().y - 8.0),
                                        egui::pos2(bx, row_rect.center().y + 8.0),
                                    );
                                    let bresp = ui.interact(
                                        brect,
                                        egui::Id::new((
                                            "fav_folder_btn",
                                            tab.as_str(),
                                            fid,
                                            id_ext,
                                        )),
                                        egui::Sense::click(),
                                    );
                                    let bcol = if bresp.contains_pointer() {
                                        if *is_danger {
                                            app.danger.to_egui()
                                        } else {
                                            app.accent.to_egui()
                                        }
                                    } else {
                                        weak
                                    };
                                    let lw = lg.size().x;
                                    ui.painter()
                                        .galley(brect.center() - lg.size() / 2.0, lg, bcol);
                                    let _ = lw;
                                    match *id_ext {
                                        "asm" => assemble_clicked |= bresp.clicked(),
                                        "add" => add_cmd_clicked |= bresp.clicked(),
                                        "ren" => rename_clicked |= bresp.clicked(),
                                        _ => delete_clicked |= bresp.clicked(),
                                    }
                                    bx -= lw + pad;
                                }
                                if assemble_clicked {
                                    let cmds = self.history_db.fav_items(fid);
                                    if let Some(shell_id) =
                                        self.terminals.get(&tab).map(|td| td.shell_id.clone())
                                    {
                                        let line = assemble_commands(&cmds, &shell_id);
                                        if !line.is_empty()
                                            && !open_snippet_fill_fields(
                                                &mut self.terminals,
                                                &mut self.fav_submenu,
                                                &mut self.fav_sub_focused,
                                                &mut self.history_menu_just_closed,
                                                &mut self.snippet_fill,
                                                &mut self.snippet_fill_just_opened,
                                                &tab,
                                                line.clone(),
                                            )
                                        {
                                            if let Some(td) = self.terminals.get_mut(&tab) {
                                                // Type without executing
                                                // (parity with the history
                                                // list's Enter).
                                                td.instance.write(line.as_bytes());
                                                td.instance.history_nav = None;
                                                self.history_menu_just_closed
                                                    .insert(tab.clone(), true);
                                            }
                                        }
                                    }
                                }
                                if add_cmd_clicked {
                                    self.fav_cmd_dialog = Some((fid, String::new()));
                                    self.fav_cmd_just_opened = true;
                                }
                                if rename_clicked {
                                    self.fav_name_dialog =
                                        Some((Some(fid), folders[fidx].1.clone()));
                                }
                                if delete_clicked && !is_default_folder {
                                    self.fav_delete_confirm = Some((fid, folders[fidx].1.clone()));
                                    self.fav_del_just_opened = true;
                                }
                            }

                            // folder drag start
                            let handle_rect = egui::Rect::from_min_size(
                                egui::pos2(row_rect.min.x, row_rect.min.y),
                                egui::vec2(20.0, row_h),
                            );
                            if handle_rect.contains(hover_pos)
                                && pointer_down
                                && self.fav_drag_src.is_none()
                                && self.fav_item_drag.is_none()
                            {
                                self.fav_drag_src = Some(fidx);
                            }
                        }
                        // Advance to the next row (this was lost with the
                        // dead inline-item block that used to share it —
                        // every folder painted at the SAME y).
                        y += row_h;
                    }

                    // Folder drag: insertion line + drop
                    if let Some(src) = self.fav_drag_src {
                        if pointer_released {
                            if let Some((dst_idx, after)) = self.fav_drag_dst.take() {
                                let n = folders.len();
                                let dst = drag_drop_destination(src, dst_idx, after, n);
                                if src != dst {
                                    let mut ids: Vec<i64> =
                                        folders.iter().map(|(id, _)| *id).collect();
                                    let moved = ids.remove(src);
                                    ids.insert(dst, moved);
                                    self.history_db.fav_folder_reorder(&ids);
                                    self.fav_folders = self.history_db.fav_folders();
                                }
                            }
                            self.fav_drag_src = None;
                        } else {
                            let mut target: Option<(usize, bool)> = None;
                            for (fi, (_, frect)) in folder_row_index.iter().enumerate() {
                                if frect.contains(hover_pos) && fi != src {
                                    target = Some((fi, hover_pos.y > frect.center().y));
                                    break;
                                }
                            }
                            self.fav_drag_dst = target;
                            if let Some((ti, after)) = target {
                                if let Some((_, trect)) = folder_row_index.get(ti) {
                                    let iy = if after { trect.max.y } else { trect.min.y };
                                    ui.painter().line_segment(
                                        [
                                            egui::pos2(fx0 + 4.0, iy),
                                            egui::pos2(fx0 + col_w - 4.0, iy),
                                        ],
                                        egui::Stroke::new(2.0, app.accent.to_egui()),
                                    );
                                }
                            }
                        }
                    }

                    // ---- THIRD column: the selected folder's commands ----
                    // Rendered INSIDE the same window, flush against the
                    // folder column, same row style (this replaces the
                    // old floating-popup submenu). Items are read LIVE
                    // from the DB so deletes/reorders reflect instantly.
                    // The column is a permanent fixture: always rendered,
                    // even for an empty favorites list (empty column = no
                    // rows, just the background).
                    if show_favs {
                        let sub_x0 = fx0 + col_w;
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(sub_x0, frame_rect.min.y),
                                egui::vec2(sub_w, rows_h + footer_h),
                            ),
                            0.0,
                            menu_bg,
                        );
                        if let Some((fid, _, _, kb_sel)) = self.fav_submenu.clone() {
                            // Row-addressed: duplicate commands are distinct
                            // rows; delete/move/reorder go by rowid.
                            let with_ids = self.history_db.fav_items_with_ids(fid);
                            let items: Vec<String> =
                                with_ids.iter().map(|(_, c)| c.clone()).collect();
                            let row_ids: Vec<i64> = with_ids.iter().map(|(rid, _)| *rid).collect();
                            let mut send_cmd: Option<String> = None;
                            let mut remove_idx: Option<usize> = None;
                            let pointer = ui.input(|i| i.pointer.hover_pos());
                            // Sub-column scroll (wheel + scrollbar), same
                            // style as the main list.
                            let sub_visible_rows = (rows_h / row_h) as usize;
                            let sub_max_scroll = items.len().saturating_sub(sub_visible_rows);
                            let sub_scroll_id = egui::Id::new(("hist_subcol_scroll", tab.as_str()));
                            let mut sub_scroll: usize = ctx
                                .memory(|m| m.data.get_temp(sub_scroll_id).unwrap_or(0))
                                .min(sub_max_scroll);
                            let mut y = frame_rect.min.y - sub_scroll as f32 * row_h;
                            for (idx, cmd) in items.iter().enumerate() {
                                if y + row_h < frame_rect.min.y {
                                    y += row_h;
                                    continue;
                                }
                                if y >= frame_rect.min.y + rows_h {
                                    break;
                                }
                                let row = egui::Rect::from_min_size(
                                    egui::pos2(sub_x0, y),
                                    egui::vec2(sub_w, row_h),
                                );
                                let row_hover = row.contains(hover_pos);
                                if idx % 2 == 1 {
                                    ui.painter().rect_filled(row, 0.0, menu_alt);
                                }
                                // Drag handle (leading edge): DOTS_SIX_VERTICAL.
                                let handle_g = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        egui_phosphor::regular::DOTS_SIX_VERTICAL.to_string(),
                                        egui::FontId::proportional(11.0),
                                        weak,
                                    )
                                });
                                ui.painter().galley(
                                    egui::pos2(
                                        row.min.x + 4.0,
                                        row.center().y - handle_g.size().y / 2.0,
                                    ),
                                    handle_g,
                                    weak,
                                );
                                // Selection tint (hover or keyboard).
                                let is_kb = kb_sel.is_some_and(|k| k == idx);
                                if row_hover || is_kb {
                                    ui.painter().rect_filled(
                                        row,
                                        0.0,
                                        egui::Color32::from_rgba_unmultiplied(
                                            sel_bg.r(),
                                            sel_bg.g(),
                                            sel_bg.b(),
                                            90,
                                        ),
                                    );
                                }
                                let g = ui.fonts(|f| {
                                    f.layout_no_wrap(
                                        cmd.clone(),
                                        egui::FontId::monospace(font_size * 0.9),
                                        menu_fg,
                                    )
                                });
                                ui.set_clip_rect(egui::Rect::from_min_max(
                                    row.min,
                                    egui::pos2(row.max.x - 26.0, row.max.y),
                                ));
                                ui.painter().galley(
                                    egui::pos2(row.min.x + 24.0, row.center().y - g.size().y / 2.0),
                                    g,
                                    menu_fg,
                                );
                                ui.set_clip_rect(frame_rect.expand(1.0));
                                let rresp = ui.interact(
                                    row,
                                    egui::Id::new(("fav_sub_row", tab.as_str(), fid, idx)),
                                    egui::Sense::click_and_drag(),
                                );
                                if rresp.clicked() {
                                    send_cmd = Some(cmd.clone());
                                }
                                // Delete (TRASH icon) on hover/selection.
                                if row_hover || is_kb {
                                    let dcol = weak;
                                    let dg = ui.fonts(|f| {
                                        f.layout_no_wrap(
                                            egui_phosphor::regular::TRASH.to_string(),
                                            egui::FontId::proportional(13.0),
                                            dcol,
                                        )
                                    });
                                    let dw = dg.size().x;
                                    let drect = egui::Rect::from_min_max(
                                        egui::pos2(row.max.x - 8.0 - dw, row.center().y - 8.0),
                                        egui::pos2(row.max.x - 8.0, row.center().y + 8.0),
                                    );
                                    let dresp = ui.interact(
                                        drect,
                                        egui::Id::new(("fav_sub_del", tab.as_str(), fid, idx)),
                                        egui::Sense::click(),
                                    );
                                    let dhot = if dresp.contains_pointer() {
                                        app.danger.to_egui()
                                    } else {
                                        weak
                                    };
                                    let dg2 = ui.fonts(|f| {
                                        f.layout_no_wrap(
                                            egui_phosphor::regular::TRASH.to_string(),
                                            egui::FontId::proportional(13.0),
                                            dhot,
                                        )
                                    });
                                    ui.painter().galley(
                                        drect.center() - dg2.size() / 2.0,
                                        dg2,
                                        dhot,
                                    );
                                    let _ = dg;
                                    if dresp.clicked() {
                                        remove_idx = Some(idx);
                                    }
                                }
                                // Drag start: ONLY from the handle zone.
                                let press = ui.input(|i| i.pointer.press_origin());
                                let in_handle = press
                                    .is_some_and(|pp| row.contains(pp) && pp.x < row.min.x + 22.0);
                                if in_handle
                                    && ui.input(|i| i.pointer.primary_down())
                                    && self.fav_item_drag.is_none()
                                {
                                    self.fav_item_drag = Some((fid, idx));
                                }
                                y += row_h;
                            }
                            // Sub-column scrollbar + wheel.
                            if items.len() > sub_visible_rows {
                                let sb_track = egui::Rect::from_min_max(
                                    egui::pos2(sub_x0 + sub_w - 6.0, frame_rect.min.y),
                                    egui::pos2(sub_x0 + sub_w, frame_rect.min.y + rows_h),
                                );
                                let thumb_h = (sub_visible_rows as f32 / items.len() as f32
                                    * rows_h)
                                    .max(16.0);
                                let scrollable = (rows_h - thumb_h).max(1.0);
                                let thumb_y = frame_rect.min.y
                                    + scrollable * (sub_scroll as f32 / sub_max_scroll as f32);
                                let sb_col = egui::Color32::from_rgba_unmultiplied(
                                    weak.r(),
                                    weak.g(),
                                    weak.b(),
                                    110,
                                );
                                let track_col = egui::Color32::from_rgba_unmultiplied(
                                    weak.r(),
                                    weak.g(),
                                    weak.b(),
                                    40,
                                );
                                ui.painter().rect_filled(sb_track, 0.0, track_col);
                                ui.painter().rect_filled(
                                    egui::Rect::from_min_size(
                                        egui::pos2(sb_track.min.x, thumb_y),
                                        egui::vec2(sb_track.width(), thumb_h),
                                    ),
                                    0.0,
                                    sb_col,
                                );
                                let sb_resp = ui.interact(
                                    sb_track,
                                    egui::Id::new(("fav_sub_sb", tab.as_str())),
                                    egui::Sense::click_and_drag(),
                                );
                                if sb_resp.dragged() {
                                    let dy = ui.input(|i| i.pointer.delta().y);
                                    let lines = dy * sub_max_scroll as f32 / scrollable;
                                    sub_scroll = (sub_scroll as f32 + lines)
                                        .round()
                                        .clamp(0.0, sub_max_scroll as f32)
                                        as usize;
                                    ctx.memory_mut(|m| {
                                        m.data.insert_temp(sub_scroll_id, sub_scroll)
                                    });
                                }
                            }
                            let sub_wheel = ui.input(|i| {
                                i.events
                                    .iter()
                                    .try_fold(0.0f32, |acc, e| match e {
                                        egui::Event::MouseWheel { delta, .. } => {
                                            Some(acc + delta.y)
                                        }
                                        _ => Some(acc),
                                    })
                                    .unwrap_or(0.0)
                            });
                            let sub_rect_wheel = egui::Rect::from_min_size(
                                egui::pos2(sub_x0, frame_rect.min.y),
                                egui::vec2(sub_w, rows_h),
                            );
                            if sub_rect_wheel.contains(hover_pos) && sub_wheel != 0.0 {
                                sub_scroll = if sub_wheel > 0.0 {
                                    sub_scroll.saturating_sub(1)
                                } else {
                                    (sub_scroll + 1).min(sub_max_scroll)
                                };
                                ctx.memory_mut(|m| m.data.insert_temp(sub_scroll_id, sub_scroll));
                            }

                            // In-flight drag: insertion line + drop.
                            if let Some((src_fid, src_idx)) = self.fav_item_drag {
                                let released = ui.input(|i| i.pointer.any_released());
                                if released {
                                    if let Some((dst_fid, dst_idx, after)) =
                                        self.fav_item_drop.take()
                                    {
                                        if dst_fid == src_fid {
                                            if src_idx < row_ids.len() {
                                                let dst = drag_drop_destination(
                                                    src_idx,
                                                    dst_idx,
                                                    after,
                                                    row_ids.len(),
                                                );
                                                if src_idx != dst {
                                                    let mut ids2 = row_ids.clone();
                                                    let moved = ids2.remove(src_idx);
                                                    ids2.insert(dst, moved);
                                                    self.history_db
                                                        .fav_item_reorder_rows(src_fid, &ids2);
                                                }
                                            }
                                        } else if let Some(cmd) = items.get(src_idx) {
                                            // Cross-folder drop COPIES (adds)
                                            // the command to the target —
                                            // the source keeps its copy.
                                            self.history_db.fav_add_to(dst_fid, cmd);
                                        }
                                        self.fav_folders = self.history_db.fav_folders();
                                    }
                                    self.fav_item_drag = None;
                                } else if let Some(pp) = pointer {
                                    let sub_rect = egui::Rect::from_min_size(
                                        egui::pos2(sub_x0, frame_rect.min.y),
                                        egui::vec2(sub_w, rows_h),
                                    );
                                    if sub_rect.contains(pp) {
                                        let idx = (((pp.y - frame_rect.min.y) / row_h) as usize)
                                            .min(items.len().saturating_sub(1));
                                        let after = pp.y
                                            > frame_rect.min.y + idx as f32 * row_h + row_h / 2.0;
                                        self.fav_item_drop = Some((fid, idx, after));
                                        let iy = if after {
                                            frame_rect.min.y + (idx + 1) as f32 * row_h
                                        } else {
                                            frame_rect.min.y + idx as f32 * row_h
                                        };
                                        ui.painter().line_segment(
                                            [
                                                egui::pos2(sub_x0 + 2.0, iy),
                                                egui::pos2(sub_x0 + sub_w - 2.0, iy),
                                            ],
                                            egui::Stroke::new(2.0, sel_bg),
                                        );
                                    } else {
                                        self.fav_item_drop = None;
                                    }
                                }
                            }
                            // Apply row actions AFTER painting (borrow
                            // discipline): send or live-delete.
                            if let Some(cmd) = send_cmd {
                                if !open_snippet_fill_fields(
                                    &mut self.terminals,
                                    &mut self.fav_submenu,
                                    &mut self.fav_sub_focused,
                                    &mut self.history_menu_just_closed,
                                    &mut self.snippet_fill,
                                    &mut self.snippet_fill_just_opened,
                                    &tab,
                                    cmd.clone(),
                                ) {
                                    if let Some(td) = self.terminals.get_mut(&tab) {
                                        // Type without executing (parity
                                        // with the history list).
                                        td.instance.write(cmd.as_bytes());
                                        td.instance.history_nav = None;
                                    }
                                }
                                self.history_menu_just_closed.insert(tab.clone(), true);
                                self.fav_submenu = None;
                            }
                            if let Some(idx) = remove_idx {
                                if let Some(rid) = row_ids.get(idx) {
                                    self.history_db.fav_item_remove_row(*rid);
                                    self.fav_folders = self.history_db.fav_folders();
                                    // The renderer re-reads the DB live
                                    // every frame, so the row vanishes on
                                    // its own; replacing the in-Flight
                                    // snapshot MID-AREA (old behavior)
                                    // caused a one-frame clip/paint
                                    // mismatch — the black flash. Only
                                    // close the column if the folder is
                                    // now empty, and clamp the keyboard
                                    // selection.
                                    let fresh = self.history_db.fav_items_with_ids(fid);
                                    // Keep the column open (permanent
                                    // fixture) — an emptied folder now
                                    // shows an empty command list.
                                    let sel = self
                                        .fav_submenu
                                        .as_ref()
                                        .and_then(|(_, _, _, s)| *s)
                                        .unwrap_or(0)
                                        .min(fresh.len().saturating_sub(1));
                                    self.fav_submenu = Some((
                                        fid,
                                        egui::Pos2::ZERO,
                                        fresh.iter().map(|(_, c)| c.clone()).collect(),
                                        Some(sel),
                                    ));
                                }
                            }
                            let _ = pointer;
                        }
                    }

                    // Folder-column scrollbar (same style as the main
                    // list's): shows only when folders overflow 10 rows.
                    let folder_count = folders.len();
                    let col_visible_rows = ((rows_h - 24.0).max(row_h) / row_h) as usize;
                    if folder_count > col_visible_rows {
                        let sb_track = egui::Rect::from_min_max(
                            egui::pos2(fx0 + col_w - 6.0, frame_rect.min.y),
                            egui::pos2(fx0 + col_w, frame_rect.min.y + rows_h),
                        );
                        let thumb_h =
                            (col_visible_rows as f32 / folder_count as f32 * rows_h).max(16.0);
                        let scrollable = (rows_h - thumb_h).max(1.0);
                        let thumb_y = frame_rect.min.y
                            + scrollable * (col_scroll as f32 / max_col_scroll as f32);
                        let sb_col = egui::Color32::from_rgba_unmultiplied(
                            weak.r(),
                            weak.g(),
                            weak.b(),
                            110,
                        );
                        let track_col =
                            egui::Color32::from_rgba_unmultiplied(weak.r(), weak.g(), weak.b(), 40);
                        ui.painter().rect_filled(sb_track, 0.0, track_col);
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(sb_track.min.x, thumb_y),
                                egui::vec2(sb_track.width(), thumb_h),
                            ),
                            0.0,
                            sb_col,
                        );
                        let sb_resp = ui.interact(
                            sb_track,
                            egui::Id::new(("fav_col_sb", tab.as_str())),
                            egui::Sense::click_and_drag(),
                        );
                        if sb_resp.dragged() {
                            let dy = ui.input(|i| i.pointer.delta().y);
                            let lines = dy * max_col_scroll as f32 / scrollable;
                            col_scroll = (col_scroll as f32 + lines)
                                .round()
                                .clamp(0.0, max_col_scroll as f32)
                                as usize;
                            ctx.memory_mut(|m| m.data.insert_temp(col_scroll_id, col_scroll));
                        }
                    }

                    // Column wheel scroll
                    let col_rect = egui::Rect::from_min_size(
                        egui::pos2(fx0, frame_rect.min.y),
                        egui::vec2(col_w, rows_h),
                    );
                    let wheel = ui.input(|i| {
                        i.events
                            .iter()
                            .try_fold(0.0f32, |acc, e| match e {
                                egui::Event::MouseWheel { delta, .. } => Some(acc + delta.y),
                                _ => Some(acc),
                            })
                            .unwrap_or(0.0)
                    });
                    if col_rect.contains(hover_pos) && wheel != 0.0 {
                        col_scroll = if wheel > 0.0 {
                            col_scroll.saturating_sub(1)
                        } else {
                            (col_scroll + 1).min(max_col_scroll)
                        };
                        ctx.memory_mut(|m| m.data.insert_temp(col_scroll_id, col_scroll));
                    }
                }

                // ---- Shared footer ----
                let footer = egui::Rect::from_min_size(
                    egui::pos2(frame_rect.min.x, frame_rect.min.y + rows_h),
                    egui::vec2(total_w, footer_h),
                );
                ui.painter().rect_filled(footer, 0.0, menu_bg);
                ui.painter()
                    .hline(footer.x_range(), footer.min.y, (1.0, border));
                ui.painter().text(
                    egui::pos2(footer.min.x + 8.0, footer.center().y),
                    egui::Align2::LEFT_CENTER,
                    format!("{}{}", total, self.texts.terminal.history_count),
                    egui::FontId::proportional(11.0),
                    weak,
                );
                // Favorites column count, left-aligned in its own column.
                if show_favs {
                    let fx0 = frame_rect.min.x + list_w;
                    ui.painter().text(
                        egui::pos2(fx0 + 8.0, footer.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!(
                            "{}{}",
                            self.fav_folders.len(),
                            self.texts.terminal.fav_folder_count
                        ),
                        egui::FontId::proportional(11.0),
                        weak,
                    );
                    // Submenu (command) column count, when that column is
                    // open (its x starts at the favorites column's right).
                    if let Some((fid, _, _, _)) = self.fav_submenu.clone() {
                        let sub_x0 = fx0 + fav_w;
                        let item_count = self.history_db.fav_items(fid).len();
                        ui.painter().text(
                            egui::pos2(sub_x0 + 8.0, footer.center().y),
                            egui::Align2::LEFT_CENTER,
                            format!("{}{}", item_count, self.texts.terminal.fav_item_count),
                            egui::FontId::proportional(11.0),
                            weak,
                        );
                    }
                }
                // X close pinned at the far right.
                let x_rect = egui::Rect::from_min_size(
                    egui::pos2(footer.max.x - 8.0 - 18.0, footer.center().y - 9.0),
                    egui::vec2(18.0, 18.0),
                );
                let xresp = ui.interact(
                    x_rect,
                    egui::Id::new(("hist_close", tab.as_str())),
                    egui::Sense::click(),
                );
                let xcol = if xresp.hovered() { menu_fg } else { weak };
                let xg = ui.fonts(|f| {
                    f.layout_no_wrap(
                        egui_phosphor::regular::X.to_string(),
                        egui::FontId::proportional(11.0),
                        xcol,
                    )
                });
                ui.painter()
                    .galley(x_rect.center() - xg.size() / 2.0, xg, xcol);
                if xresp.clicked() {
                    close_clicked = true;
                }
                // The clear-history and clear-favorites buttons were moved
                // into their respective column headers (history column and
                // favorites column), so the footer now only holds the
                // count on the left and the X close on the right.

                // Lines painted LAST: rows, alternating bands,
                // scrollbars and the footer can never cover them.
                if show_favs {
                    let fx0 = frame_rect.min.x + list_w;
                    ui.painter().line_segment(
                        [
                            egui::pos2(fx0, frame_rect.min.y),
                            egui::pos2(fx0, frame_rect.min.y + rows_h),
                        ],
                        (1.0, border),
                    );
                }
                // Column separators: history|favorites and favorites|
                // commands — vertical hairlines matching the outer border.
                if show_favs {
                    ui.painter().line_segment(
                        [
                            egui::pos2(frame_rect.min.x + list_w, frame_rect.min.y),
                            egui::pos2(
                                frame_rect.min.x + list_w,
                                frame_rect.min.y + rows_h + footer_h,
                            ),
                        ],
                        egui::Stroke::new(1.0, border),
                    );
                    // favorites|commands separator: always drawn — the
                    // command column is a permanent fixture now.
                    let x = frame_rect.min.x + list_w + fav_w;
                    ui.painter().line_segment(
                        [
                            egui::pos2(x, frame_rect.min.y),
                            egui::pos2(x, frame_rect.min.y + rows_h + footer_h),
                        ],
                        egui::Stroke::new(1.0, border),
                    );
                }
                ui.painter().rect_stroke(
                    frame_rect,
                    0.0,
                    egui::Stroke::new(1.0, border),
                    egui::StrokeKind::Middle,
                );
            });

        ctx.memory_mut(|m| {
            m.data.insert_temp(scroll_id, scroll);
            m.data.insert_temp(fav_scroll_id, fav_scroll);
        });

        // Row click = select + confirm (same as Enter).
        if let Some(i) = entry_clicked {
            if let Some(td) = self.terminals.get_mut(&tab) {
                if let Some(nav) = td.instance.history_nav.as_mut() {
                    nav.selected = i;
                }
            }
            self.confirm_history_entry(&tab);
            return;
        }
        // Favorite row click: send the command.
        // Row action: add to global favorites.
        if let Some(i) = row_fav_clicked {
            let cmd = self
                .terminals
                .get(&tab)
                .and_then(|td| td.instance.history_nav.as_ref())
                .and_then(|nav| nav.entries.get(i).cloned());
            if let Some(cmd) = cmd {
                self.history_db.fav_add(&cmd);
                if let Some(td) = self.terminals.get_mut(&tab) {
                    if let Some(nav) = td.instance.history_nav.as_mut() {
                        nav.favorites = self.history_db.fav_all();
                    }
                }
            }
        }
        // Row action: delete one history entry.
        if let Some(i) = row_del_clicked {
            self.history_db.remove_entry(&tab, i);
            if let Some(td) = self.terminals.get_mut(&tab) {
                if let Some(nav) = td.instance.history_nav.as_mut() {
                    if i < nav.entries.len() {
                        nav.entries.remove(i);
                    }
                    if !nav.entries.is_empty() {
                        nav.selected = nav.selected.min(nav.entries.len() - 1);
                    }
                }
            }
        }
        // Favorite row delete.
        // Footer: clear global favorites (same confirm dialog as settings).
        if clear_favs_clicked {
            self.show_clear_favorites_confirm = true;
            self.fav_clear_just_opened = true;
        }
        if clear_history_clicked {
            self.history_clear_confirm = Some(tab.clone());
            self.hist_clear_just_opened = true;
        }
        if close_clicked {
            self.close_history_menu(&tab);
        }
    }
    /// Confirmation dialog for clearing a terminal's command history
    /// (from the history-menu footer "clear" button). Styled like the
    /// password popups: compact metrics, fixed sizes, danger confirm.
    pub(crate) fn render_history_clear_confirm(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.history_clear_confirm.clone() else {
            return;
        };
        // Rising edge: start on the safe side (CANCEL).
        if std::mem::take(&mut self.hist_clear_just_opened) {
            self.dialog_kb_confirm = false;
        }
        // Unified protocol, BEFORE the Modal.
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirmed = keys.confirm;
        let mut cancelled = keys.cancel;
        if keys.close {
            self.history_clear_confirm = None;
            return;
        }
        let mut kb = self.dialog_kb_confirm;
        let title = self.texts.stats.clear_history_title.clone();
        let body = self.texts.stats.clear_history_body.clone();
        let confirm_txt = self.texts.theme_editor.dialog_confirm.clone();
        let cancel_txt = self.texts.theme_editor.cancel.clone();
        let danger = self.active_theme.app.danger.to_egui();
        let text_col = self.active_theme.app.text.to_egui();
        // Fixed-size dialog: the window is pinned to 360x300 so its
        // available_height is a finite constant (auto-sized windows feed
        // infinite height into the bottom-fill math, which both started
        // the dialog huge and made it grow every frame). The button row
        // lives in a bottom panel: 20px bottom margin + 24px row,
        // horizontally centered.
        let dlg_w = 360.0f32;
        let dlg_h = 96.0f32;
        let center = ctx.screen_rect().center();
        let pos = egui::pos2(center.x - dlg_w / 2.0, center.y - dlg_h / 2.0);
        let _ = pos;
        let modal = egui::Modal::new(egui::Id::new("hist_clear_confirm"))
            .frame(egui::Frame::window(&ctx.style()).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                ui.set_min_size(egui::vec2(dlg_w, dlg_h));
                ui.heading(title);
                ui.style_mut().spacing.item_spacing = egui::vec2(6.0, 4.0);
                ui.style_mut().spacing.interact_size.y = 24.0;
                ui.style_mut().spacing.button_padding = egui::vec2(10.0, 3.0);
                // Button row pinned to the bottom: 20px margin + 24px row.
                egui::TopBottomPanel::bottom("hist_clear_confirm_footer")
                    .frame(egui::Frame::new())
                    .exact_height(44.0)
                    .show_inside(ui, |ui| {
                        ui.add_space(20.0);
                        let (c, x) = Self::dialog_button_row(
                            ui,
                            &mut kb,
                            egui::Id::new("hist_clear_confirm_btn"),
                            egui::Id::new("hist_clear_cancel_btn"),
                            &confirm_txt,
                            &cancel_txt,
                        );
                        confirmed |= c;
                        cancelled |= x;
                    });
                // Body fills the remaining central area (top-aligned).
                ui.label(egui::RichText::new(body).size(13.0).color(text_col));
                let _ = danger;
            });
        // Backdrop click cancels.
        if modal.backdrop_response.clicked() {
            cancelled = true;
        }
        if confirmed {
            self.history_db.clear(&tab);
            if let Some(td) = self.terminals.get_mut(&tab) {
                td.instance.history_nav = None;
            }
            self.history_clear_confirm = None;
        } else if cancelled {
            self.history_clear_confirm = None;
        }
    }
    /// Draw the two-button row for any [`dialog_keys`]-driven dialog.
    /// Pure rendering + click detection: the keyboard cursor lives in
    /// `dialog_kb_confirm` (toggled by `dialog_keys` BEFORE the dialog
    /// is created), and the selected side is highlighted so the user
    /// sees which button Enter will activate. Returns
    /// (confirm_clicked, cancel_clicked).
    pub(crate) fn dialog_button_row(
        ui: &mut egui::Ui,
        kb_confirm: &mut bool,
        confirm_id: egui::Id,
        cancel_id: egui::Id,
        confirm_label: &str,
        cancel_label: &str,
    ) -> (bool, bool) {
        let draw = |ui: &mut egui::Ui, id: egui::Id, label: &str, is_selected: bool| -> bool {
            let font = egui::FontId::proportional(13.0);
            let galley =
                ui.fonts(|f| f.layout_no_wrap(label.to_string(), font, ui.visuals().text_color()));
            let size = egui::vec2(72.0, 22.0);
            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
            let resp = ui.interact(rect, id, egui::Sense::click());
            // Visuals: selected button uses a brighter border so the
            // user sees which one Enter will activate.
            let base = &ui.style().visuals.widgets.inactive;
            let hovered = &ui.style().visuals.widgets.hovered;
            let (fill, stroke) = if is_selected {
                (hovered.weak_bg_fill, hovered.bg_stroke)
            } else if resp.contains_pointer() {
                (hovered.weak_bg_fill, base.bg_stroke)
            } else {
                (base.weak_bg_fill, base.bg_stroke)
            };
            let corner = if is_selected { 3.0 } else { 0.0 };
            ui.painter().rect_filled(rect, corner, fill);
            ui.painter()
                .rect_stroke(rect, corner, stroke, egui::StrokeKind::Middle);
            ui.painter().galley(
                rect.center() - galley.size() / 2.0,
                galley,
                ui.visuals().text_color(),
            );
            resp.clicked()
        };

        let (c, x) = ui
            .horizontal(|ui| {
                let c = draw(ui, confirm_id, confirm_label, *kb_confirm);
                ui.add_space(8.0);
                let x = draw(ui, cancel_id, cancel_label, !*kb_confirm);
                (c, x)
            })
            .inner;
        (c, x)
    }
    pub(crate) fn render_fav_name_dialog(&mut self, ctx: &egui::Context) {
        let Some((folder_id, _)) = self.fav_name_dialog.clone() else {
            return;
        };
        // Rising edge: a freshly-opened dialog starts on the safe side.
        if std::mem::take(&mut self.fav_name_just_opened) {
            self.dialog_kb_confirm = false;
        }
        // Unified protocol, BEFORE the Modal so nothing swallows the keys.
        // Enter in an input dialog means CONFIRM regardless of cursor.
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirm = keys.enter || keys.confirm;
        let mut cancel = keys.cancel;
        if keys.close {
            self.fav_name_dialog = None;
            return;
        }
        let title = match folder_id {
            Some(_) => self.texts.terminal.fav_rename_title.clone(),
            None => self.texts.terminal.fav_new_title.clone(),
        };
        let confirm_id = egui::Id::new("fav_name_confirm_btn");
        let cancel_id = egui::Id::new("fav_name_cancel_btn");
        egui::Modal::new(egui::Id::new("fav_name_dialog"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.set_min_width(280.0);
                ui.heading(title);
                ui.add_space(4.0);
                ui.label(self.texts.terminal.fav_name_label.clone());
                let resp = ui.text_edit_singleline(&mut self.fav_name_dialog.as_mut().unwrap().1);
                // Focus the input every frame the modal is open, so
                // external focus changes (terminal pointer focus, egui's
                // first-frame nav, etc.) cannot pull it away. This is
                // cheaper and more robust than the previous
                // "first frame seeds, then surrender" dance.
                resp.request_focus();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let (c, x) = Self::dialog_button_row(
                        ui,
                        &mut self.dialog_kb_confirm,
                        confirm_id,
                        cancel_id,
                        &self.texts.theme_editor.dialog_confirm.clone(),
                        &self.texts.theme_editor.cancel.clone(),
                    );
                    confirm |= c;
                    cancel |= x;
                });
            });
        if confirm || cancel {
            if confirm {
                if let Some((fid, name)) = self.fav_name_dialog.take() {
                    let name = name.trim().to_string();
                    if name.is_empty() {
                        self.fav_name_dialog = None;
                    } else {
                        match fid {
                            Some(id) => {
                                self.history_db.fav_folder_rename(id, &name);
                            }
                            None => {
                                self.history_db.fav_folder_create(&name);
                            }
                        }
                        self.fav_folders = self.history_db.fav_folders();
                    }
                }
            } else {
                self.fav_name_dialog = None;
            }
        }
    }
    /// Add-command dialog: text input, Enter confirms, Esc cancels.
    /// The command is appended to the folder on confirm.
    pub(crate) fn render_fav_cmd_dialog(&mut self, ctx: &egui::Context) {
        if self.fav_cmd_dialog.is_none() {
            return;
        }
        // Rising edge: a freshly-opened dialog starts on the safe side.
        if std::mem::take(&mut self.fav_cmd_just_opened) {
            self.dialog_kb_confirm = false;
        }
        // Unified protocol, BEFORE the Modal. Enter in an input dialog
        // means CONFIRM regardless of cursor.
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirm = keys.enter || keys.confirm;
        let mut cancel = keys.cancel;
        if keys.close {
            self.fav_cmd_dialog = None;
            return;
        }
        let confirm_id = egui::Id::new("fav_cmd_confirm_btn");
        let cancel_id = egui::Id::new("fav_cmd_cancel_btn");
        egui::Modal::new(egui::Id::new("fav_cmd_dialog"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.heading(self.texts.terminal.fav_cmd_dialog_title.clone());
                ui.add_space(4.0);
                ui.label(self.texts.terminal.fav_cmd_dialog_label.clone());
                let resp = ui.text_edit_singleline(&mut self.fav_cmd_dialog.as_mut().unwrap().1);
                // Keep the input focused every frame the modal is open —
                // cheaper and more robust than first-frame seeding.
                resp.request_focus();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let (c, x) = Self::dialog_button_row(
                        ui,
                        &mut self.dialog_kb_confirm,
                        confirm_id,
                        cancel_id,
                        &self.texts.theme_editor.dialog_confirm.clone(),
                        &self.texts.theme_editor.cancel.clone(),
                    );
                    confirm |= c;
                    cancel |= x;
                });
            });
        if confirm || cancel {
            if confirm {
                if let Some((fid, cmd)) = self.fav_cmd_dialog.take() {
                    let cmd = cmd.trim().to_string();
                    if cmd.is_empty() {
                        self.fav_cmd_dialog = None;
                    } else {
                        self.history_db.fav_add_to(fid, &cmd);
                        self.fav_folders = self.history_db.fav_folders();
                    }
                }
            } else {
                self.fav_cmd_dialog = None;
            }
        }
    }
    /// Delete-folder confirmation: warns the folder AND all its commands
    /// go away.
    pub(crate) fn render_fav_delete_confirm(&mut self, ctx: &egui::Context) {
        let Some((fid, name)) = self.fav_delete_confirm.clone() else {
            return;
        };
        // Rising edge: a freshly-opened dialog starts on the safe side
        // (CANCEL - a stray Enter must not delete the folder).
        if std::mem::take(&mut self.fav_del_just_opened) {
            self.dialog_kb_confirm = false;
        }
        // Unified protocol, BEFORE the Modal: Enter activates whichever
        // side the cursor is on.
        let keys = dialog_keys(ctx, &mut self.dialog_kb_confirm, true);
        let mut confirm = keys.confirm;
        let mut cancel = keys.cancel;
        if keys.close {
            self.fav_delete_confirm = None;
            return;
        }
        let confirm_id = egui::Id::new("fav_del_confirm_btn");
        let cancel_id = egui::Id::new("fav_del_cancel_btn");
        let modal = egui::Modal::new(egui::Id::new("fav_delete_confirm"))
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.heading(self.texts.terminal.fav_delete_title.clone());
                ui.add_space(4.0);
                ui.strong(format!("\u{201c}{name}\u{201d}"));
                ui.label(self.texts.terminal.fav_delete_body.clone());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let (c, x) = Self::dialog_button_row(
                        ui,
                        &mut self.dialog_kb_confirm,
                        confirm_id,
                        cancel_id,
                        &self.texts.theme_editor.dialog_confirm.clone(),
                        &self.texts.theme_editor.cancel.clone(),
                    );
                    confirm |= c;
                    cancel |= x;
                });
            });
        // Clicking the backdrop (outside the dialog) cancels — the modal
        // blocks every other interaction anyway.
        if modal.backdrop_response.clicked() {
            cancel = true;
        }
        let _ = modal.is_top_modal;
        if confirm || cancel {
            if confirm {
                self.history_db.fav_folder_delete(fid);
                // Closing the folder also closes its floating submenu.
                self.fav_submenu = None;
                self.fav_folders = self.history_db.fav_folders();
            }
            self.fav_delete_confirm = None;
        }
    }
    /// Single funnel for closing a terminal's history menu (Esc, confirm,
    /// click): removes the menu AND sets the one-frame latch that stops
    /// the confirming keypress's own Text event from re-opening it via
    /// the auto-matcher.
    pub(crate) fn close_history_menu(&mut self, tab: &str) {
        if let Some(td) = self.terminals.get_mut(tab) {
            td.instance.history_nav = None;
        }
        // Drop the transient column state WITH the menu (the snapshot in
        // menu_cursors already captured what to restore); leaving it set
        // made restored sessions drive two lists at once.
        self.fav_submenu = None;
        self.fav_sub_focused = false;
        self.history_menu_just_closed.insert(tab.to_string(), true);
    }
    /// Confirm (send) the selected entry of a terminal's history menu.
    pub(crate) fn confirm_history_entry(&mut self, tab: &str) {
        let selected = self.terminals.get_mut(tab).and_then(|td| {
            let nav = td.instance.history_nav.take()?;
            let command = nav.entries.get(nav.selected)?.clone();
            if let Some(word) = nav.auto_word.clone() {
                let del = vec![0x7fu8; word.chars().count()];
                td.instance.write(&del);
            }
            td.instance.write(command.as_bytes());
            Some(command)
        });
        // Single-frame latch: the Space/Enter keypress that confirmed
        // produces its own Text event this frame — the matcher must not
        // act on it. From the next frame on, ONLY a real key edit (typed
        // / deleted char) can re-open matching.
        self.history_menu_just_closed.insert(tab.to_string(), true);
        // Confirming rewrites the terminal line (delete word + send the
        // full command): the pending buffer no longer matches the grid.
        self.auto_match_pending.remove(tab);
        if let Some(command) = selected {
            let host = self
                .terminals
                .get(tab)
                .and_then(|td| td.host.as_ref())
                .map(|h| h.addr.clone())
                .unwrap_or_default();
            self.history_db.add(tab, &command, &host);
        }
    }
}

use alacritty_terminal::index::Point as TerminalGridPoint;
use alacritty_terminal::term::cell;
use alacritty_terminal::term::point_to_viewport;
use alacritty_terminal::term::TermMode;
use alacritty_terminal::vte::ansi::{Color, NamedColor};
use egui::epaint::RectShape;
use egui::Color32;
use egui::Modifiers;
use egui::MouseWheelUnit;
use egui::Shape;
use egui::Widget;
use egui::{Align2, Painter, Pos2, Rect, Response, Stroke, Vec2};
use egui::{CornerRadius, Key};
use egui::{Id, PointerButton};

use crate::backend::BackendCommand;
use crate::backend::TerminalBackend;
use crate::backend::{LinkAction, MouseButton, SelectionType};
use crate::bindings::Binding;
use crate::bindings::{BindingAction, BindingsLayout, InputKind};
use crate::font::TerminalFont;
use crate::theme::TerminalTheme;
use crate::types::Size;

const EGUI_TERM_WIDGET_ID_PREFIX: &str = "egui_term::instance::";

#[derive(Debug, Clone)]
enum InputAction {
    BackendCall(BackendCommand),
    WriteToClipboard(String),
    Ignore,
}

const RESIZE_DEBOUNCE_SECS: f64 = 0.08;

pub fn terminal_focus_event_filter() -> egui::EventFilter {
    egui::EventFilter {
        tab: true,
        horizontal_arrows: true,
        vertical_arrows: true,
        escape: true,
    }
}

#[derive(Clone, Default)]
pub struct TerminalViewState {
    is_dragged: bool,
    scroll_pixels: f32,
    /// Scrollbar thumb drag: (lines-per-pixel, offset at drag start).
    scrollbar_drag: Option<(f32, usize)>,
    /// Target absolute display offset requested by the scrollbar; applied
    /// to the backend right after drawing.
    pending_scroll_to: Option<usize>,
    /// Scrollbar geometry+colors to paint at the end of show().
    pending_scrollbar:
        Option<(egui::Rect, egui::Rect, egui::Color32, egui::Color32)>,
    current_mouse_position_on_grid: TerminalGridPoint,
    /// Last cols/rows actually applied to the terminal backend.
    last_cols: u16,
    last_rows: u16,
    /// Pending size while layout is still changing (drag).
    pending_cols: u16,
    pending_rows: u16,
    pending_since: Option<f64>,
}

pub struct TerminalView<'a> {
    widget_id: Id,
    has_focus: bool,
    size: Vec2,
    backend: &'a mut TerminalBackend,
    font: TerminalFont,
    theme: TerminalTheme,
    bindings_layout: BindingsLayout,
}

impl Widget for TerminalView<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> Response {
        let (layout, painter) =
            ui.allocate_painter(self.size, egui::Sense::click());

        let widget_id = self.widget_id;
        let mut state = ui.memory(|m| {
            m.data
                .get_temp::<TerminalViewState>(widget_id)
                .unwrap_or_default()
        });

        // Scrollback scrollbar (interaction + Foreground-layer paint).
        self.draw_scrollbar(ui, &mut state, &layout);

        self.focus(&layout)
            .resize(&layout, &mut state)
            .process_input(&layout, &mut state)
            .show(&mut state, &layout, &painter);

        ui.memory_mut(|m| m.data.insert_temp(widget_id, state));
        layout
    }
}

impl<'a> TerminalView<'a> {
    pub fn new(ui: &mut egui::Ui, backend: &'a mut TerminalBackend) -> Self {
        let widget_id = ui.make_persistent_id(format!(
            "{}{}",
            EGUI_TERM_WIDGET_ID_PREFIX, backend.id
        ));

        Self {
            widget_id,
            has_focus: false,
            size: ui.available_size(),
            backend,
            font: TerminalFont::default(),
            theme: TerminalTheme::default(),
            bindings_layout: BindingsLayout::new(),
        }
    }

    #[inline]
    pub fn set_theme(mut self, theme: TerminalTheme) -> Self {
        self.theme = theme;
        self
    }

    #[inline]
    pub fn set_font(mut self, font: TerminalFont) -> Self {
        self.font = font;
        self
    }

    #[inline]
    pub fn set_focus(mut self, has_focus: bool) -> Self {
        self.has_focus = has_focus;
        self
    }

    #[inline]
    pub fn set_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub fn add_bindings(
        mut self,
        bindings: Vec<(Binding<InputKind>, BindingAction)>,
    ) -> Self {
        self.bindings_layout.add_bindings(bindings);
        self
    }

    fn focus(self, layout: &Response) -> Self {
        if self.has_focus {
            layout.request_focus();
            layout.ctx.memory_mut(|memory| {
                memory.set_focus_lock_filter(
                    layout.id,
                    terminal_focus_event_filter(),
                )
            });
        } else {
            layout.surrender_focus();
        }

        self
    }

    /// Compute stable integer cols/rows and only apply backend resize when
    /// the grid size actually changes and has settled (debounce). This avoids
    /// thrashing reflow when dock splitters / window edges jitter by <1 cell.
    fn resize(self, layout: &Response, state: &mut TerminalViewState) -> Self {
        let font_size = self.font.font_measure(&layout.ctx);
        let layout_size = Size::from(layout.rect.size());

        let cell_w = font_size.width.floor().max(1.0);
        let cell_h = font_size.height.floor().max(1.0);
        // Slight positive bias so a nearly-full cell still counts, without
        // oscillating at the exact boundary.
        let cols = ((layout_size.width + 0.5) / cell_w).floor() as u16;
        let rows = ((layout_size.height + 0.5) / cell_h).floor() as u16;

        if cols == 0 || rows == 0 {
            return self;
        }

        // First real size: apply immediately so the PTY matches the view.
        if state.last_cols == 0 || state.last_rows == 0 {
            state.last_cols = cols;
            state.last_rows = rows;
            state.pending_cols = cols;
            state.pending_rows = rows;
            state.pending_since = None;
            self.backend.process_command(BackendCommand::Resize(
                layout_size,
                font_size,
            ));
            return self;
        }

        // Already at this grid size — nothing to do (even if pixels jitter).
        if cols == state.last_cols && rows == state.last_rows {
            state.pending_cols = cols;
            state.pending_rows = rows;
            state.pending_since = None;
            return self;
        }

        let now = layout.ctx.input(|i| i.time);

        // Size still changing: restart debounce timer.
        if cols != state.pending_cols || rows != state.pending_rows {
            state.pending_cols = cols;
            state.pending_rows = rows;
            state.pending_since = Some(now);
            layout.ctx.request_repaint_after(
                std::time::Duration::from_secs_f64(RESIZE_DEBOUNCE_SECS),
            );
            return self;
        }

        // Pending size stable long enough — commit reflow once.
        if let Some(since) = state.pending_since {
            let elapsed = now - since;
            if elapsed >= RESIZE_DEBOUNCE_SECS {
                state.last_cols = cols;
                state.last_rows = rows;
                state.pending_since = None;
                self.backend.process_command(BackendCommand::Resize(
                    layout_size,
                    font_size,
                ));
            } else {
                layout.ctx.request_repaint_after(
                    std::time::Duration::from_secs_f64(
                        RESIZE_DEBOUNCE_SECS - elapsed,
                    ),
                );
            }
        }

        self
    }

    fn process_input(
        self,
        layout: &Response,
        state: &mut TerminalViewState,
    ) -> Self {
        if !layout.has_focus() {
            return self;
        }

        let pointer_inside = layout.contains_pointer();
        let modifiers = layout.ctx.input(|i| i.modifiers);
        let events = layout.ctx.input(|i| i.events.clone());
        for event in events {
            let mut input_actions = vec![];

            match event {
                egui::Event::Text(_)
                | egui::Event::Key { .. }
                | egui::Event::Copy
                | egui::Event::Paste(_) => {
                    input_actions.push(process_keyboard_event(
                        event,
                        self.backend,
                        &self.bindings_layout,
                        modifiers,
                    ))
                },
                egui::Event::MouseWheel { unit, delta, .. }
                    if pointer_inside =>
                {
                    input_actions.push(process_mouse_wheel(
                        state,
                        self.font.font_type().size,
                        unit,
                        delta,
                    ))
                },
                egui::Event::PointerButton {
                    button,
                    pressed,
                    modifiers,
                    pos,
                    ..
                } if should_process_pointer_button(
                    pointer_inside,
                    pressed,
                    state.is_dragged,
                ) =>
                {
                    input_actions.push(process_button_click(
                        state,
                        layout,
                        self.backend,
                        &self.bindings_layout,
                        button,
                        pos,
                        &modifiers,
                        pressed,
                    ))
                },
                egui::Event::PointerMoved(pos) if pointer_inside => {
                    input_actions = process_mouse_move(
                        state,
                        layout,
                        self.backend,
                        pos,
                        &modifiers,
                    )
                },
                egui::Event::PointerGone => {
                    state.is_dragged = false;
                },
                _ => {},
            };

            for action in input_actions {
                match action {
                    InputAction::BackendCall(cmd) => {
                        self.backend.process_command(cmd);
                    },
                    InputAction::WriteToClipboard(data) => {
                        layout.ctx.copy_text(data);
                    },
                    InputAction::Ignore => {},
                }
            }
        }

        self
    }

    /// Right-edge scrollbar for the scrollback history: appears only when
    /// content exceeds the viewport, thumb size/position track the live
    /// display_offset (wheel, resize and drag all stay in sync), fully
    /// themed from the terminal colors.
    fn draw_scrollbar(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut TerminalViewState,
        layout: &egui::Response,
    ) {
        use alacritty_terminal::grid::Dimensions;

        let content = self.backend.sync();
        let display_offset = content.grid.display_offset();
        let screen_lines = content.grid.screen_lines();
        let history = content.grid.history_size();
        let max_offset = history;
        if max_offset == 0 {
            state.scrollbar_drag = None;
            return;
        }

        let total = (screen_lines + history) as f32;
        let track = egui::Rect::from_min_max(
            egui::pos2(layout.rect.max.x - 8.0, layout.rect.min.y),
            egui::pos2(layout.rect.max.x, layout.rect.max.y),
        );
        // Thumb covers its share of the total lines (min 24px grip).
        let thumb_h =
            ((screen_lines as f32 / total) * track.height()).max(24.0);
        let scrollable = (track.height() - thumb_h).max(1.0);
        // display_offset == history → viewing the oldest line → thumb TOP.
        let thumb_y = track.min.y
            + scrollable * (1.0 - display_offset as f32 / max_offset as f32);
        let thumb = egui::Rect::from_min_size(
            egui::pos2(track.min.x, thumb_y),
            egui::vec2(track.width(), thumb_h),
        );

        let track_resp = ui.interact(
            track,
            egui::Id::new((self.widget_id, "sb_track")),
            egui::Sense::click_and_drag(),
        );
        let thumb_resp = ui.interact(
            thumb.expand(3.0).intersect(track),
            egui::Id::new((self.widget_id, "sb_thumb")),
            egui::Sense::click_and_drag(),
        );
        let hovered = thumb_resp.hovered() || track_resp.hovered();
        let dragging = state.scrollbar_drag.is_some();

        if thumb_resp.drag_started() {
            let lines_per_px = max_offset as f32 / scrollable;
            state.scrollbar_drag = Some((lines_per_px, display_offset));
        }
        if let Some((lines_per_px, start_offset)) = state.scrollbar_drag {
            if thumb_resp.dragged() {
                let dy = ui.input(|i| i.pointer.delta().y);
                // Thumb DOWN → offset toward 0 (newer); UP → older.
                let delta_lines = -(dy * lines_per_px);
                let target = (start_offset as f32 + delta_lines)
                    .round()
                    .clamp(0.0, max_offset as f32);
                state.pending_scroll_to = Some(target as usize);
            } else {
                state.scrollbar_drag = None;
            }
        } else if track_resp.clicked() {
            if let Some(p) = track_resp.interact_pointer_pos() {
                let page = screen_lines as f32;
                let delta = if p.y < thumb.min.y { page } else { -page };
                let target = (display_offset as f32 + delta)
                    .round()
                    .clamp(0.0, max_offset as f32);
                state.pending_scroll_to = Some(target as usize);
            }
        }

        // Apply any requested scroll target directly to the backend.
        if let Some(target) = state.pending_scroll_to.take() {
            let delta = target as i64 - display_offset as i64;
            if delta != 0 {
                self.backend
                    .process_command(BackendCommand::Scroll(delta as i32));
                self.backend.set_dirty();
            }
        }

        // Themed colors; brighter on hover/drag.
        let fg = self.theme.get_color(Color::Named(NamedColor::Foreground));
        let base_alpha: u8 = if dragging {
            200
        } else if hovered {
            160
        } else {
            90
        };
        let bar_bg = Color32::from_rgba_unmultiplied(
            fg.r(),
            fg.g(),
            fg.b(),
            (base_alpha as u32 * 35 / 100) as u8,
        );
        let thumb_col =
            Color32::from_rgba_unmultiplied(fg.r(), fg.g(), fg.b(), base_alpha);

        // Defer painting to show(): it runs on the widget's own painter
        // AFTER the background/content shapes, so nothing covers the bar.
        state.pending_scrollbar = Some((track, thumb, bar_bg, thumb_col));
    }

    fn show(
        self,
        state: &mut TerminalViewState,
        layout: &Response,
        painter: &Painter,
    ) {
        // Request repaint for cursor blinking when focused
        if self.has_focus {
            painter.ctx().request_repaint();
            self.backend.set_dirty();
        }

        let content = self.backend.sync();
        let layout_min = layout.rect.min;
        let layout_max = layout.rect.max;
        let cell_height = content.terminal_size.cell_height as f32;
        let cell_width = content.terminal_size.cell_width as f32;
        let global_bg =
            self.theme.get_color(Color::Named(NamedColor::Background));
        let display_offset = content.grid.display_offset();

        let mut shapes = vec![Shape::Rect(RectShape::filled(
            Rect::from_min_max(layout_min, layout_max),
            CornerRadius::ZERO,
            global_bg,
        ))];

        // Grid points use absolute line coords; convert to viewport rows via
        // alacritty's point_to_viewport (same as Alacritty's own renderer).
        // Using `line + display_offset` alone is wrong for some reflow cases and
        // draws the same logical content stacked → looks like "copy on wrap".
        for indexed in content.grid.display_iter() {
            let Some(vp) = point_to_viewport(display_offset, indexed.point)
            else {
                continue;
            };

            let flags = indexed.cell.flags;
            if flags.contains(cell::Flags::WIDE_CHAR_SPACER) {
                continue;
            }

            let is_app_cursor_mode =
                content.terminal_mode.contains(TermMode::APP_CURSOR);
            let is_wide_char = flags.contains(cell::Flags::WIDE_CHAR);
            let is_inverse = flags.contains(cell::Flags::INVERSE);
            let is_dim =
                flags.intersects(cell::Flags::DIM | cell::Flags::DIM_BOLD);
            let is_selected = content
                .selectable_range
                .is_some_and(|r| r.contains(indexed.point));
            let is_hovered_hyperlink =
                content.hovered_hyperlink.as_ref().is_some_and(|r| {
                    r.contains(&indexed.point)
                        && r.contains(&state.current_mouse_position_on_grid)
                });

            let x = layout_min.x + (cell_width * vp.column.0 as f32);
            let y = layout_min.y + (cell_height * vp.line as f32);

            let mut fg = self.theme.get_color(indexed.fg);
            let mut bg = self.theme.get_color(indexed.bg);
            let draw_w = if is_wide_char {
                cell_width * 2.0
            } else {
                cell_width
            };

            if is_dim {
                fg = fg.linear_multiply(0.7);
            }

            let (fg, bg) = resolved_cell_colors(
                &self.theme,
                fg,
                bg,
                is_inverse,
                is_selected,
            );

            if global_bg != bg {
                shapes.push(Shape::Rect(RectShape::filled(
                    Rect::from_min_size(
                        Pos2::new(x, y),
                        Vec2::new(draw_w + 1.0, cell_height + 1.0),
                    ),
                    CornerRadius::ZERO,
                    bg,
                )));
            }

            if is_hovered_hyperlink {
                let underline_height = y + cell_height;
                shapes.push(Shape::LineSegment {
                    points: [
                        Pos2::new(x, underline_height),
                        Pos2::new(x + draw_w, underline_height),
                    ],
                    stroke: Stroke::new(
                        cell_height * 0.15,
                        self.theme.link_color(),
                    )
                    .into(),
                });
            }

            if indexed.c != ' ' && indexed.c != '\t' {
                let mut fg = fg;
                let mut bg = bg;
                if content.grid.cursor.point == indexed.point
                    && is_app_cursor_mode
                {
                    std::mem::swap(&mut fg, &mut bg);
                }

                shapes.push(Shape::text(
                    &painter.fonts(|c| c.clone()),
                    Pos2 {
                        x: x + (draw_w / 2.0),
                        y,
                    },
                    Align2::CENTER_TOP,
                    indexed.c,
                    self.font.font_type(),
                    fg,
                ));
            }
        }

        // Cursor: map absolute grid point → viewport, then draw if visible.
        if self.has_focus {
            let time = painter.ctx().input(|i| i.time);
            let blink_on = (time * 2.0) as i64 % 2 == 0;
            if blink_on {
                if let Some(vp) =
                    point_to_viewport(display_offset, content.grid.cursor.point)
                {
                    let cx = layout_min.x + (cell_width * vp.column.0 as f32);
                    let cy = layout_min.y + (cell_height * vp.line as f32);
                    shapes.push(Shape::Rect(RectShape::filled(
                        Rect::from_min_size(
                            Pos2::new(cx, cy),
                            Vec2::new(cell_width, cell_height),
                        ),
                        CornerRadius::default(),
                        self.theme.cursor_color(),
                    )));
                }
            }
        }

        painter.extend(shapes);

        // Scrollbar (deferred from draw_scrollbar): painted after the
        // background/content shapes so it stays visible.
        if let Some((track, thumb, bar_bg, thumb_col)) =
            state.pending_scrollbar.take()
        {
            painter.rect_filled(track, 3.0, bar_bg);
            painter.rect_filled(thumb, 3.0, thumb_col);
        }
    }
}

/// Resolve foreground/background for a cell given inverse/selection state.
///
/// Inverse swaps the cell's own colors (ANSI reverse video). Selection uses
/// the theme's selection colors rather than swapping, so selected text stays
/// readable across palettes.
fn resolved_cell_colors(
    theme: &TerminalTheme,
    mut fg: Color32,
    mut bg: Color32,
    inverse: bool,
    selected: bool,
) -> (Color32, Color32) {
    if inverse {
        std::mem::swap(&mut fg, &mut bg);
    }
    if selected {
        (theme.selection_text_color(), theme.selection_bg_color())
    } else {
        (fg, bg)
    }
}

fn process_keyboard_event(
    event: egui::Event,
    backend: &TerminalBackend,
    bindings_layout: &BindingsLayout,
    modifiers: Modifiers,
) -> InputAction {
    match event {
        egui::Event::Text(text) => {
            process_text_event(&text, modifiers, backend, bindings_layout)
        },
        egui::Event::Paste(text) => InputAction::BackendCall(
            #[cfg(not(any(target_os = "ios", target_os = "macos")))]
            if modifiers.contains(Modifiers::COMMAND | Modifiers::SHIFT) {
                BackendCommand::Write(text.as_bytes().to_vec())
            } else {
                // Hotfix - Send ^V when there's not selection on view.
                BackendCommand::Write([0x16].to_vec())
            },
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            {
                BackendCommand::Write(text.as_bytes().to_vec())
            },
        ),
        egui::Event::Copy => {
            #[cfg(not(any(target_os = "ios", target_os = "macos")))]
            if modifiers.contains(Modifiers::COMMAND | Modifiers::SHIFT) {
                let content = backend.selectable_content();
                InputAction::WriteToClipboard(content)
            } else {
                // Hotfix - Send ^C when there's not selection on view.
                InputAction::BackendCall(BackendCommand::Write([0x3].to_vec()))
            }
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            {
                let content = backend.selectable_content();
                InputAction::WriteToClipboard(content)
            }
        },
        egui::Event::Key {
            key,
            pressed,
            modifiers,
            ..
        } => process_keyboard_key(
            backend,
            bindings_layout,
            key,
            modifiers,
            pressed,
        ),
        _ => InputAction::Ignore,
    }
}

fn process_text_event(
    text: &str,
    modifiers: Modifiers,
    backend: &TerminalBackend,
    bindings_layout: &BindingsLayout,
) -> InputAction {
    if let Some(key) = Key::from_name(text) {
        if bindings_layout.get_action(
            InputKind::KeyCode(key),
            modifiers,
            backend.last_content().terminal_mode,
        ) == BindingAction::Ignore
        {
            InputAction::BackendCall(BackendCommand::Write(
                text.as_bytes().to_vec(),
            ))
        } else {
            InputAction::Ignore
        }
    } else {
        InputAction::BackendCall(BackendCommand::Write(
            text.as_bytes().to_vec(),
        ))
    }
}

fn process_keyboard_key(
    backend: &TerminalBackend,
    bindings_layout: &BindingsLayout,
    key: Key,
    modifiers: Modifiers,
    pressed: bool,
) -> InputAction {
    if !pressed {
        return InputAction::Ignore;
    }

    let terminal_mode = backend.last_content().terminal_mode;
    let binding_action = bindings_layout.get_action(
        InputKind::KeyCode(key),
        modifiers,
        terminal_mode,
    );

    match binding_action {
        BindingAction::Char(c) => {
            let mut buf = [0, 0, 0, 0];
            let str = c.encode_utf8(&mut buf);
            InputAction::BackendCall(BackendCommand::Write(
                str.as_bytes().to_vec(),
            ))
        },
        BindingAction::Esc(seq) => InputAction::BackendCall(
            BackendCommand::Write(seq.as_bytes().to_vec()),
        ),
        _ => InputAction::Ignore,
    }
}

fn process_mouse_wheel(
    state: &mut TerminalViewState,
    font_size: f32,
    unit: MouseWheelUnit,
    delta: Vec2,
) -> InputAction {
    match unit {
        MouseWheelUnit::Line => {
            let lines = delta.y.signum() * delta.y.abs().ceil();
            InputAction::BackendCall(BackendCommand::Scroll(lines as i32))
        },
        MouseWheelUnit::Point => {
            state.scroll_pixels -= delta.y;
            let lines = (state.scroll_pixels / font_size).trunc();
            state.scroll_pixels %= font_size;
            if lines != 0.0 {
                InputAction::BackendCall(BackendCommand::Scroll(-lines as i32))
            } else {
                InputAction::Ignore
            }
        },
        MouseWheelUnit::Page => InputAction::Ignore,
    }
}

fn process_button_click(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    bindings_layout: &BindingsLayout,
    button: PointerButton,
    position: Pos2,
    modifiers: &Modifiers,
    pressed: bool,
) -> InputAction {
    match button {
        PointerButton::Primary => process_left_button(
            state,
            layout,
            backend,
            bindings_layout,
            position,
            modifiers,
            pressed,
        ),
        _ => InputAction::Ignore,
    }
}

fn should_process_pointer_button(
    pointer_inside: bool,
    pressed: bool,
    was_dragged: bool,
) -> bool {
    pointer_inside || (!pressed && was_dragged)
}

fn process_left_button(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    bindings_layout: &BindingsLayout,
    position: Pos2,
    modifiers: &Modifiers,
    pressed: bool,
) -> InputAction {
    let terminal_mode = backend.last_content().terminal_mode;
    if terminal_mode.intersects(TermMode::MOUSE_MODE) {
        InputAction::BackendCall(BackendCommand::MouseReport(
            MouseButton::LeftButton,
            *modifiers,
            state.current_mouse_position_on_grid,
            pressed,
        ))
    } else if pressed {
        process_left_button_pressed(state, layout, position)
    } else {
        process_left_button_released(
            state,
            layout,
            backend,
            bindings_layout,
            position,
            modifiers,
        )
    }
}

fn process_left_button_pressed(
    state: &mut TerminalViewState,
    layout: &Response,
    position: Pos2,
) -> InputAction {
    state.is_dragged = true;
    InputAction::BackendCall(build_start_select_command(layout, position))
}

fn process_left_button_released(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    bindings_layout: &BindingsLayout,
    position: Pos2,
    modifiers: &Modifiers,
) -> InputAction {
    state.is_dragged = false;
    if layout.double_clicked() || layout.triple_clicked() {
        InputAction::BackendCall(build_start_select_command(layout, position))
    } else {
        let terminal_content = backend.last_content();
        let binding_action = bindings_layout.get_action(
            InputKind::Mouse(PointerButton::Primary),
            *modifiers,
            terminal_content.terminal_mode,
        );

        if binding_action == BindingAction::LinkOpen {
            InputAction::BackendCall(BackendCommand::ProcessLink(
                LinkAction::Open,
                state.current_mouse_position_on_grid,
            ))
        } else {
            InputAction::Ignore
        }
    }
}

fn build_start_select_command(
    layout: &Response,
    cursor_position: Pos2,
) -> BackendCommand {
    let selection_type = if layout.double_clicked() {
        SelectionType::Semantic
    } else if layout.triple_clicked() {
        SelectionType::Lines
    } else {
        SelectionType::Simple
    };

    BackendCommand::SelectStart(
        selection_type,
        cursor_position.x - layout.rect.min.x,
        cursor_position.y - layout.rect.min.y,
    )
}

fn process_mouse_move(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    position: Pos2,
    modifiers: &Modifiers,
) -> Vec<InputAction> {
    let terminal_content = backend.last_content();
    let cursor_x = position.x - layout.rect.min.x;
    let cursor_y = position.y - layout.rect.min.y;
    state.current_mouse_position_on_grid = TerminalBackend::selection_point(
        cursor_x,
        cursor_y,
        &terminal_content.terminal_size,
        terminal_content.grid.display_offset(),
    );

    let mut actions = vec![];
    // Handle command or selection update based on terminal mode and modifiers
    if state.is_dragged {
        let terminal_mode = terminal_content.terminal_mode;
        let cmd = if terminal_mode.contains(TermMode::MOUSE_MOTION)
            && modifiers.is_none()
        {
            InputAction::BackendCall(BackendCommand::MouseReport(
                MouseButton::LeftMove,
                *modifiers,
                state.current_mouse_position_on_grid,
                true,
            ))
        } else {
            InputAction::BackendCall(BackendCommand::SelectUpdate(
                cursor_x, cursor_y,
            ))
        };

        actions.push(cmd);
    }

    // Handle link hover if applicable
    if modifiers.command_only() {
        actions.push(InputAction::BackendCall(BackendCommand::ProcessLink(
            LinkAction::Hover,
            state.current_mouse_position_on_grid,
        )));
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::{should_process_pointer_button, terminal_focus_event_filter};

    #[test]
    fn pointer_release_outside_is_processed_when_button_was_pressed_inside() {
        assert!(should_process_pointer_button(false, false, true));
        assert!(!should_process_pointer_button(false, false, false));
    }

    #[test]
    fn focused_terminal_keeps_vertical_arrows_from_moving_egui_focus() {
        let filter = terminal_focus_event_filter();

        assert!(filter.tab);
        assert!(filter.horizontal_arrows);
        assert!(filter.vertical_arrows);
        assert!(filter.escape);
    }
}

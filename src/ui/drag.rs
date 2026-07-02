use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

#[derive(Debug, Clone)]
pub struct DragState {
    pub is_dragging: bool,
    pub start_x: u16,
    pub start_y: u16,
    pub current_x: u16,
    pub current_y: u16,
    pub target_pane_id: Option<String>,
}

impl DragState {
    pub fn new() -> Self {
        DragState {
            is_dragging: false,
            start_x: 0,
            start_y: 0,
            current_x: 0,
            current_y: 0,
            target_pane_id: None,
        }
    }

    pub fn start_drag(&mut self, x: u16, y: u16, pane_id: &str) {
        self.is_dragging = true;
        self.start_x = x;
        self.start_y = y;
        self.current_x = x;
        self.current_y = y;
        self.target_pane_id = Some(pane_id.to_string());
    }

    pub fn update_drag(&mut self, x: u16, y: u16) {
        if self.is_dragging {
            self.current_x = x;
            self.current_y = y;
        }
    }

    pub fn end_drag(&mut self) -> Option<(f32, f32)> {
        if self.is_dragging {
            self.is_dragging = false;
            let dx = self.current_x as f32 - self.start_x as f32;
            let dy = self.current_y as f32 - self.start_y as f32;
            self.target_pane_id = None;
            Some((dx, dy))
        } else {
            None
        }
    }

    pub fn calculate_new_ratio(&self, area_width: u16, area_height: u16) -> Option<f32> {
        if self.is_dragging {
            let dx = self.current_x as f32 - self.start_x as f32;
            let dy = self.current_y as f32 - self.start_y as f32;

            if area_width > 0 && area_height > 0 {
                let ratio_change = if dx.abs() > dy.abs() {
                    dx / area_width as f32
                } else {
                    dy / area_height as f32
                };
                Some(0.5 + ratio_change)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for DragState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_mouse_event(event: MouseEvent, drag_state: &mut DragState) {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Start drag on separator
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if let Some(_) = drag_state.end_drag() {
                // Apply new ratio
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            drag_state.update_drag(event.column, event.row);
        }
        _ => {}
    }
}

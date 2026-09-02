//! Remote phone control: embedded HTTP server, snapshot sharing and
//! command queue plumbing.

pub mod ansi;
pub mod frp;
pub mod protocol;
pub mod server;
pub mod tunnel;
pub mod ws;

/// The phone web page, embedded at compile time.
pub(crate) const REMOTE_PAGE: &str = include_str!("../../assets/remote.html");

/// Map a wire mouse-button code (xterm-style) to the backend enum.
/// Codes: 0/1/2 buttons, 32-35 motion, 64/65 wheel, 99 other.
pub fn remote_mouse_button(code: u8) -> egui_term::MouseButton {
    match code {
        0 => egui_term::MouseButton::LeftButton,
        1 => egui_term::MouseButton::MiddleButton,
        2 => egui_term::MouseButton::RightButton,
        32 => egui_term::MouseButton::LeftMove,
        33 => egui_term::MouseButton::MiddleMove,
        34 => egui_term::MouseButton::RightMove,
        35 => egui_term::MouseButton::NoneMove,
        64 => egui_term::MouseButton::ScrollUp,
        65 => egui_term::MouseButton::ScrollDown,
        _ => egui_term::MouseButton::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::remote_mouse_button;
    use egui_term::MouseButton;

    #[test]
    fn mouse_button_codes_map_to_backend_variants() {
        assert!(matches!(remote_mouse_button(0), MouseButton::LeftButton));
        assert!(matches!(remote_mouse_button(1), MouseButton::MiddleButton));
        assert!(matches!(remote_mouse_button(2), MouseButton::RightButton));
        assert!(matches!(remote_mouse_button(32), MouseButton::LeftMove));
        assert!(matches!(remote_mouse_button(35), MouseButton::NoneMove));
        assert!(matches!(remote_mouse_button(64), MouseButton::ScrollUp));
        assert!(matches!(remote_mouse_button(65), MouseButton::ScrollDown));
        assert!(matches!(remote_mouse_button(99), MouseButton::Other));
        assert!(matches!(remote_mouse_button(200), MouseButton::Other));
    }
}

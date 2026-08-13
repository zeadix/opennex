mod backend;
mod bindings;
mod font;
mod theme;
mod types;
mod view;

pub use backend::settings::BackendSettings;
pub use backend::{BackendCommand, TerminalBackend, TerminalMode};
pub use bindings::{Binding, BindingAction, InputKind, KeyboardBinding};
pub use font::{FontSettings, TerminalFont};
pub use theme::{ColorPalette, TerminalTheme, TerminalVisualColors};
pub use view::{terminal_focus_event_filter, TerminalView};

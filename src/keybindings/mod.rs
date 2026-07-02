pub mod defaults;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub type KeyBinding = (KeyModifiers, KeyCode);

pub struct KeybindingManager {
    bindings: HashMap<String, KeyBinding>,
}

impl KeybindingManager {
    pub fn new() -> Self {
        let mut manager = KeybindingManager {
            bindings: HashMap::new(),
        };
        manager.load_defaults();
        manager
    }

    pub fn load_defaults(&mut self) {
        let defaults = defaults::get_default_bindings();
        for (action, binding) in defaults {
            self.bindings.insert(action, binding);
        }
    }

    pub fn get_binding(&self, action: &str) -> Option<&KeyBinding> {
        self.bindings.get(action)
    }

    pub fn set_binding(&mut self, action: &str, binding: KeyBinding) {
        self.bindings.insert(action.to_string(), binding);
    }

    pub fn check_key(&self, key: KeyEvent) -> Option<String> {
        for (action, (modifiers, code)) in &self.bindings {
            if key.modifiers == *modifiers && key.code == *code {
                return Some(action.clone());
            }
        }
        None
    }
}

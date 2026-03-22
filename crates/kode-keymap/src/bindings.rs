use std::collections::HashMap;

use kode_core::event::KeyEvent;
use crate::mode::Mode;

/// A keybinding maps a key in a mode to a named action.
#[derive(Debug, Clone)]
pub struct Keybinding {
    pub mode: Mode,
    pub key: KeyEvent,
    pub action: String,
    pub description: String,
}

/// Keybinding registry for user customization.
#[derive(Debug, Default)]
pub struct KeybindingRegistry {
    bindings: HashMap<(Mode, KeyEvent), Keybinding>,
}

impl KeybindingRegistry {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn register(&mut self, binding: Keybinding) {
        self.bindings
            .insert((binding.mode, binding.key), binding);
    }

    pub fn get(&self, mode: Mode, key: KeyEvent) -> Option<&Keybinding> {
        self.bindings.get(&(mode, key))
    }

    pub fn all_for_mode(&self, mode: Mode) -> Vec<&Keybinding> {
        self.bindings
            .values()
            .filter(|b| b.mode == mode)
            .collect()
    }
}

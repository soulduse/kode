use kode_core::event::{KeyCode, KeyEvent, Modifiers};
use winit::event::ElementState;
use winit::keyboard::{Key, NamedKey};

/// Convert a winit keyboard event to a KodeEvent KeyEvent.
pub fn translate_winit_key(
    event: &winit::event::KeyEvent,
    modifier_state: &winit::event::Modifiers,
) -> Option<KeyEvent> {
    // Only handle key presses, not releases
    if event.state != ElementState::Pressed {
        return None;
    }

    let code = match &event.logical_key {
        Key::Character(c) => {
            let ch = c.chars().next()?;
            KeyCode::Char(ch)
        }
        Key::Named(named) => match named {
            NamedKey::Enter => KeyCode::Enter,
            NamedKey::Escape => KeyCode::Escape,
            NamedKey::Backspace => KeyCode::Backspace,
            NamedKey::Delete => KeyCode::Delete,
            NamedKey::Tab => KeyCode::Tab,
            NamedKey::ArrowUp => KeyCode::Up,
            NamedKey::ArrowDown => KeyCode::Down,
            NamedKey::ArrowLeft => KeyCode::Left,
            NamedKey::ArrowRight => KeyCode::Right,
            NamedKey::Home => KeyCode::Home,
            NamedKey::End => KeyCode::End,
            NamedKey::PageUp => KeyCode::PageUp,
            NamedKey::PageDown => KeyCode::PageDown,
            NamedKey::F1 => KeyCode::F(1),
            NamedKey::F2 => KeyCode::F(2),
            NamedKey::F3 => KeyCode::F(3),
            NamedKey::F4 => KeyCode::F(4),
            NamedKey::F5 => KeyCode::F(5),
            NamedKey::F6 => KeyCode::F(6),
            NamedKey::F7 => KeyCode::F(7),
            NamedKey::F8 => KeyCode::F(8),
            NamedKey::F9 => KeyCode::F(9),
            NamedKey::F10 => KeyCode::F(10),
            NamedKey::F11 => KeyCode::F(11),
            NamedKey::F12 => KeyCode::F(12),
            _ => return None,
        },
        _ => return None,
    };

    let state = modifier_state.state();
    let mut mods = Modifiers::NONE;
    if state.control_key() {
        mods = mods | Modifiers::CTRL;
    }
    if state.alt_key() {
        mods = mods | Modifiers::ALT;
    }
    if state.shift_key() {
        mods = mods | Modifiers::SHIFT;
    }
    if state.super_key() {
        mods = mods | Modifiers::SUPER;
    }

    Some(KeyEvent::new(code, mods))
}

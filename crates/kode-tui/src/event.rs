use crossterm::event as ct;
use kode_core::event::{KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent};

/// Convert a crossterm key event to a kode KeyEvent.
pub fn crossterm_to_kode_key(key: ct::KeyEvent) -> Option<KeyEvent> {
    // Ignore key release and repeat events
    if key.kind != ct::KeyEventKind::Press {
        return None;
    }

    let code = match key.code {
        ct::KeyCode::Char(c) => KeyCode::Char(c),
        ct::KeyCode::Enter => KeyCode::Enter,
        ct::KeyCode::Esc => KeyCode::Escape,
        ct::KeyCode::Backspace => KeyCode::Backspace,
        ct::KeyCode::Delete => KeyCode::Delete,
        ct::KeyCode::Tab => KeyCode::Tab,
        ct::KeyCode::Up => KeyCode::Up,
        ct::KeyCode::Down => KeyCode::Down,
        ct::KeyCode::Left => KeyCode::Left,
        ct::KeyCode::Right => KeyCode::Right,
        ct::KeyCode::Home => KeyCode::Home,
        ct::KeyCode::End => KeyCode::End,
        ct::KeyCode::PageUp => KeyCode::PageUp,
        ct::KeyCode::PageDown => KeyCode::PageDown,
        ct::KeyCode::F(n) => KeyCode::F(n),
        _ => return None,
    };

    let mut modifiers = Modifiers::NONE;
    if key.modifiers.contains(ct::KeyModifiers::CONTROL) {
        modifiers = modifiers | Modifiers::CTRL;
    }
    if key.modifiers.contains(ct::KeyModifiers::ALT) {
        modifiers = modifiers | Modifiers::ALT;
    }
    if key.modifiers.contains(ct::KeyModifiers::SHIFT) {
        modifiers = modifiers | Modifiers::SHIFT;
    }

    Some(KeyEvent::new(code, modifiers))
}

/// Convert a crossterm mouse event to a kode MouseEvent.
pub fn crossterm_to_kode_mouse(mouse: ct::MouseEvent) -> Option<MouseEvent> {
    let x = mouse.column as f32;
    let y = mouse.row as f32;

    match mouse.kind {
        ct::MouseEventKind::Down(btn) => {
            let button = match btn {
                ct::MouseButton::Left => MouseButton::Left,
                ct::MouseButton::Right => MouseButton::Right,
                ct::MouseButton::Middle => MouseButton::Middle,
            };
            Some(MouseEvent::Press { button, x, y })
        }
        ct::MouseEventKind::Up(_) => Some(MouseEvent::Release { x, y }),
        ct::MouseEventKind::Moved => Some(MouseEvent::Move { x, y }),
        ct::MouseEventKind::ScrollUp => Some(MouseEvent::Scroll {
            delta_x: 0.0,
            delta_y: -3.0,
        }),
        ct::MouseEventKind::ScrollDown => Some(MouseEvent::Scroll {
            delta_x: 0.0,
            delta_y: 3.0,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(code: ct::KeyCode, modifiers: ct::KeyModifiers) -> ct::KeyEvent {
        ct::KeyEvent {
            code,
            modifiers,
            kind: ct::KeyEventKind::Press,
            state: ct::KeyEventState::NONE,
        }
    }

    #[test]
    fn char_key() {
        let ct_key = make_key(ct::KeyCode::Char('a'), ct::KeyModifiers::NONE);
        let kode_key = crossterm_to_kode_key(ct_key).unwrap();
        assert_eq!(kode_key.code, KeyCode::Char('a'));
        assert_eq!(kode_key.modifiers, Modifiers::NONE);
    }

    #[test]
    fn ctrl_modifier() {
        let ct_key = make_key(ct::KeyCode::Char('c'), ct::KeyModifiers::CONTROL);
        let kode_key = crossterm_to_kode_key(ct_key).unwrap();
        assert!(kode_key.modifiers.contains(Modifiers::CTRL));
    }

    #[test]
    fn arrow_keys() {
        let ct_key = make_key(ct::KeyCode::Up, ct::KeyModifiers::NONE);
        let kode_key = crossterm_to_kode_key(ct_key).unwrap();
        assert_eq!(kode_key.code, KeyCode::Up);
    }

    #[test]
    fn function_key() {
        let ct_key = make_key(ct::KeyCode::F(5), ct::KeyModifiers::NONE);
        let kode_key = crossterm_to_kode_key(ct_key).unwrap();
        assert_eq!(kode_key.code, KeyCode::F(5));
    }

    #[test]
    fn ignores_release() {
        let ct_key = ct::KeyEvent {
            code: ct::KeyCode::Char('a'),
            modifiers: ct::KeyModifiers::NONE,
            kind: ct::KeyEventKind::Release,
            state: ct::KeyEventState::NONE,
        };
        assert!(crossterm_to_kode_key(ct_key).is_none());
    }

    #[test]
    fn mouse_press() {
        let ct_mouse = ct::MouseEvent {
            kind: ct::MouseEventKind::Down(ct::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: ct::KeyModifiers::NONE,
        };
        match crossterm_to_kode_mouse(ct_mouse) {
            Some(MouseEvent::Press { button, x, y }) => {
                assert_eq!(button, MouseButton::Left);
                assert_eq!(x, 10.0);
                assert_eq!(y, 5.0);
            }
            other => panic!("Expected Press, got {:?}", other),
        }
    }

    #[test]
    fn mouse_scroll() {
        let ct_mouse = ct::MouseEvent {
            kind: ct::MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: ct::KeyModifiers::NONE,
        };
        match crossterm_to_kode_mouse(ct_mouse) {
            Some(MouseEvent::Scroll { delta_y, .. }) => {
                assert!(delta_y > 0.0);
            }
            other => panic!("Expected Scroll, got {:?}", other),
        }
    }
}

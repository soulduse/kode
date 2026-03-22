use kode_core::event::{KeyCode, KeyEvent, Modifiers};

/// Convert a key event to terminal escape sequence bytes.
pub fn key_to_escape(key: &KeyEvent) -> Option<Vec<u8>> {
    let ctrl = key.modifiers.contains(Modifiers::CTRL);
    let alt = key.modifiers.contains(Modifiers::ALT);

    match key.code {
        KeyCode::Char(c) if ctrl => {
            // Ctrl+A = 0x01, Ctrl+Z = 0x1A
            let ctrl_code = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a').wrapping_add(1);
            if ctrl_code <= 26 {
                if alt {
                    Some(vec![0x1b, ctrl_code])
                } else {
                    Some(vec![ctrl_code])
                }
            } else {
                None
            }
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            if alt {
                let mut result = vec![0x1b];
                result.extend_from_slice(s.as_bytes());
                Some(result)
            } else {
                Some(s.as_bytes().to_vec())
            }
        }
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Escape => Some(vec![0x1b]),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),

        // Arrow keys
        KeyCode::Up if ctrl => Some(b"\x1b[1;5A".to_vec()),
        KeyCode::Down if ctrl => Some(b"\x1b[1;5B".to_vec()),
        KeyCode::Right if ctrl => Some(b"\x1b[1;5C".to_vec()),
        KeyCode::Left if ctrl => Some(b"\x1b[1;5D".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),

        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),

        // Function keys
        KeyCode::F(1) => Some(b"\x1bOP".to_vec()),
        KeyCode::F(2) => Some(b"\x1bOQ".to_vec()),
        KeyCode::F(3) => Some(b"\x1bOR".to_vec()),
        KeyCode::F(4) => Some(b"\x1bOS".to_vec()),
        KeyCode::F(5) => Some(b"\x1b[15~".to_vec()),
        KeyCode::F(6) => Some(b"\x1b[17~".to_vec()),
        KeyCode::F(7) => Some(b"\x1b[18~".to_vec()),
        KeyCode::F(8) => Some(b"\x1b[19~".to_vec()),
        KeyCode::F(9) => Some(b"\x1b[20~".to_vec()),
        KeyCode::F(10) => Some(b"\x1b[21~".to_vec()),
        KeyCode::F(11) => Some(b"\x1b[23~".to_vec()),
        KeyCode::F(12) => Some(b"\x1b[24~".to_vec()),
        KeyCode::F(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_char() {
        let key = KeyEvent::plain(KeyCode::Char('a'));
        assert_eq!(key_to_escape(&key), Some(vec![b'a']));
    }

    #[test]
    fn ctrl_c() {
        let key = KeyEvent::new(KeyCode::Char('c'), Modifiers::CTRL);
        assert_eq!(key_to_escape(&key), Some(vec![0x03]));
    }

    #[test]
    fn arrow_keys() {
        let key = KeyEvent::plain(KeyCode::Up);
        assert_eq!(key_to_escape(&key), Some(b"\x1b[A".to_vec()));
    }

    #[test]
    fn enter_key() {
        let key = KeyEvent::plain(KeyCode::Enter);
        assert_eq!(key_to_escape(&key), Some(vec![b'\r']));
    }

    #[test]
    fn alt_char() {
        let key = KeyEvent::new(KeyCode::Char('x'), Modifiers::ALT);
        assert_eq!(key_to_escape(&key), Some(vec![0x1b, b'x']));
    }

    #[test]
    fn function_keys() {
        let key = KeyEvent::plain(KeyCode::F(1));
        assert_eq!(key_to_escape(&key), Some(b"\x1bOP".to_vec()));
    }

    #[test]
    fn unicode_char() {
        let key = KeyEvent::plain(KeyCode::Char('가'));
        let result = key_to_escape(&key).unwrap();
        assert_eq!(std::str::from_utf8(&result).unwrap(), "가");
    }
}

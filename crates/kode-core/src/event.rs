/// Central event type for all cross-component communication.
#[derive(Debug, Clone)]
pub enum KodeEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize { width: u32, height: u32 },
    Command(EditorCommand),
    Tick,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    pub fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: Modifiers::NONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Delete,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const CTRL: Self = Self(1);
    pub const ALT: Self = Self(2);
    pub const SHIFT: Self = Self(4);
    pub const SUPER: Self = Self(8);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone)]
pub enum MouseEvent {
    Press {
        button: MouseButton,
        x: f32,
        y: f32,
    },
    Release {
        x: f32,
        y: f32,
    },
    Move {
        x: f32,
        y: f32,
    },
    Scroll {
        delta_x: f32,
        delta_y: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone)]
pub enum EditorCommand {
    InsertChar(char),
    NewLine,
    DeleteForward,
    DeleteBackward,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveFileStart,
    MoveFileEnd,
    PageUp,
    PageDown,
    Save,
    Open(std::path::PathBuf),
    Quit,
    Undo,
    Redo,
    SelectAll,
    Copy,
    Paste(String),
    Cut,
    Find(String),
    Replace { find: String, replace: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_operations() {
        let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;
        assert!(ctrl_shift.contains(Modifiers::CTRL));
        assert!(ctrl_shift.contains(Modifiers::SHIFT));
        assert!(!ctrl_shift.contains(Modifiers::ALT));
    }
}

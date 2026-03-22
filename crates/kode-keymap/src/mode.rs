/// Vim editing modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    VisualBlock,
    Command,
    Replace,
}

impl Mode {
    pub fn is_insert(&self) -> bool {
        matches!(self, Mode::Insert)
    }

    pub fn is_normal(&self) -> bool {
        matches!(self, Mode::Normal)
    }

    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual | Mode::VisualLine | Mode::VisualBlock)
    }

    pub fn cursor_style(&self) -> CursorStyle {
        match self {
            Mode::Normal | Mode::Visual | Mode::VisualLine | Mode::VisualBlock => {
                CursorStyle::Block
            }
            Mode::Insert => CursorStyle::Line,
            Mode::Replace => CursorStyle::Underline,
            Mode::Command => CursorStyle::Line,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
            Mode::VisualBlock => "V-BLOCK",
            Mode::Command => "COMMAND",
            Mode::Replace => "REPLACE",
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}

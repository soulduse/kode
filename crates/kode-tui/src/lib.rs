/// TUI fallback rendering using crossterm.
///
/// Shares kode-editor, kode-keymap, kode-workspace logic
/// with the GPU frontend but renders to a terminal instead.
///
/// This module will be implemented in a later phase.
pub struct TuiBackend {
    _placeholder: (),
}

impl TuiBackend {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for TuiBackend {
    fn default() -> Self {
        Self::new()
    }
}

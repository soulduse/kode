/// Text shaping and layout using cosmic-text.
///
/// Handles font loading, text shaping, line breaking,
/// and glyph positioning for the editor viewport.
pub struct TextLayout {
    _placeholder: (),
}

impl TextLayout {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for TextLayout {
    fn default() -> Self {
        Self::new()
    }
}

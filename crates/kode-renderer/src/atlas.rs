/// Glyph atlas for caching rasterized glyphs in a GPU texture.
///
/// Maps GlyphKey (font_id, glyph_id, size) to texture atlas regions.
/// Uses cosmic-text SwashCache for glyph rasterization.
pub struct GlyphAtlas {
    _placeholder: (),
}

impl GlyphAtlas {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new()
    }
}

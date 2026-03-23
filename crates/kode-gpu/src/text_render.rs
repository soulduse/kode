use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer, Viewport,
};

pub struct KodeTextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub text_atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub viewport: Viewport,
    pub cell_width: f32,
    pub line_height: f32,
    pub font_size: f32,
}

impl KodeTextRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let mut text_atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut text_atlas, device, wgpu::MultisampleState::default(), None);
        let viewport = Viewport::new(device, &cache);

        let font_size = 14.0;
        let line_height = 20.0;
        let cell_width = 8.4; // approximate monospace width at 14px

        Self {
            font_system,
            swash_cache,
            text_atlas,
            text_renderer,
            viewport,
            cell_width,
            line_height,
            font_size,
        }
    }

    /// Create a text buffer with the default font size.
    pub fn create_buffer(&mut self, text: &str, width: f32) -> Buffer {
        self.create_buffer_with_size(text, width, self.font_size, self.line_height)
    }

    /// Create a text buffer with a custom font size and line height.
    pub fn create_buffer_with_size(
        &mut self,
        text: &str,
        width: f32,
        font_size: f32,
        line_height: f32,
    ) -> Buffer {
        self.create_buffer_with_family(text, width, font_size, line_height, Family::Monospace)
    }

    /// Create a text buffer with a specific font family.
    pub fn create_buffer_with_family(
        &mut self,
        text: &str,
        width: f32,
        font_size: f32,
        line_height: f32,
        family: Family<'_>,
    ) -> Buffer {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, line_height),
        );
        buffer.set_size(&mut self.font_system, Some(width), None);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(family),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    /// Create a text buffer that does NOT wrap (for single-line code rendering).
    pub fn create_buffer_no_wrap(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
    ) -> Buffer {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(font_size, line_height),
        );
        // No width limit → no word wrapping
        buffer.set_size(&mut self.font_system, None, None);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    /// Measure the x offset and width of a character at a given char index in a no-wrap buffer.
    /// Returns (x_offset, char_width) in pixels. If char_index is at or past end, returns
    /// (total_width, default_cell_width).
    pub fn measure_cursor_x(
        &mut self,
        text: &str,
        char_index: usize,
        font_size: f32,
        line_height: f32,
    ) -> (f32, f32) {
        let buffer = self.create_buffer_no_wrap(text, font_size, line_height);
        let default_w = font_size * 0.6;

        // Convert char index to byte offset
        let byte_offset: usize = text.chars().take(char_index).map(|c| c.len_utf8()).sum();

        for run in buffer.layout_runs() {
            let mut last_x_end = 0.0f32;
            let glyphs = run.glyphs;


            for glyph in glyphs.iter() {
                if glyph.start >= byte_offset {
                    if glyph.start == byte_offset {
                        return (glyph.x, glyph.w);
                    }
                    return (last_x_end, default_w);
                }
                last_x_end = glyph.x + glyph.w;
            }
            return (last_x_end, default_w);
        }

        (0.0, default_w)
    }

    /// Measure the char index at a given pixel x in a no-wrap buffer.
    /// Returns the char index closest to the given x position.
    pub fn measure_click_to_col(
        &mut self,
        text: &str,
        click_x: f32,
        font_size: f32,
        line_height: f32,
    ) -> usize {
        let buffer = self.create_buffer_no_wrap(text, font_size, line_height);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                // If click is within this glyph's hitbox
                if click_x < glyph.x + glyph.w {
                    // Convert byte offset back to char index
                    let char_idx = text[..glyph.start].chars().count();
                    // If click is in the right half, go to next char
                    if click_x > glyph.x + glyph.w / 2.0 {
                        return char_idx + 1;
                    }
                    return char_idx;
                }
            }
        }

        // Past end of line
        text.trim_end_matches('\n').chars().count()
    }

    /// Prepare and render text areas in a single render pass.
    pub fn render_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
        text_areas: Vec<PreparedTextArea>,
    ) {
        // Build glyphon TextArea slice
        let areas: Vec<TextArea> = text_areas
            .iter()
            .map(|t| TextArea {
                buffer: &t.buffer,
                left: t.left,
                top: t.top,
                scale: 1.0,
                bounds: TextBounds {
                    left: t.bounds_left as i32,
                    top: t.bounds_top as i32,
                    right: t.bounds_right as i32,
                    bottom: t.bounds_bottom as i32,
                },
                default_color: t.color,
                custom_glyphs: &[],
            })
            .collect();

        self.viewport.update(queue, glyphon::Resolution { width, height });

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.text_atlas,
                &self.viewport,
                areas,
                &mut self.swash_cache,
            )
            .unwrap();

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.text_renderer
                .render(&self.text_atlas, &self.viewport, &mut pass)
                .unwrap();
        }
    }
}

/// A prepared text area ready for rendering.
pub struct PreparedTextArea {
    pub buffer: Buffer,
    pub left: f32,
    pub top: f32,
    pub bounds_left: f32,
    pub bounds_top: f32,
    pub bounds_right: f32,
    pub bounds_bottom: f32,
    pub color: Color,
}

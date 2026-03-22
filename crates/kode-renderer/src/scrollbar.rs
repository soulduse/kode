use kode_core::color::Color;
use kode_core::geometry::Rect;

use crate::RenderCommand;

/// Render a vertical scrollbar indicator.
pub fn render_scrollbar(
    viewport_height: f32,
    total_lines: usize,
    visible_lines: usize,
    scroll_offset: usize,
    x: f32,
    y: f32,
    width: f32,
    color: Color,
) -> Option<RenderCommand> {
    if total_lines <= visible_lines {
        return None;
    }

    let ratio = visible_lines as f32 / total_lines as f32;
    let bar_height = (viewport_height * ratio).max(20.0);
    let scroll_ratio = scroll_offset as f32 / (total_lines - visible_lines) as f32;
    let bar_y = y + scroll_ratio * (viewport_height - bar_height);

    Some(RenderCommand::DrawRect {
        rect: Rect::new(x, bar_y, width, bar_height),
        color: color.to_array(),
    })
}

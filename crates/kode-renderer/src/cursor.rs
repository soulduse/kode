use kode_core::color::Color;
use kode_core::geometry::Rect;

use crate::RenderCommand;

/// Cursor rendering styles.
#[derive(Debug, Clone, Copy)]
pub enum CursorShape {
    Block,
    Line,
    Underline,
}

/// Render a cursor at the given cell position.
pub fn render_cursor(
    shape: CursorShape,
    x: f32,
    y: f32,
    cell_width: f32,
    cell_height: f32,
    color: Color,
) -> RenderCommand {
    let rect = match shape {
        CursorShape::Block => Rect::new(x, y, cell_width, cell_height),
        CursorShape::Line => Rect::new(x, y, 2.0, cell_height),
        CursorShape::Underline => Rect::new(x, y + cell_height - 2.0, cell_width, 2.0),
    };
    RenderCommand::DrawRect {
        rect,
        color: color.to_array(),
    }
}

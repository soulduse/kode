pub mod atlas;
pub mod cursor;
pub mod gutter;
pub mod pipeline;
pub mod scrollbar;
pub mod text;

use kode_core::geometry::Rect;

/// A render command for the compositor.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawText {
        text: String,
        x: f32,
        y: f32,
        color: [f32; 4],
    },
    DrawRect {
        rect: Rect,
        color: [f32; 4],
    },
    DrawLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
    },
}

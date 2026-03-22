use kode_core::color::Color;

use crate::RenderCommand;

/// Render line number gutter.
pub fn render_gutter(
    start_line: usize,
    visible_lines: usize,
    active_line: usize,
    x: f32,
    y_start: f32,
    line_height: f32,
    gutter_color: Color,
    active_color: Color,
) -> Vec<RenderCommand> {
    let mut commands = Vec::with_capacity(visible_lines);
    for i in 0..visible_lines {
        let line_num = start_line + i + 1;
        let y = y_start + (i as f32) * line_height;
        let color = if start_line + i == active_line {
            active_color
        } else {
            gutter_color
        };
        commands.push(RenderCommand::DrawText {
            text: format!("{:>4}", line_num),
            x,
            y,
            color: color.to_array(),
        });
    }
    commands
}

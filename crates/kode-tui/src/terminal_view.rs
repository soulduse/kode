use kode_terminal::grid;
use kode_terminal::Terminal;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::colors::to_ratatui_color;

/// Render a terminal emulator into the given area.
pub fn render_terminal(frame: &mut Frame, area: Rect, terminal: &Terminal) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let cells = grid::extract_visible_cells(&terminal.emulator);
    let buf = frame.buffer_mut();

    for (row_idx, row) in cells.iter().enumerate().take(area.height as usize) {
        for (col_idx, cell) in row.iter().enumerate().take(area.width as usize) {
            let x = area.x + col_idx as u16;
            let y = area.y + row_idx as u16;

            if let Some(ratatui_cell) = buf.cell_mut((x, y)) {
                let mut style = Style::default()
                    .fg(to_ratatui_color(&cell.fg))
                    .bg(to_ratatui_color(&cell.bg));

                if cell.bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cell.italic {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                if cell.strikethrough {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }

                ratatui_cell.set_char(cell.ch);
                ratatui_cell.set_style(style);
            }
        }
    }
}

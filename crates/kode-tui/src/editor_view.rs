use kode_editor::document::Document;
use kode_keymap::mode::Mode;
use ratatui::buffer::Buffer as RataBuf;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;

use crate::colors::ThemeStyles;

const GUTTER_WIDTH: u16 = 5;
const SCROLL_OFF: usize = 5;

/// Render an editor document into the given area.
pub fn render_editor(
    frame: &mut Frame,
    area: Rect,
    doc: &Document,
    mode: Mode,
    styles: &ThemeStyles,
) {
    if area.width < GUTTER_WIDTH + 2 || area.height == 0 {
        return;
    }

    let visible_lines = area.height as usize;
    let scroll = doc.scroll_offset();

    // Gutter area
    let gutter_area = Rect {
        x: area.x,
        y: area.y,
        width: GUTTER_WIDTH,
        height: area.height,
    };

    // Text area
    let text_area = Rect {
        x: area.x + GUTTER_WIDTH,
        y: area.y,
        width: area.width - GUTTER_WIDTH - 1, // -1 for scrollbar
        height: area.height,
    };

    let buf = frame.buffer_mut();
    let cursor_line = doc.cursors.primary().line();

    // Fill background
    fill_rect(buf, area, styles.background);

    // Render gutter + text lines
    for i in 0..visible_lines {
        let line_idx = scroll + i;
        let y = area.y + i as u16;

        if line_idx < doc.buffer.line_count() {
            // Gutter: line number
            let is_current = line_idx == cursor_line;
            let gutter_style = if is_current {
                styles.gutter_active
            } else {
                styles.gutter
            };
            let num_str = format!("{:>4} ", line_idx + 1);
            set_string_styled(buf, gutter_area.x, y, &num_str, gutter_style);

            // Current line highlight
            if is_current {
                for x in text_area.x..text_area.x + text_area.width {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(styles.line_highlight);
                    }
                }
            }

            // Text content
            if let Some(line_text) = doc.buffer.line_to_string(line_idx) {
                let text_style = styles.foreground;
                let max_col = text_area.width as usize;
                for (col, ch) in line_text.chars().enumerate() {
                    if col >= max_col {
                        break;
                    }
                    let x = text_area.x + col as u16;
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_char(ch);
                        cell.set_style(text_style);
                    }
                }
            }
        } else {
            // After end of file: show '~'
            set_string_styled(buf, gutter_area.x, y, "    ~", styles.gutter);
        }
    }

    // Cursor positioning
    let cursor = doc.cursors.primary();
    let cursor_row = cursor.line().saturating_sub(scroll);
    if cursor_row < visible_lines {
        let cursor_x = text_area.x + cursor.col() as u16;
        let cursor_y = text_area.y + cursor_row as u16;

        if cursor_x < text_area.x + text_area.width && cursor_y < text_area.y + text_area.height {
            match mode.cursor_style() {
                kode_keymap::mode::CursorStyle::Block => {
                    if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                        cell.set_style(styles.cursor);
                    }
                }
                _ => {
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
    }

    // Scrollbar
    let total_lines = doc.buffer.line_count();
    if total_lines > visible_lines {
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y,
            width: 1,
            height: area.height,
        };
        let mut scrollbar_state =
            ScrollbarState::new(total_lines.saturating_sub(visible_lines)).position(scroll);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }
}

/// Ensure the cursor is visible by adjusting scroll offset.
pub fn ensure_cursor_visible(doc: &mut Document, visible_lines: usize) {
    let cursor_line = doc.cursors.primary().line();
    let scroll = doc.scroll_offset();

    if cursor_line < scroll + SCROLL_OFF {
        doc.set_scroll_offset(cursor_line.saturating_sub(SCROLL_OFF));
    } else if cursor_line + SCROLL_OFF >= scroll + visible_lines {
        doc.set_scroll_offset(cursor_line + SCROLL_OFF + 1 - visible_lines);
    }
}

fn fill_rect(buf: &mut RataBuf, area: Rect, style: Style) {
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(style);
            }
        }
    }
}

fn set_string_styled(buf: &mut RataBuf, x: u16, y: u16, s: &str, style: Style) {
    for (i, ch) in s.chars().enumerate() {
        let cx = x + i as u16;
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_char(ch);
            cell.set_style(style);
        }
    }
}

use kode_keymap::mode::Mode;
use kode_workspace::session::Session;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::colors::ThemeStyles;

/// Render the tab bar at the given area (typically row 0).
pub fn render_tab_bar(frame: &mut Frame, area: Rect, session: &Session, styles: &ThemeStyles) {
    if area.height == 0 {
        return;
    }

    let buf = frame.buffer_mut();

    // Fill background
    for x in area.x..area.x + area.width {
        if let Some(cell) = buf.cell_mut((x, area.y)) {
            cell.set_char(' ');
            cell.set_style(styles.background);
        }
    }

    let mut x = area.x + 1;
    for (idx, tab) in session.tabs.iter().enumerate() {
        let is_active = idx == session.active_tab;
        let style = if is_active {
            styles.tab_active
        } else {
            styles.tab_inactive
        };

        let label = format!(" {} ", tab.name);
        for ch in label.chars() {
            if x >= area.x + area.width {
                break;
            }
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_char(ch);
                cell.set_style(style);
            }
            x += 1;
        }
        x += 1; // spacing
    }
}

/// Render the status line at the given area (typically last row).
pub fn render_status_line(
    frame: &mut Frame,
    area: Rect,
    mode: Mode,
    title: &str,
    is_modified: bool,
    cursor_line: usize,
    cursor_col: usize,
    language: &str,
    command_text: Option<&str>,
    styles: &ThemeStyles,
) {
    if area.height == 0 {
        return;
    }

    let buf = frame.buffer_mut();

    // Fill background
    for x in area.x..area.x + area.width {
        if let Some(cell) = buf.cell_mut((x, area.y)) {
            cell.set_char(' ');
            cell.set_style(styles.background);
        }
    }

    // If in command mode, show command line
    if let Some(cmd) = command_text {
        let cmd_display = format!(":{}", cmd);
        set_string(buf, area.x, area.y, &cmd_display, styles.foreground);
        frame.set_cursor_position((area.x + cmd_display.len() as u16, area.y));
        return;
    }

    let mut x = area.x;

    // Mode indicator
    let mode_style = match mode {
        Mode::Normal => styles.mode_normal,
        Mode::Insert => styles.mode_insert,
        Mode::Visual | Mode::VisualLine | Mode::VisualBlock => styles.mode_visual,
        Mode::Command => styles.mode_command,
        Mode::Replace => styles.mode_insert,
    };
    let mode_text = format!(" {} ", mode.display_name());
    x = set_string(buf, x, area.y, &mode_text, mode_style);
    x = set_string(buf, x, area.y, " ", styles.foreground);

    // File name + modified indicator
    let modified = if is_modified { " [+]" } else { "" };
    let file_info = format!("{}{}", title, modified);
    x = set_string(buf, x, area.y, &file_info, styles.foreground);
    x = set_string(buf, x, area.y, " │ ", styles.gutter);

    // Cursor position
    let pos_text = format!("{}:{}", cursor_line + 1, cursor_col + 1);
    x = set_string(buf, x, area.y, &pos_text, styles.foreground);
    x = set_string(buf, x, area.y, " │ ", styles.gutter);

    // Language
    set_string(buf, x, area.y, language, styles.foreground);
}

/// Draw a border around a pane area.
pub fn render_pane_border(
    frame: &mut Frame,
    area: Rect,
    focused: bool,
    title: &str,
    styles: &ThemeStyles,
) {
    let style = if focused {
        styles.border_focused
    } else {
        styles.border_unfocused
    };

    let buf = frame.buffer_mut();

    // Top border
    if area.height > 0 {
        if let Some(cell) = buf.cell_mut((area.x, area.y)) {
            cell.set_char('┌');
            cell.set_style(style);
        }
        for x in area.x + 1..area.x + area.width.saturating_sub(1) {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_char('─');
                cell.set_style(style);
            }
        }
        if area.width > 1 {
            if let Some(cell) = buf.cell_mut((area.x + area.width - 1, area.y)) {
                cell.set_char('┐');
                cell.set_style(style);
            }
        }

        // Title in top border
        if !title.is_empty() && area.width > 4 {
            let title_display = format!(" {} ", title);
            let start_x = area.x + 2;
            for (i, ch) in title_display.chars().enumerate() {
                let x = start_x + i as u16;
                if x >= area.x + area.width - 1 {
                    break;
                }
                if let Some(cell) = buf.cell_mut((x, area.y)) {
                    cell.set_char(ch);
                    cell.set_style(style);
                }
            }
        }
    }

    // Bottom border
    if area.height > 1 {
        let bottom_y = area.y + area.height - 1;
        if let Some(cell) = buf.cell_mut((area.x, bottom_y)) {
            cell.set_char('└');
            cell.set_style(style);
        }
        for x in area.x + 1..area.x + area.width.saturating_sub(1) {
            if let Some(cell) = buf.cell_mut((x, bottom_y)) {
                cell.set_char('─');
                cell.set_style(style);
            }
        }
        if area.width > 1 {
            if let Some(cell) = buf.cell_mut((area.x + area.width - 1, bottom_y)) {
                cell.set_char('┘');
                cell.set_style(style);
            }
        }
    }

    // Left/right borders
    for y in area.y + 1..area.y + area.height.saturating_sub(1) {
        if let Some(cell) = buf.cell_mut((area.x, y)) {
            cell.set_char('│');
            cell.set_style(style);
        }
        if area.width > 1 {
            if let Some(cell) = buf.cell_mut((area.x + area.width - 1, y)) {
                cell.set_char('│');
                cell.set_style(style);
            }
        }
    }
}

/// Inner area of a bordered pane.
pub fn inner_rect(area: Rect) -> Rect {
    if area.width < 3 || area.height < 3 {
        return Rect::default();
    }
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width - 2,
        height: area.height - 2,
    }
}

fn set_string(
    buf: &mut ratatui::buffer::Buffer,
    x: u16,
    y: u16,
    s: &str,
    style: Style,
) -> u16 {
    let mut cx = x;
    for ch in s.chars() {
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_char(ch);
            cell.set_style(style);
        }
        cx += 1;
    }
    cx
}

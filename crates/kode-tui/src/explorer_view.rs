use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::colors::ThemeStyles;
use crate::file_explorer::FileExplorer;

/// Render the file explorer tree into the given area.
pub fn render_explorer(
    frame: &mut Frame,
    area: Rect,
    explorer: &FileExplorer,
    focused: bool,
    styles: &ThemeStyles,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let buf = frame.buffer_mut();

    // Fill background
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(styles.background);
            }
        }
    }

    // Reserve bottom row(s) for input prompt or delete confirmation
    let prompt_height = if explorer.input_mode.is_some() || explorer.confirm_delete.is_some() {
        1u16
    } else {
        0
    };
    let tree_height = area.height.saturating_sub(prompt_height) as usize;

    // Render tree entries
    for i in 0..tree_height {
        let entry_idx = explorer.scroll_offset + i;
        let y = area.y + i as u16;

        if entry_idx >= explorer.entries.len() {
            break;
        }

        let entry = &explorer.entries[entry_idx];
        let is_cursor = entry_idx == explorer.cursor;

        // Cursor line highlight
        if is_cursor && focused {
            for x in area.x..area.x + area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_style(styles.line_highlight);
                }
            }
        }

        // Build display string: indent + arrow + icon + name
        let indent = "  ".repeat(entry.depth);
        let arrow = if entry.is_dir {
            if entry.expanded { "▾ " } else { "▸ " }
        } else {
            "  "
        };
        let icon = if entry.is_dir {
            file_icon_dir()
        } else {
            file_icon(&entry.name)
        };
        let display = format!("{}{}{} {}", indent, arrow, icon, entry.name);

        // Choose style
        let style = if entry.is_dir {
            styles.function // blue for directories
        } else {
            styles.foreground
        };
        let style = if is_cursor && focused {
            style.patch(styles.line_highlight)
        } else {
            style
        };

        // Write to buffer
        let max_x = area.x + area.width;
        let mut x = area.x;
        for ch in display.chars() {
            if x >= max_x {
                break;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(ch);
                cell.set_style(style);
            }
            x += 1;
        }
    }

    // Render prompt at bottom if active
    if let Some(ref mode) = explorer.input_mode {
        let prompt_y = area.y + area.height - 1;
        let prompt_text = match mode {
            crate::file_explorer::InputMode::Create { is_dir, .. } => {
                if *is_dir {
                    format!("mkdir: {}_", explorer.input_buffer)
                } else {
                    format!("new: {}_", explorer.input_buffer)
                }
            }
            crate::file_explorer::InputMode::Rename { .. } => {
                format!("rename: {}_", explorer.input_buffer)
            }
        };

        set_string(buf, area.x, prompt_y, &prompt_text, styles.info, area.width);
    }

    // Render delete confirmation at bottom
    if let Some(idx) = explorer.confirm_delete {
        let prompt_y = area.y + area.height - 1;
        let name = explorer
            .entries
            .get(idx)
            .map(|e| e.name.as_str())
            .unwrap_or("?");
        let prompt = format!("Delete {}? (y/n)", name);
        set_string(buf, area.x, prompt_y, &prompt, styles.error, area.width);
    }
}

fn set_string(
    buf: &mut ratatui::buffer::Buffer,
    x: u16,
    y: u16,
    s: &str,
    style: Style,
    max_width: u16,
) {
    let mut cx = x;
    let max_x = x + max_width;
    for ch in s.chars() {
        if cx >= max_x {
            break;
        }
        if let Some(cell) = buf.cell_mut((cx, y)) {
            cell.set_char(ch);
            cell.set_style(style);
        }
        cx += 1;
    }
}

fn file_icon_dir() -> &'static str {
    "\u{1F4C1}" // folder icon, but use simple char for TUI width
}

fn file_icon(name: &str) -> &'static str {
    match name.rsplit('.').next() {
        Some("rs") => "\u{1F980}",  // crab for Rust — but may be wide
        Some("toml") => "\u{2699}",
        Some("md") => "\u{1F4DD}",
        Some("kt") | Some("kts") => "K",
        Some("java") => "J",
        Some("py") => "P",
        Some("js") | Some("ts") | Some("tsx") => "J",
        Some("yml") | Some("yaml") => "Y",
        Some("json") => "{",
        Some("lock") => "\u{1F512}",
        _ => " ",
    }
}

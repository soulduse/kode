use kode_core::color::ThemeColors;
use kode_core::geometry::Rect as KodeRect;
use kode_workspace::pane::PaneContent;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::chrome;
use crate::colors::{theme_styles, ThemeStyles};
use crate::editor_view;
use crate::terminal_view;

/// Main draw function — renders the entire TUI frame.
pub fn draw(frame: &mut Frame, app: &crate::AppView) {
    let full = frame.area();
    let styles = theme_styles(&ThemeColors::default());

    // Fill entire screen with background
    let buf = frame.buffer_mut();
    for y in full.y..full.y + full.height {
        for x in full.x..full.x + full.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(styles.background);
            }
        }
    }

    // Tab bar: row 0
    if full.height > 2 {
        let tab_area = Rect::new(full.x, full.y, full.width, 1);
        chrome::render_tab_bar(frame, tab_area, app.session, &styles);
    }

    // Status line: last row
    if full.height > 1 {
        let status_area = Rect::new(full.x, full.y + full.height - 1, full.width, 1);
        render_status(frame, status_area, app, &styles);
    }

    // Pane area: rows 1..height-1
    if full.height <= 2 {
        return;
    }

    let pane_rects = &app.pane_rects;
    for (pane_id, kode_rect) in pane_rects {
        let tui_rect = kode_to_ratatui_rect(kode_rect, full);
        if tui_rect.width == 0 || tui_rect.height == 0 {
            continue;
        }

        let pane = match app.panes.get(pane_id) {
            Some(p) => p,
            None => continue,
        };

        // Pane border
        let title = match pane.content {
            PaneContent::Editor(doc_id) => {
                app.documents
                    .get(&doc_id)
                    .map(|d| d.title())
                    .unwrap_or_default()
            }
            PaneContent::Terminal(_) => "terminal".to_string(),
            PaneContent::BeanExplorer => "beans".to_string(),
            PaneContent::EndpointExplorer => "endpoints".to_string(),
        };

        chrome::render_pane_border(frame, tui_rect, pane.focused, &title, &styles);
        let inner = chrome::inner_rect(tui_rect);
        if inner.width == 0 || inner.height == 0 {
            continue;
        }

        match pane.content {
            PaneContent::Editor(doc_id) => {
                if let Some(doc) = app.documents.get(&doc_id) {
                    editor_view::render_editor(frame, inner, doc, app.mode, &styles);
                }
            }
            PaneContent::Terminal(term_id) => {
                if let Some(terminal) = app.terminals.get(&term_id) {
                    terminal_view::render_terminal(frame, inner, terminal);
                }
            }
            _ => {}
        }
    }
}

fn render_status(frame: &mut Frame, area: Rect, app: &crate::AppView, styles: &ThemeStyles) {
    // Get info from focused pane
    let (title, is_modified, cursor_line, cursor_col, language) =
        if let Some(pane) = app.panes.get(&app.focused_pane) {
            match pane.content {
                PaneContent::Editor(doc_id) => {
                    if let Some(doc) = app.documents.get(&doc_id) {
                        (
                            doc.title(),
                            doc.is_modified(),
                            doc.cursors.primary().line(),
                            doc.cursors.primary().col(),
                            doc.language.as_deref().unwrap_or("plaintext").to_string(),
                        )
                    } else {
                        default_status()
                    }
                }
                PaneContent::Terminal(_) => {
                    ("terminal".to_string(), false, 0, 0, "shell".to_string())
                }
                _ => default_status(),
            }
        } else {
            default_status()
        };

    chrome::render_status_line(
        frame,
        area,
        app.mode,
        &title,
        is_modified,
        cursor_line,
        cursor_col,
        &language,
        app.command_text,
        styles,
    );
}

fn default_status() -> (String, bool, usize, usize, String) {
    ("[untitled]".to_string(), false, 0, 0, "plaintext".to_string())
}

/// Convert kode f32 rect to ratatui u16 rect, offset by tab bar.
fn kode_to_ratatui_rect(rect: &KodeRect, full: Rect) -> Rect {
    let x = rect.x() as u16;
    let y = rect.y() as u16 + 1; // +1 for tab bar
    let w = rect.width() as u16;
    let h = rect.height() as u16;

    // Clamp to screen bounds
    let max_x = full.x + full.width;
    let max_y = full.y + full.height.saturating_sub(1); // -1 for status line
    let x = x.min(max_x);
    let y = y.min(max_y);
    let w = w.min(max_x.saturating_sub(x));
    let h = h.min(max_y.saturating_sub(y));

    Rect::new(x, y, w, h)
}

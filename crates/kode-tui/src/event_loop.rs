use std::io;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use kode_core::error::KodeResult;
use kode_core::event::{KeyCode, KeyEvent, Modifiers};
use kode_terminal::input::key_to_escape;
use kode_workspace::pane::PaneContent;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::editor_view;
use crate::event::{crossterm_to_kode_key, crossterm_to_kode_mouse};
use crate::ui;
use crate::AppView;

const TICK_RATE_MS: u64 = 50;

/// Run the TUI event loop.
pub fn run(app: &mut crate::TuiApp) -> KodeResult<()> {
    // Setup terminal
    enable_raw_mode().map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)
        .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?;

    // Set initial viewport
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    app.inner.set_viewport(cols, rows);

    let result = run_loop(&mut terminal, app);

    // Cleanup
    let _ = disable_raw_mode();
    let _ = execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    );
    let _ = terminal.show_cursor();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut crate::TuiApp,
) -> KodeResult<()> {
    loop {
        // Ensure cursor visible for all editor panes
        ensure_all_cursors_visible(app);

        // Render
        let view = app.view();
        terminal
            .draw(|frame| ui::draw(frame, &view))
            .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?;

        // Poll for events
        if event::poll(Duration::from_millis(TICK_RATE_MS))
            .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?
        {
            match event::read()
                .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?
            {
                Event::Key(key) => {
                    if let Some(kode_key) = crossterm_to_kode_key(key) {
                        handle_key(app, kode_key);
                    }
                }
                Event::Mouse(mouse) => {
                    if let Some(_kode_mouse) = crossterm_to_kode_mouse(mouse) {
                        // Mouse handling — future enhancement
                    }
                }
                Event::Resize(w, h) => {
                    app.inner.set_viewport(w, h);
                    resize_terminals(app);
                }
                _ => {}
            }
        }

        // Process terminal PTY output
        for terminal_inst in app.inner.terminals.values_mut() {
            let _ = terminal_inst.process_output();
        }

        if !app.inner.is_running() {
            break;
        }
    }

    Ok(())
}

fn handle_key(app: &mut crate::TuiApp, key: KeyEvent) {
    // Check if focused pane is a terminal
    if let Some(pane) = app.inner.panes.get(&app.inner.focused_pane) {
        if let PaneContent::Terminal(term_id) = pane.content {
            // In terminal pane: forward keys to PTY, except Ctrl-A (workspace prefix)
            if key.code == KeyCode::Char('a') && key.modifiers.contains(Modifiers::CTRL) {
                // Let the key parser handle workspace prefix
                app.inner.handle_key_event(key);
                return;
            }

            // Forward key to terminal
            if let Some(escape_bytes) = key_to_escape(&key) {
                if let Some(terminal) = app.inner.terminals.get_mut(&term_id) {
                    let _ = terminal.write_input(&escape_bytes);
                }
            }
            return;
        }
    }

    // Editor pane: go through key parser
    app.inner.handle_key_event(key);
}

fn ensure_all_cursors_visible(app: &mut crate::TuiApp) {
    let viewport = app.inner.viewport;
    let pane_height = (viewport.height() as usize).saturating_sub(2); // borders

    for pane in app.inner.panes.values() {
        if let PaneContent::Editor(doc_id) = pane.content {
            if let Some(doc) = app.inner.documents.get_mut(&doc_id) {
                editor_view::ensure_cursor_visible(doc, pane_height.max(1));
            }
        }
    }
}

fn resize_terminals(app: &mut crate::TuiApp) {
    let pane_rects = app.inner.pane_rects();
    for (pane_id, rect) in &pane_rects {
        if let Some(pane) = app.inner.panes.get(pane_id) {
            if let PaneContent::Terminal(term_id) = pane.content {
                if let Some(terminal) = app.inner.terminals.get_mut(&term_id) {
                    // Subtract 2 for borders
                    let rows = (rect.height() as u16).saturating_sub(2).max(1);
                    let cols = (rect.width() as u16).saturating_sub(2).max(1);
                    terminal.resize(rows, cols);
                }
            }
        }
    }
}

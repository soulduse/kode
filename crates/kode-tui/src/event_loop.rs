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
    if let Some(pane) = app.inner.panes.get(&app.inner.focused_pane) {
        match pane.content {
            PaneContent::Terminal(term_id) => {
                // In terminal pane: forward keys to PTY, except Ctrl-A (workspace prefix)
                if key.code == KeyCode::Char('a') && key.modifiers.contains(Modifiers::CTRL) {
                    app.inner.handle_key_event(key);
                    return;
                }
                if let Some(escape_bytes) = key_to_escape(&key) {
                    if let Some(terminal) = app.inner.terminals.get_mut(&term_id) {
                        let _ = terminal.write_input(&escape_bytes);
                    }
                }
                return;
            }
            PaneContent::FileExplorer(explorer_id) => {
                // Workspace prefix (Ctrl-A) goes through key parser
                if key.code == KeyCode::Char('a') && key.modifiers.contains(Modifiers::CTRL) {
                    app.inner.handle_key_event(key);
                    return;
                }
                handle_explorer_key(app, explorer_id, key);
                return;
            }
            _ => {}
        }
    }

    // Editor pane: go through key parser
    app.inner.handle_key_event(key);
}

fn handle_explorer_key(app: &mut crate::TuiApp, explorer_id: usize, key: KeyEvent) {
    // Check if explorer is in input mode (create/rename)
    let in_input_mode = app.inner.explorers.get(&explorer_id)
        .map(|e| e.input_mode.is_some())
        .unwrap_or(false);

    let in_confirm_delete = app.inner.explorers.get(&explorer_id)
        .map(|e| e.confirm_delete.is_some())
        .unwrap_or(false);

    if in_confirm_delete {
        match key.code {
            KeyCode::Char('y') => {
                if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                    let _ = explorer.confirm_delete_yes();
                }
            }
            _ => {
                if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                    explorer.cancel_input();
                }
            }
        }
        return;
    }

    if in_input_mode {
        match key.code {
            KeyCode::Escape => {
                if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                    explorer.cancel_input();
                }
            }
            KeyCode::Enter => {
                let result = app.inner.explorers.get_mut(&explorer_id)
                    .and_then(|e| e.confirm_input().ok())
                    .flatten();
                if let Some(path) = result {
                    app.inner.open_file_from_explorer(path);
                }
            }
            KeyCode::Backspace => {
                if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                    explorer.input_buffer.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                    explorer.input_buffer.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    // Normal explorer navigation
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.move_cursor_down();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.move_cursor_up();
            }
        }
        KeyCode::Char('l') | KeyCode::Enter => {
            // Directory: expand, File: open in editor
            let action = app.inner.explorers.get(&explorer_id).and_then(|e| {
                e.selected_entry().map(|entry| {
                    if entry.is_dir {
                        ExplorerAction::ToggleExpand
                    } else {
                        ExplorerAction::OpenFile(entry.path.clone())
                    }
                })
            });
            match action {
                Some(ExplorerAction::ToggleExpand) => {
                    if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                        explorer.toggle_expand();
                    }
                }
                Some(ExplorerAction::OpenFile(path)) => {
                    app.inner.open_file_from_explorer(path);
                }
                None => {}
            }
        }
        KeyCode::Char('h') | KeyCode::Backspace => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.collapse_current();
            }
        }
        KeyCode::Char('a') => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.start_create(false);
            }
        }
        KeyCode::Char('A') => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.start_create(true);
            }
        }
        KeyCode::Char('d') => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.request_delete();
            }
        }
        KeyCode::Char('r') => {
            if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                explorer.start_rename();
            }
        }
        KeyCode::Char('q') | KeyCode::Escape => {
            app.inner.toggle_explorer();
        }
        _ => {}
    }
}

enum ExplorerAction {
    ToggleExpand,
    OpenFile(std::path::PathBuf),
}

fn ensure_all_cursors_visible(app: &mut crate::TuiApp) {
    let viewport = app.inner.viewport;
    let pane_height = (viewport.height() as usize).saturating_sub(2); // borders

    for pane in app.inner.panes.values() {
        match pane.content {
            PaneContent::Editor(doc_id) => {
                if let Some(doc) = app.inner.documents.get_mut(&doc_id) {
                    editor_view::ensure_cursor_visible(doc, pane_height.max(1));
                }
            }
            PaneContent::FileExplorer(explorer_id) => {
                if let Some(explorer) = app.inner.explorers.get_mut(&explorer_id) {
                    explorer.ensure_cursor_visible(pane_height.max(1));
                }
            }
            _ => {}
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

pub mod app_state;
pub mod file_explorer;

pub use app_state::{AppState, create_app_state};
pub use file_explorer::FileExplorer;

use std::collections::HashMap;

use kode_core::geometry::Rect;
use kode_editor::document::Document;
use kode_keymap::mode::Mode;
use kode_terminal::Terminal;
use kode_workspace::pane::{Pane, PaneId};
use kode_workspace::session::Session;

/// Read-only view of app state for rendering.
pub struct AppView<'a> {
    pub session: &'a Session,
    pub panes: &'a HashMap<PaneId, Pane>,
    pub documents: &'a HashMap<usize, Document>,
    pub terminals: &'a HashMap<usize, Terminal>,
    pub explorers: &'a HashMap<usize, FileExplorer>,
    pub focused_pane: PaneId,
    pub mode: Mode,
    pub pane_rects: Vec<(PaneId, Rect)>,
    pub command_text: Option<&'a str>,
}

/// Create an AppView snapshot from an AppState.
pub fn create_app_view(state: &AppState) -> AppView<'_> {
    let pane_rects = state.pane_rects();
    let command_text = if state.mode() == Mode::Command {
        Some(state.command_text())
    } else {
        None
    };

    AppView {
        session: &state.session,
        panes: &state.panes,
        documents: &state.documents,
        terminals: &state.terminals,
        explorers: &state.explorers,
        focused_pane: state.focused_pane,
        mode: state.mode(),
        pane_rects,
        command_text,
    }
}

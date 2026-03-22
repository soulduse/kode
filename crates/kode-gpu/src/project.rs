use std::collections::HashMap;
use std::path::PathBuf;

use kode_editor::document::Document;
use kode_state::{create_app_state, AppState, FileExplorer};
use kode_workspace::layout::{Direction, LayoutNode};
use kode_workspace::pane::{Pane, PaneId};
use kode_workspace::session::Session;
use kode_workspace::tab::Tab;

/// Build an AppState for editing within the given project directory.
pub fn build_editor_state(project_path: PathBuf) -> AppState {
    let mut documents = HashMap::new();
    documents.insert(0, Document::new());

    let mut explorers = HashMap::new();
    explorers.insert(0, FileExplorer::new(0, project_path));

    let explorer_pane_id: PaneId = 0;
    let editor_pane_id: PaneId = 1;

    let mut panes = HashMap::new();
    panes.insert(explorer_pane_id, Pane::file_explorer(explorer_pane_id, 0));
    let mut editor_pane = Pane::editor(editor_pane_id, 0);
    editor_pane.focused = true;
    panes.insert(editor_pane_id, editor_pane);

    let layout = LayoutNode::Split {
        direction: Direction::Vertical,
        ratio: 0.25,
        first: Box::new(LayoutNode::Leaf(explorer_pane_id)),
        second: Box::new(LayoutNode::Leaf(editor_pane_id)),
    };
    let mut tab = Tab::new(0, "main".into(), editor_pane_id);
    tab.layout = layout;
    let session = Session::new(tab);

    create_app_state(
        documents,
        HashMap::new(),
        explorers,
        panes,
        session,
        editor_pane_id,
    )
}

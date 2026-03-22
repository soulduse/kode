/// Unique pane identifier.
pub type PaneId = usize;

/// Content type of a pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    Editor(usize),
    Terminal(usize),
}

/// A pane in the workspace.
#[derive(Debug, Clone)]
pub struct Pane {
    pub id: PaneId,
    pub content: PaneContent,
    pub focused: bool,
}

impl Pane {
    pub fn editor(id: PaneId, doc_id: usize) -> Self {
        Self {
            id,
            content: PaneContent::Editor(doc_id),
            focused: false,
        }
    }

    pub fn terminal(id: PaneId, term_id: usize) -> Self {
        Self {
            id,
            content: PaneContent::Terminal(term_id),
            focused: false,
        }
    }
}

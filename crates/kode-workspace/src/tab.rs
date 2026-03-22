use crate::layout::LayoutNode;
use crate::pane::PaneId;

/// A tab contains a layout of panes.
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub layout: LayoutNode,
}

impl Tab {
    pub fn new(id: usize, name: String, initial_pane: PaneId) -> Self {
        Self {
            id,
            name,
            layout: LayoutNode::Leaf(initial_pane),
        }
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::layout::{Direction, LayoutNode};
use crate::pane::{Pane, PaneContent, PaneId};
use crate::session::Session;
use crate::tab::Tab;

/// Serializable session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub tabs: Vec<TabState>,
    pub active_tab: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub name: String,
    pub layout: LayoutNode,
    pub panes: Vec<PaneState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneState {
    pub id: PaneId,
    #[serde(rename = "type")]
    pub pane_type: PaneType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PaneType {
    Editor,
    Terminal,
}

/// Build a SessionState from the current session and pane map.
pub fn save_session(
    session: &Session,
    panes: &HashMap<PaneId, Pane>,
    editor_files: &HashMap<usize, Option<PathBuf>>,
    terminal_cwds: &HashMap<usize, PathBuf>,
) -> SessionState {
    let mut tabs = Vec::new();
    for tab in &session.tabs {
        let pane_ids = tab.layout.pane_ids();
        let mut pane_states = Vec::new();

        for &pid in &pane_ids {
            if let Some(pane) = panes.get(&pid) {
                let state = match pane.content {
                    PaneContent::Editor(doc_id) => PaneState {
                        id: pid,
                        pane_type: PaneType::Editor,
                        file: editor_files.get(&doc_id).cloned().flatten(),
                        cwd: None,
                    },
                    PaneContent::Terminal(term_id) => PaneState {
                        id: pid,
                        pane_type: PaneType::Terminal,
                        file: None,
                        cwd: terminal_cwds.get(&term_id).cloned(),
                    },
                    PaneContent::BeanExplorer | PaneContent::EndpointExplorer => {
                        continue; // Skip non-persistent panes
                    }
                };
                pane_states.push(state);
            }
        }

        tabs.push(TabState {
            name: tab.name.clone(),
            layout: tab.layout.clone(),
            panes: pane_states,
        });
    }

    SessionState {
        tabs,
        active_tab: session.active_tab,
    }
}

/// Save session state to a JSON file.
pub fn save_to_file(state: &SessionState, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)
}

/// Load session state from a JSON file.
pub fn load_from_file(path: &Path) -> std::io::Result<SessionState> {
    let json = std::fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Default session file path.
pub fn default_session_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".config")
        });
    config_dir.join("kode").join("sessions").join("last.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Direction;

    #[test]
    fn serialize_roundtrip() {
        let mut layout = LayoutNode::Leaf(0);
        layout.split(0, 1, Direction::Vertical);

        let state = SessionState {
            tabs: vec![TabState {
                name: "main".into(),
                layout,
                panes: vec![
                    PaneState {
                        id: 0,
                        pane_type: PaneType::Editor,
                        file: Some(PathBuf::from("/path/to/file.kt")),
                        cwd: None,
                    },
                    PaneState {
                        id: 1,
                        pane_type: PaneType::Terminal,
                        file: None,
                        cwd: Some(PathBuf::from("/home/user")),
                    },
                ],
            }],
            active_tab: 0,
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        let restored: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.tabs.len(), 1);
        assert_eq!(restored.tabs[0].panes.len(), 2);
        assert_eq!(restored.tabs[0].panes[0].pane_type, PaneType::Editor);
        assert_eq!(restored.tabs[0].panes[1].pane_type, PaneType::Terminal);
    }

    #[test]
    fn save_session_from_state() {
        let tab = Tab::new(0, "test".into(), 0);
        let session = Session::new(tab);

        let mut panes = HashMap::new();
        panes.insert(0, Pane::editor(0, 0));

        let mut editor_files = HashMap::new();
        editor_files.insert(0usize, Some(PathBuf::from("test.kt")));

        let terminal_cwds = HashMap::new();

        let state = save_session(&session, &panes, &editor_files, &terminal_cwds);
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.tabs[0].panes.len(), 1);
        assert_eq!(
            state.tabs[0].panes[0].file,
            Some(PathBuf::from("test.kt"))
        );
    }
}

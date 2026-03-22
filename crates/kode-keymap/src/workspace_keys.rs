use kode_core::event::{KeyCode, KeyEvent, Modifiers};

/// Actions triggered by workspace (tmux-style) key bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceAction {
    SplitVertical,
    SplitHorizontal,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    ResizeLeft(u32),
    ResizeRight(u32),
    ResizeUp(u32),
    ResizeDown(u32),
    NextTab,
    PrevTab,
    NewTab,
    CloseTab,
    ClosePane,
    ZoomPane,
    NewTerminal,
    Detach,
    Equalize,
}

/// Parse a key event after the workspace prefix (Ctrl-A) was pressed.
pub fn parse_workspace_key(key: &KeyEvent) -> Option<WorkspaceAction> {
    match key.code {
        // Split
        KeyCode::Char('|') | KeyCode::Char('\\') => Some(WorkspaceAction::SplitVertical),
        KeyCode::Char('-') => Some(WorkspaceAction::SplitHorizontal),

        // Focus navigation
        KeyCode::Char('h') | KeyCode::Left => Some(WorkspaceAction::FocusLeft),
        KeyCode::Char('j') | KeyCode::Down => Some(WorkspaceAction::FocusDown),
        KeyCode::Char('k') | KeyCode::Up => Some(WorkspaceAction::FocusUp),
        KeyCode::Char('l') | KeyCode::Right => Some(WorkspaceAction::FocusRight),

        // Resize (Shift + hjkl)
        KeyCode::Char('H') => Some(WorkspaceAction::ResizeLeft(1)),
        KeyCode::Char('J') => Some(WorkspaceAction::ResizeDown(1)),
        KeyCode::Char('K') => Some(WorkspaceAction::ResizeUp(1)),
        KeyCode::Char('L') => Some(WorkspaceAction::ResizeRight(1)),

        // Tabs
        KeyCode::Char('c') => Some(WorkspaceAction::NewTab),
        KeyCode::Char('n') => Some(WorkspaceAction::NextTab),
        KeyCode::Char('p') => Some(WorkspaceAction::PrevTab),
        KeyCode::Char('&') => Some(WorkspaceAction::CloseTab),

        // Pane management
        KeyCode::Char('x') => Some(WorkspaceAction::ClosePane),
        KeyCode::Char('z') => Some(WorkspaceAction::ZoomPane),
        KeyCode::Char('t') => Some(WorkspaceAction::NewTerminal),
        KeyCode::Char('=') => Some(WorkspaceAction::Equalize),

        // Session
        KeyCode::Char('d') => Some(WorkspaceAction::Detach),

        _ => None,
    }
}

/// Check if a key event is the workspace prefix (Ctrl-A).
pub fn is_prefix(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('a') && key.modifiers.contains(Modifiers::CTRL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_detection() {
        let key = KeyEvent::new(KeyCode::Char('a'), Modifiers::CTRL);
        assert!(is_prefix(&key));

        let key = KeyEvent::plain(KeyCode::Char('a'));
        assert!(!is_prefix(&key));
    }

    #[test]
    fn split_keys() {
        let key = KeyEvent::plain(KeyCode::Char('|'));
        assert_eq!(
            parse_workspace_key(&key),
            Some(WorkspaceAction::SplitVertical)
        );

        let key = KeyEvent::plain(KeyCode::Char('-'));
        assert_eq!(
            parse_workspace_key(&key),
            Some(WorkspaceAction::SplitHorizontal)
        );
    }

    #[test]
    fn focus_keys() {
        let key = KeyEvent::plain(KeyCode::Char('h'));
        assert_eq!(
            parse_workspace_key(&key),
            Some(WorkspaceAction::FocusLeft)
        );
    }

    #[test]
    fn resize_keys() {
        let key = KeyEvent::plain(KeyCode::Char('H'));
        assert_eq!(
            parse_workspace_key(&key),
            Some(WorkspaceAction::ResizeLeft(1))
        );
    }

    #[test]
    fn tab_keys() {
        assert_eq!(
            parse_workspace_key(&KeyEvent::plain(KeyCode::Char('c'))),
            Some(WorkspaceAction::NewTab)
        );
        assert_eq!(
            parse_workspace_key(&KeyEvent::plain(KeyCode::Char('n'))),
            Some(WorkspaceAction::NextTab)
        );
    }

    #[test]
    fn unknown_key() {
        let key = KeyEvent::plain(KeyCode::Char('q'));
        assert_eq!(parse_workspace_key(&key), None);
    }
}

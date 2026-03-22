use kode_core::event::{EditorCommand, KodeEvent};

use crate::abi::PluginEvent;

/// Convert a KodeEvent into a PluginEvent if applicable.
pub fn to_plugin_event(event: &KodeEvent) -> Option<PluginEvent> {
    match event {
        KodeEvent::Command(cmd) => command_to_plugin_event(cmd),
        KodeEvent::Key(key) => {
            use kode_core::event::KeyCode;
            match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                | KeyCode::Home | KeyCode::End | KeyCode::PageUp | KeyCode::PageDown => {
                    Some(PluginEvent {
                        event_type: "cursor_move".into(),
                        uri: None,
                        language: None,
                        line: None,
                        content_changed: false,
                    })
                }
                _ => None,
            }
        }
        KodeEvent::Tick => Some(PluginEvent {
            event_type: "tick".into(),
            uri: None,
            language: None,
            line: None,
            content_changed: false,
        }),
        _ => None,
    }
}

fn command_to_plugin_event(cmd: &EditorCommand) -> Option<PluginEvent> {
    match cmd {
        EditorCommand::Open(path) => Some(PluginEvent {
            event_type: "buffer_open".into(),
            uri: Some(format!("file://{}", path.display())),
            language: None,
            line: None,
            content_changed: false,
        }),
        EditorCommand::InsertChar(_)
        | EditorCommand::NewLine
        | EditorCommand::DeleteForward
        | EditorCommand::DeleteBackward => Some(PluginEvent {
            event_type: "buffer_change".into(),
            uri: None,
            language: None,
            line: None,
            content_changed: true,
        }),
        EditorCommand::Save => Some(PluginEvent {
            event_type: "buffer_save".into(),
            uri: None,
            language: None,
            line: None,
            content_changed: false,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kode_core::event::KeyEvent;
    use kode_core::event::{KeyCode, Modifiers};

    #[test]
    fn tick_converts() {
        let event = to_plugin_event(&KodeEvent::Tick);
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, "tick");
    }

    #[test]
    fn cursor_move_converts() {
        let key = KeyEvent::plain(KeyCode::Down);
        let event = to_plugin_event(&KodeEvent::Key(key));
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, "cursor_move");
    }

    #[test]
    fn insert_char_converts() {
        let event = to_plugin_event(&KodeEvent::Command(EditorCommand::InsertChar('a')));
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, "buffer_change");
    }

    #[test]
    fn quit_does_not_convert() {
        let event = to_plugin_event(&KodeEvent::Quit);
        assert!(event.is_none());
    }
}

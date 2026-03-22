use serde::{Deserialize, Serialize};

/// Event sent from host to plugin (JSON serialized).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default)]
    pub content_changed: bool,
}

/// Response from plugin to host (JSON deserialized).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginResponse {
    #[serde(default)]
    pub decorations: Vec<Decoration>,
}

/// A visual decoration produced by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration {
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_end: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
}

fn default_style() -> String {
    "foreground".into()
}

/// Plugin metadata returned by kode_plugin_info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Known event type strings for plugin subscription.
pub mod event_types {
    pub const BUFFER_OPEN: &str = "buffer_open";
    pub const BUFFER_CHANGE: &str = "buffer_change";
    pub const BUFFER_SAVE: &str = "buffer_save";
    pub const CURSOR_MOVE: &str = "cursor_move";
    pub const TICK: &str = "tick";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_plugin_event() {
        let event = PluginEvent {
            event_type: "buffer_change".into(),
            uri: Some("file:///test.kt".into()),
            language: Some("kotlin".into()),
            line: Some(42),
            content_changed: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("buffer_change"));
        assert!(json.contains("file:///test.kt"));
    }

    #[test]
    fn deserialize_plugin_response() {
        let json = r##"{"decorations":[{"line":10,"color":"#fab387","annotation":"TODO: fix this","side":"right","style":"foreground"}]}"##;
        let resp: PluginResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.decorations.len(), 1);
        assert_eq!(resp.decorations[0].line, 10);
        assert_eq!(resp.decorations[0].annotation.as_deref(), Some("TODO: fix this"));
    }

    #[test]
    fn empty_response() {
        let json = r#"{"decorations":[]}"#;
        let resp: PluginResponse = serde_json::from_str(json).unwrap();
        assert!(resp.decorations.is_empty());
    }
}

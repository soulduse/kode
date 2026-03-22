use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Plugin manifest parsed from plugin.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    #[serde(default)]
    pub events: EventConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    /// Directory where the manifest was loaded from.
    #[serde(skip)]
    pub base_dir: PathBuf,
}

/// Plugin metadata section.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_wasm")]
    pub wasm: String,
}

/// Event subscription configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventConfig {
    #[serde(default)]
    pub subscribe: Vec<String>,
}

/// Resource limits for the plugin.
#[derive(Debug, Clone, Deserialize)]
pub struct LimitsConfig {
    #[serde(default = "default_memory")]
    pub max_memory_mb: u32,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: default_memory(),
            timeout_ms: default_timeout(),
        }
    }
}

fn default_wasm() -> String {
    "plugin.wasm".into()
}

fn default_memory() -> u32 {
    64
}

fn default_timeout() -> u64 {
    200
}

impl PluginManifest {
    /// Load a manifest from a plugin.toml file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let mut manifest: Self = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
        manifest.base_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        Ok(manifest)
    }

    /// Get the full path to the WASM binary.
    pub fn wasm_path(&self) -> PathBuf {
        self.base_dir.join(&self.plugin.wasm)
    }

    /// Check if the plugin subscribes to a given event type.
    pub fn subscribes_to(&self, event_type: &str) -> bool {
        self.events.subscribe.iter().any(|s| s == event_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest() {
        let toml = r#"
[plugin]
name = "todo-highlighter"
version = "0.1.0"
description = "Highlights TODO comments"

[events]
subscribe = ["buffer_open", "buffer_change"]

[limits]
max_memory_mb = 32
timeout_ms = 100
"#;
        let manifest: PluginManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.plugin.name, "todo-highlighter");
        assert_eq!(manifest.events.subscribe.len(), 2);
        assert_eq!(manifest.limits.max_memory_mb, 32);
        assert_eq!(manifest.limits.timeout_ms, 100);
    }

    #[test]
    fn defaults() {
        let toml = r#"
[plugin]
name = "minimal"
version = "0.1.0"
"#;
        let manifest: PluginManifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.plugin.wasm, "plugin.wasm");
        assert_eq!(manifest.limits.max_memory_mb, 64);
        assert_eq!(manifest.limits.timeout_ms, 200);
        assert!(manifest.events.subscribe.is_empty());
    }

    #[test]
    fn subscribes_to_check() {
        let toml = r#"
[plugin]
name = "test"
version = "0.1.0"

[events]
subscribe = ["buffer_open", "tick"]
"#;
        let manifest: PluginManifest = toml::from_str(toml).unwrap();
        assert!(manifest.subscribes_to("buffer_open"));
        assert!(manifest.subscribes_to("tick"));
        assert!(!manifest.subscribes_to("buffer_change"));
    }
}

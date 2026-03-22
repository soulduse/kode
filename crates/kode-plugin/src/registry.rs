use std::collections::HashMap;
use std::path::Path;

use wasmtime::Engine;

use crate::abi::{Decoration, PluginEvent, PluginInfo};
use crate::host::{self, PluginInstance};
use crate::manifest::PluginManifest;

/// Manages all loaded plugins.
pub struct PluginManager {
    engine: Engine,
    plugins: Vec<PluginInstance>,
    decorations: HashMap<String, Vec<Decoration>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let engine = host::create_engine().expect("Failed to create wasmtime engine");
        Self {
            engine,
            plugins: Vec::new(),
            decorations: HashMap::new(),
        }
    }

    /// Discover and load plugins from a directory.
    /// Each subdirectory should contain a plugin.toml.
    pub fn discover(&mut self, plugin_dir: &Path) {
        if !plugin_dir.exists() {
            tracing::debug!("Plugin directory not found: {}", plugin_dir.display());
            return;
        }

        let entries = match std::fs::read_dir(plugin_dir) {
            Ok(entries) => entries,
            Err(e) => {
                tracing::warn!("Failed to read plugin dir: {}", e);
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    self.load_plugin(&manifest_path);
                }
            }
        }

        tracing::info!("Loaded {} plugin(s)", self.plugins.len());
    }

    /// Load a single plugin from its manifest path.
    pub fn load_plugin(&mut self, manifest_path: &Path) {
        match PluginManifest::load(manifest_path) {
            Ok(manifest) => {
                let name = manifest.plugin.name.clone();
                match PluginInstance::load(&self.engine, manifest) {
                    Ok(instance) => {
                        tracing::info!("Loaded plugin: {}", name);
                        self.plugins.push(instance);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load plugin '{}': {}", name, e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to parse manifest {}: {}",
                    manifest_path.display(),
                    e
                );
            }
        }
    }

    /// Dispatch an event to all subscribed plugins and collect decorations.
    pub fn dispatch_event(&mut self, event: &PluginEvent) -> Vec<Decoration> {
        let mut all_decorations = Vec::new();

        for plugin in &mut self.plugins {
            if let Some(response) = plugin.handle_event(event) {
                all_decorations.extend(response.decorations);
            }

            // Drain and log any plugin messages
            for msg in plugin.drain_logs() {
                tracing::debug!("[plugin:{}] {}", plugin.name(), msg);
            }
        }

        // Cache decorations by URI if present
        if let Some(uri) = &event.uri {
            if !all_decorations.is_empty() {
                self.decorations
                    .insert(uri.clone(), all_decorations.clone());
            }
        }

        all_decorations
    }

    /// Get cached decorations for a URI.
    pub fn get_decorations(&self, uri: &str) -> &[Decoration] {
        self.decorations
            .get(uri)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// List all loaded plugins.
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|p| PluginInfo {
                name: p.manifest.plugin.name.clone(),
                version: p.manifest.plugin.version.clone(),
                description: if p.manifest.plugin.description.is_empty() {
                    None
                } else {
                    Some(p.manifest.plugin.description.clone())
                },
            })
            .collect()
    }

    /// Enable a plugin by name.
    pub fn enable_plugin(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.name() == name) {
            plugin.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a plugin by name.
    pub fn disable_plugin(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.name() == name) {
            plugin.enabled = false;
            true
        } else {
            false
        }
    }

    /// Get the number of loaded plugins.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Get the default plugin directory.
    pub fn default_plugin_dir() -> std::path::PathBuf {
        dirs_next::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".kode")
            .join("plugins")
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_plugin_manager() {
        let pm = PluginManager::new();
        assert_eq!(pm.plugin_count(), 0);
        assert!(pm.list_plugins().is_empty());
    }

    #[test]
    fn discover_nonexistent_dir() {
        let mut pm = PluginManager::new();
        pm.discover(Path::new("/nonexistent/path"));
        assert_eq!(pm.plugin_count(), 0);
    }

    #[test]
    fn get_decorations_empty() {
        let pm = PluginManager::new();
        assert!(pm.get_decorations("file:///test.kt").is_empty());
    }
}

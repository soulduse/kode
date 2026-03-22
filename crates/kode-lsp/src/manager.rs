use std::collections::HashMap;
use std::io;

use crate::client::LspClient;
use crate::diagnostics::DiagnosticStore;

/// Configuration for an LSP server.
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub language_id: String,
    pub command: String,
    pub args: Vec<String>,
}

/// Manages multiple LSP server instances, one per language.
pub struct LspManager {
    configs: HashMap<String, LspServerConfig>,
    clients: HashMap<String, LspClient>,
    pub diagnostics: DiagnosticStore,
}

impl LspManager {
    pub fn new() -> Self {
        let mut manager = Self {
            configs: HashMap::new(),
            clients: HashMap::new(),
            diagnostics: DiagnosticStore::new(),
        };

        // Register default Kotlin LSP
        manager.register(LspServerConfig {
            language_id: "kotlin".into(),
            command: "kotlin-language-server".into(),
            args: vec![],
        });

        manager
    }

    /// Register an LSP server configuration for a language.
    pub fn register(&mut self, config: LspServerConfig) {
        self.configs.insert(config.language_id.clone(), config);
    }

    /// Get or start the LSP client for a language.
    pub async fn get_or_start(
        &mut self,
        language_id: &str,
        root_uri: &str,
    ) -> io::Result<&mut LspClient> {
        if self.clients.contains_key(language_id) {
            return Ok(self.clients.get_mut(language_id).unwrap());
        }

        let config = self.configs.get(language_id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("No LSP server configured for '{}'", language_id),
            )
        })?;

        let command = config.command.clone();
        let args: Vec<String> = config.args.clone();
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let mut client = LspClient::start(language_id, &command, &arg_refs).await?;
        client.initialize(root_uri).await?;

        tracing::info!("LSP server started for '{}'", language_id);
        self.clients.insert(language_id.to_string(), client);
        Ok(self.clients.get_mut(language_id).unwrap())
    }

    /// Notify the appropriate server that a document was opened.
    pub async fn notify_open(
        &mut self,
        language_id: &str,
        uri: &str,
        version: i32,
        text: &str,
        root_uri: &str,
    ) -> io::Result<()> {
        let client = self.get_or_start(language_id, root_uri).await?;
        client.did_open(uri, language_id, version, text).await
    }

    /// Notify the appropriate server of document changes.
    pub async fn notify_change(
        &mut self,
        language_id: &str,
        uri: &str,
        version: i32,
        text: &str,
        root_uri: &str,
    ) -> io::Result<()> {
        let client = self.get_or_start(language_id, root_uri).await?;
        client.did_change(uri, version, text).await
    }

    /// Notify the appropriate server that a document was saved.
    pub async fn notify_save(
        &mut self,
        language_id: &str,
        uri: &str,
        root_uri: &str,
    ) -> io::Result<()> {
        let client = self.get_or_start(language_id, root_uri).await?;
        client.did_save(uri).await
    }

    /// Notify the appropriate server that a document was closed.
    pub async fn notify_close(
        &mut self,
        language_id: &str,
        uri: &str,
        root_uri: &str,
    ) -> io::Result<()> {
        let client = self.get_or_start(language_id, root_uri).await?;
        client.did_close(uri).await
    }

    /// Get a mutable reference to a running client.
    pub fn client_mut(&mut self, language_id: &str) -> Option<&mut LspClient> {
        self.clients.get_mut(language_id)
    }

    /// Check if a language has a configured server.
    pub fn has_config(&self, language_id: &str) -> bool {
        self.configs.contains_key(language_id)
    }

    /// Check if a server is running for a language.
    pub fn is_running(&self, language_id: &str) -> bool {
        self.clients.contains_key(language_id)
    }

    /// Shut down all running servers.
    pub async fn shutdown_all(&mut self) {
        for (lang, client) in self.clients.iter_mut() {
            if let Err(e) = client.shutdown().await {
                tracing::warn!("Failed to shut down LSP for '{}': {}", lang, e);
            }
        }
        self.clients.clear();
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_kotlin() {
        let manager = LspManager::new();
        assert!(manager.has_config("kotlin"));
        assert!(!manager.has_config("java"));
    }

    #[test]
    fn register_custom_server() {
        let mut manager = LspManager::new();
        manager.register(LspServerConfig {
            language_id: "rust".into(),
            command: "rust-analyzer".into(),
            args: vec![],
        });
        assert!(manager.has_config("rust"));
    }
}

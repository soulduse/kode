use std::collections::HashMap;

use lsp_types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams};
use serde_json::Value;

/// Stores diagnostics per document URI.
#[derive(Debug, Default)]
pub struct DiagnosticStore {
    store: HashMap<String, Vec<Diagnostic>>,
}

impl DiagnosticStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a publishDiagnostics notification from the server.
    pub fn handle_publish(&mut self, params_value: Value) -> Option<String> {
        let params: PublishDiagnosticsParams = serde_json::from_value(params_value).ok()?;
        let uri = params.uri.to_string();
        self.store.insert(uri.clone(), params.diagnostics);
        Some(uri)
    }

    /// Get diagnostics for a URI.
    pub fn get(&self, uri: &str) -> &[Diagnostic] {
        self.store.get(uri).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get error-level diagnostics count for a URI.
    pub fn error_count(&self, uri: &str) -> usize {
        self.get(uri)
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::ERROR))
            .count()
    }

    /// Get warning-level diagnostics count for a URI.
    pub fn warning_count(&self, uri: &str) -> usize {
        self.get(uri)
            .iter()
            .filter(|d| d.severity == Some(DiagnosticSeverity::WARNING))
            .count()
    }

    /// Clear diagnostics for a URI.
    pub fn clear(&mut self, uri: &str) {
        self.store.remove(uri);
    }

    /// All URIs with diagnostics.
    pub fn uris(&self) -> impl Iterator<Item = &str> {
        self.store.keys().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_retrieve() {
        let mut store = DiagnosticStore::new();
        let params = serde_json::json!({
            "uri": "file:///test.kt",
            "diagnostics": [
                {
                    "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 5}},
                    "severity": 1,
                    "message": "error here"
                },
                {
                    "range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 3}},
                    "severity": 2,
                    "message": "warning here"
                }
            ]
        });

        let uri = store.handle_publish(params).unwrap();
        assert_eq!(uri, "file:///test.kt");
        assert_eq!(store.get(&uri).len(), 2);
        assert_eq!(store.error_count(&uri), 1);
        assert_eq!(store.warning_count(&uri), 1);
    }

    #[test]
    fn clear_diagnostics() {
        let mut store = DiagnosticStore::new();
        let params = serde_json::json!({
            "uri": "file:///test.kt",
            "diagnostics": [
                {
                    "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
                    "message": "err"
                }
            ]
        });
        store.handle_publish(params);
        store.clear("file:///test.kt");
        assert!(store.get("file:///test.kt").is_empty());
    }
}

use std::collections::HashMap;

use tree_sitter::{Parser, Tree};

use crate::languages::LanguageConfig;

/// Manages tree-sitter parsers and parse trees per document.
pub struct ParserManager {
    parsers: HashMap<String, Parser>,
    trees: HashMap<usize, Tree>,
    configs: HashMap<String, LanguageConfig>,
}

impl ParserManager {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
            trees: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Register a language configuration.
    pub fn register_language(&mut self, lang_id: &str, config: LanguageConfig) {
        let mut parser = Parser::new();
        parser
            .set_language(&config.language)
            .expect("Failed to set parser language");
        self.parsers.insert(lang_id.to_string(), parser);
        self.configs.insert(lang_id.to_string(), config);
    }

    /// Parse a document's full text. Returns the tree.
    pub fn parse(&mut self, doc_id: usize, lang_id: &str, source: &[u8]) -> Option<&Tree> {
        let parser = self.parsers.get_mut(lang_id)?;
        let old_tree = self.trees.get(&doc_id);
        let tree = parser.parse(source, old_tree)?;
        self.trees.insert(doc_id, tree);
        self.trees.get(&doc_id)
    }

    /// Get the cached tree for a document.
    pub fn tree(&self, doc_id: usize) -> Option<&Tree> {
        self.trees.get(&doc_id)
    }

    /// Get language config.
    pub fn language_config(&self, lang_id: &str) -> Option<&LanguageConfig> {
        self.configs.get(lang_id)
    }

    /// Remove a document's parse tree (e.g., on close).
    pub fn remove_tree(&mut self, doc_id: usize) {
        self.trees.remove(&doc_id);
    }
}

impl Default for ParserManager {
    fn default() -> Self {
        Self::new()
    }
}

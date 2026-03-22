use tree_sitter::Language;

/// Configuration for a supported language.
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub language: Language,
    pub highlight_query: String,
    pub file_extensions: Vec<String>,
    pub comment_prefix: String,
}

/// Get list of all built-in language IDs.
pub fn builtin_language_ids() -> Vec<&'static str> {
    // Languages will be registered as tree-sitter grammars are added.
    // For now, this is a placeholder.
    vec![]
}

use lsp_types::ServerCapabilities;

/// Helper to query server capabilities.
pub struct ServerCaps {
    caps: ServerCapabilities,
}

impl ServerCaps {
    pub fn new(caps: ServerCapabilities) -> Self {
        Self { caps }
    }

    pub fn supports_completion(&self) -> bool {
        self.caps.completion_provider.is_some()
    }

    pub fn supports_hover(&self) -> bool {
        self.caps.hover_provider.is_some()
    }

    pub fn supports_goto_definition(&self) -> bool {
        self.caps.definition_provider.is_some()
    }

    pub fn supports_goto_implementation(&self) -> bool {
        self.caps.implementation_provider.is_some()
    }

    pub fn supports_references(&self) -> bool {
        self.caps.references_provider.is_some()
    }

    pub fn supports_code_action(&self) -> bool {
        self.caps.code_action_provider.is_some()
    }

    pub fn supports_document_symbols(&self) -> bool {
        self.caps.document_symbol_provider.is_some()
    }

    pub fn supports_workspace_symbols(&self) -> bool {
        self.caps.workspace_symbol_provider.is_some()
    }

    pub fn inner(&self) -> &ServerCapabilities {
        &self.caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_capabilities() {
        let caps = ServerCaps::new(ServerCapabilities::default());
        assert!(!caps.supports_completion());
        assert!(!caps.supports_hover());
        assert!(!caps.supports_goto_definition());
    }
}

/// LSP client implementation (Phase 3).
///
/// Will provide:
/// - JSON-RPC transport over stdio
/// - LSP client with capability negotiation
/// - Completion, diagnostics, hover, go-to-definition
pub struct LspClient {
    _placeholder: (),
}

impl LspClient {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for LspClient {
    fn default() -> Self {
        Self::new()
    }
}

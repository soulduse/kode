/// WASM plugin host (Phase 5).
///
/// Will provide:
/// - wasmtime-based plugin execution
/// - WIT interface for plugin API
/// - Plugin manifest and registry
pub struct PluginHost {
    _placeholder: (),
}

impl PluginHost {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}

/// Window creation and management using winit.
///
/// Handles:
/// - Window creation with proper DPI scaling
/// - Event loop integration
/// - Input event translation to KodeEvent
pub struct KodeWindow {
    _placeholder: (),
}

impl KodeWindow {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for KodeWindow {
    fn default() -> Self {
        Self::new()
    }
}

use kode_core::event::{KodeEvent, KeyCode, KeyEvent, Modifiers, MouseButton, MouseEvent};

/// Convert winit keyboard input to KodeEvent.
///
/// This will be expanded to handle the full winit keyboard API
/// once the GPU window is implemented.
pub fn translate_key(
    _key: &str,
    _modifiers: u8,
) -> Option<KodeEvent> {
    // Placeholder for winit key translation
    None
}

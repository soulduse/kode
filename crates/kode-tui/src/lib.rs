pub mod chrome;
pub mod colors;
pub mod editor_view;
pub mod event;
pub mod event_loop;
pub mod explorer_view;
pub mod terminal_view;
pub mod ui;

// Re-export from kode-state for backwards compatibility
pub use kode_state::file_explorer;
pub use kode_state::{AppState, AppView, FileExplorer, create_app_state, create_app_view};

/// Compatibility alias: app_bridge re-exports from kode_state.
pub mod app_bridge {
    pub use kode_state::{AppState, create_app_state};
}

/// Wrapper around AppState for TUI.
pub struct TuiApp {
    pub inner: AppState,
}

impl TuiApp {
    pub fn new(inner: AppState) -> Self {
        Self { inner }
    }

    /// Create an AppView snapshot for rendering.
    pub fn view(&self) -> AppView<'_> {
        create_app_view(&self.inner)
    }
}

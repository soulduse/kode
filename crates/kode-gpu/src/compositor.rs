use kode_core::geometry::Rect;
use kode_renderer::RenderCommand;

/// Composites multiple pane render outputs into the final frame.
///
/// Each pane renders independently into its viewport rect.
/// The compositor combines all pane outputs and renders borders,
/// status bar, and tab bar.
pub struct Compositor {
    viewports: Vec<Viewport>,
}

pub struct Viewport {
    pub pane_id: usize,
    pub rect: Rect,
    pub commands: Vec<RenderCommand>,
}

impl Compositor {
    pub fn new() -> Self {
        Self {
            viewports: Vec::new(),
        }
    }

    pub fn set_viewports(&mut self, viewports: Vec<Viewport>) {
        self.viewports = viewports;
    }

    pub fn all_commands(&self) -> impl Iterator<Item = &RenderCommand> {
        self.viewports.iter().flat_map(|v| v.commands.iter())
    }

    pub fn clear(&mut self) {
        self.viewports.clear();
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}

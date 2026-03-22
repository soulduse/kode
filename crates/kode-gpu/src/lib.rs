pub mod compositor;
pub mod gpu_app;
pub mod input;
pub mod project;
pub mod rect_pipeline;
pub mod surface;
pub mod text_render;
pub mod welcome_screen;
pub mod window;

use gpu_app::AppScreen;
use kode_core::error::KodeResult;
use winit::event_loop::{ControlFlow, EventLoop};

/// Run the application in GPU mode with a native window.
pub fn run_gpu(screen: AppScreen) -> KodeResult<()> {
    let event_loop = EventLoop::new()
        .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = gpu_app::GpuApp::new(screen);
    event_loop
        .run_app(&mut app)
        .map_err(|e| kode_core::error::KodeError::Other(e.to_string()))?;

    Ok(())
}

//! Viewer for nelo scenes

mod app;
mod ui;

pub use ui::UiRenderer;

use crate::render::Playback;
use app::App;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

// ----- Viewer -----

pub enum Viewer {
    Pending(Playback),
    Starting,
    Running { window: Arc<Window>, state: App },
}

impl Viewer {
    pub fn new(playback: impl Into<Playback>) -> Self {
        Self::Pending(playback.into())
    }
}

impl ApplicationHandler for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Pending(playback) = std::mem::replace(self, Self::Starting) {
            // First create the window.
            let (width, height) = (1280, 720);
            let attrs = Window::default_attributes()
                .with_title("Nelo Viewer")
                .with_inner_size(LogicalSize { width, height });

            let window = Arc::new(
                event_loop
                    .create_window(attrs)
                    .expect("Failed to create window"),
            );

            // Then create the state.
            let state = pollster::block_on(App::new(window.clone(), playback));

            // Update to running state.
            *self = Self::Running { window, state };
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Self::Running { window: _, state } = self else {
            return;
        };

        state.event(event_loop, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Self::Running { window, state: _ } = self else {
            return;
        };

        window.request_redraw();
    }
}

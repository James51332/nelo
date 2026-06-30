use nelo::context::GPU;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    context: Option<GPU>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Don't create the window more than once.
        if self.window.is_some() {
            return;
        }

        // Create the window and context now.
        let mut attrs = Window::default_attributes();
        attrs.title = "Nelo".into();

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );
        let size = window.inner_size();

        self.window = Some(window.clone());
        self.context = Some(pollster::block_on(GPU::new(
            window.clone(),
            size.width,
            size.height,
        )));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // Wait until we have a context before responding to window events.
        let context = match &mut self.context {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => context.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                context.render();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop!");
    let mut app = App::default();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("Unexpected event loop failure!");
}

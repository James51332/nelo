use std::sync::Arc;
use winit::window::Window;

pub struct Context {
    window: Arc<Window>,
}

impl Context {
    pub async fn new(window: Arc<Window>) -> Self {
        Self { window }
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}

    pub fn render(&mut self) {
        self.window.request_redraw();
    }
}

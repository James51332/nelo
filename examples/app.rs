use nelo::prelude::*;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new().expect("Failed to create event loop!");
    let mut app = Viewer::new(Story::demo());

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("Unexpected event loop failure!");
}

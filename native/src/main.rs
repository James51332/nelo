use nelo::render::{Camera, CircleRenderer, Gpu, SceneRenderer, Target, WindowTarget};
use nelo::scene::Scene;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const SCENE_HEIGHT: f32 = 10.0;

struct RenderState {
    gpu: Gpu,
    target: WindowTarget,
    renderer: SceneRenderer,
}

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    start: Option<Instant>,
}

impl App {
    fn draw(&mut self) {
        let t = self.start.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
        let Some(r) = &mut self.render_state else {
            return;
        };
        let Some(frame) = r.target.acquire(&r.gpu) else {
            return;
        };

        r.renderer.render(&r.gpu, &frame.view, t);
        r.target.present(&r.gpu, frame);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut attrs = Window::default_attributes();
        attrs.title = "Nelo".into();
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );
        let size = window.inner_size();

        let (gpu, surface) = pollster::block_on(Gpu::with_surface(window.clone()));
        let target = WindowTarget::new(&gpu, surface, size.width, size.height);
        let camera = Camera::new(&gpu, SCENE_HEIGHT);

        let circles = CircleRenderer::new(&gpu, camera.layout(), target.format());
        let mut renderer = SceneRenderer::new(camera, Scene::demo());
        renderer.add(circles);

        self.window = Some(window);
        self.render_state = Some(RenderState {
            gpu,
            renderer,
            target,
        });
        self.start = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(r) = &mut self.render_state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => r.target.resize(&r.gpu, size.width, size.height),
            WindowEvent::RedrawRequested => self.draw(),
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

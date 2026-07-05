use glam::prelude::*;
use nelo::context::Gpu;
use nelo::render::{Camera, Circle, CircleRenderer, FrameCtx, Renderer, Target, WindowTarget};
use nelo::timeline::Timeline;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

/// Vertical extent of the visible world, in world units.
const SCENE_HEIGHT: f32 = 10.0;

struct Renderers {
    gpu: Gpu,
    target: WindowTarget,
    camera: Camera,
    circles: CircleRenderer,
}

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderers: Option<Renderers>,
    start: Option<Instant>,
}

impl App {
    fn draw(&mut self) {
        let Some(r) = &mut self.renderers else { return };
        let t = self.start.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);

        let size = r.target.size();
        let items = demo_scene();

        // 1. Sample scene → uploads (before the pass).
        let ctx = FrameCtx {
            gpu: &r.gpu,
            time: t,
            size,
        };
        r.circles.prepare(&ctx, &items);
        r.camera.upload(&r.gpu, t, size);

        // 2. One encoder, one pass. The camera is bound at group 0.
        let Some(frame) = r.target.acquire(&r.gpu) else {
            return;
        };
        let mut encoder = r
            .gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nelo frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nelo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.04,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, r.camera.bind_group(), &[]);
            r.circles.draw(&mut pass);
        }

        // 3. Submit and present.
        r.gpu.queue().submit(std::iter::once(encoder.finish()));
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

        self.window = Some(window);
        self.renderers = Some(Renderers {
            gpu,
            target,
            camera,
            circles,
        });
        self.start = Some(Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(r) = &mut self.renderers else { return };

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

/// A small animated scene: a ring of orbiting circles around a pulsing center.
fn demo_scene() -> Vec<Circle> {
    use std::f32::consts::TAU;

    let mut circles = Vec::new();

    // Central pulsing circle.
    circles.push(Circle {
        center: Timeline::constant(Vec2::new(0.0, 0.0)),
        radius: Timeline::dynamic(|t: f32| 1.0 + 0.2 * (t as f32 * 2.0).sin()),
        color: Timeline::constant(Vec4::new(0.9, 0.9, 1.0, 1.0)),
    });

    // Orbiting ring.
    const N: usize = 12;
    for i in 0..N {
        circles.push(Circle {
            center: Timeline::dynamic(move |t: f32| {
                let phase = i as f32 / N as f32 * TAU;
                let angle = phase + t * 0.6;
                let x = 3.5 * angle.cos();
                let y = 3.5 * angle.sin();
                Vec2::new(x as f32, y as f32)
            }),
            radius: Timeline::constant(0.5),
            color: Timeline::dynamic(move |_: f32| {
                let hue = i as f32 / N as f32;
                Vec4::new(0.5 + 0.5 * hue, 0.6, 1.0 - 0.5 * hue, 1.0)
            }),
        });
    }

    circles
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop!");
    let mut app = App::default();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop
        .run_app(&mut app)
        .expect("Unexpected event loop failure!");
}

//! Render a single frame headlessly (no window) and save it as a PNG.
//!
//! Demonstrates the export path: a `TextureTarget` backed by an offscreen
//! texture, driven by exactly the same renderer code as the windowed app, then
//! read back to the CPU. Run with:
//!
//! ```sh
//! cargo run --example headless
//! ```

use nelo::context::Gpu;
use nelo::render::{Camera, Circle, CircleRenderer, FrameCtx, Renderer, Target, TextureTarget};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SCENE_HEIGHT: f32 = 10.0;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let gpu = Gpu::headless().await;
    let mut target = TextureTarget::new(&gpu, WIDTH, HEIGHT);
    let camera = Camera::new(&gpu, SCENE_HEIGHT);
    let mut circles = CircleRenderer::new(&gpu, camera.layout(), target.format());

    let t = 1.0;
    let size = target.size();
    let scene = demo_scene(t);

    // Prepare + upload before the pass.
    let ctx = FrameCtx {
        gpu: &gpu,
        time: t,
        size,
    };
    circles.prepare(&ctx, &scene);
    camera.upload(&gpu, t, size);

    // One pass, camera at group 0 — identical to the windowed driver.
    let frame = target.acquire(&gpu).expect("offscreen frame");
    let mut encoder = gpu
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nelo headless frame"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("nelo headless pass"),
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
        pass.set_bind_group(0, camera.bind_group(), &[]);
        circles.draw(&mut pass);
    }
    gpu.queue().submit(std::iter::once(encoder.finish()));
    target.present(&gpu, frame);

    // Read back and write PNG.
    let pixels = target.read(&gpu);
    let file = std::fs::File::create("headless.png").expect("create headless.png");
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, WIDTH, HEIGHT);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&pixels)
        .unwrap();

    println!("wrote headless.png ({WIDTH}x{HEIGHT})");
}

fn demo_scene(t: f64) -> Vec<Circle> {
    use std::f64::consts::TAU;

    let mut circles = vec![Circle {
        center: [0.0, 0.0],
        radius: 1.0 + 0.2 * (t * 2.0).sin() as f32,
        color: [0.9, 0.9, 1.0, 1.0],
    }];

    let n = 12;
    for i in 0..n {
        let angle = i as f64 / n as f64 * TAU + t * 0.6;
        let hue = i as f32 / n as f32;
        circles.push(Circle {
            center: [3.5 * angle.cos() as f32, 3.5 * angle.sin() as f32],
            radius: 0.5,
            color: [0.5 + 0.5 * hue, 0.6, 1.0 - 0.5 * hue, 1.0],
        });
    }

    circles
}

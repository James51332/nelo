//! Renderers the demo scene at t=1.0 sec to a PNG file.

use nelo::render::{Gpu, SceneRenderer, Target, TextureTarget};
use nelo::scene::Scene;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SCENE_TIME: f32 = 1.0;

fn main() {
    let gpu = Gpu::headless_blocking();

    // Setup a target and renderer
    let mut target = TextureTarget::new(&gpu, WIDTH, HEIGHT);
    let mut renderer = SceneRenderer::new(&gpu, target.format(), Scene::demo());

    // Run the draw loop exactly once.
    let frame = target.acquire(&gpu).expect("offscreen frame");
    renderer.render(&gpu, &frame.view, SCENE_TIME);

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

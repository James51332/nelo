//! Tool for exporting static frames using nelo.

use crate::export::Export;
use crate::render::{Gpu, Renderer, Target, TextureTarget};
use crate::scene::Scene;
use png::{BitDepth, ColorType, Encoder};
use std::fs::File;
use std::io::BufWriter;

pub struct ImageExport {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub file_name: &'static str,
    pub file_ext: &'static str,
}

impl Default for ImageExport {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            time: 0.0,
            file_name: "nelo_scene",
            file_ext: "png",
        }
    }
}

impl Export for ImageExport {
    fn export(&self, scene: Scene) -> Result<(), String> {
        // Setup a target and renderer.
        let gpu = Gpu::headless();
        let mut target = TextureTarget::new(&gpu, self.width, self.height);
        let mut renderer = Renderer::new(&gpu, scene);

        // Run the draw loop exactly once.
        let frame = target.acquire(&gpu).ok_or("Failed to acquire target")?;
        renderer.render(&gpu, &frame.view, self.time);

        // Read back and write PNG.
        let pixels = target.read(&gpu);
        let path = format!("{}.{}", self.file_name, self.file_ext);
        let file = File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);

        let mut encoder = Encoder::new(writer, self.width, self.height);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        encoder
            .write_header()
            .map_err(|e| e.to_string())?
            .write_image_data(&pixels)
            .map_err(|e| e.to_string())
    }
}

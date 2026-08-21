//! Tool for exporting static frames using nelo.

use crate::export::{Export, ExportTexture, gpu};
use crate::render::Renderer;
use crate::scene::Playback;
use png::{BitDepth, ColorType, Encoder};
use std::fs::File;
use std::io::BufWriter;
use wgpu::{Device, Queue, TextureFormat, TextureViewDescriptor};

const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub struct ImageExport {
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub file_name: &'static str,
    pub file_ext: &'static str,
    pub gpu: Option<(Device, Queue)>,
}

impl Default for ImageExport {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            time: 0.0,
            file_name: "nelo_scene",
            file_ext: "png",
            gpu: None,
        }
    }
}

impl Export for ImageExport {
    fn export(&self, scene: impl Into<Playback>) -> Result<(), String> {
        let (device, queue) = match self.gpu.as_ref() {
            Some(pair) => pair.clone(),
            None => gpu::create(),
        };

        // Setup the render pipeline.
        let target = ExportTexture::new(&device, FORMAT, self.width, self.height);
        let mut renderer = Renderer::new(device.clone(), queue.clone(), FORMAT, scene);

        // Run the draw loop exactly once.
        let desc = TextureViewDescriptor::default();
        let view = target.texture.create_view(&desc);
        renderer.render(&view, self.time);

        // Read back and write PNG.
        let pixels = match target.read(&device, &queue) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(e.to_string());
            }
        };
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

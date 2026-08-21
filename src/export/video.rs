//! Tool for exporting videos using nelo.

use wgpu::wgt::TextureViewDescriptor;
use wgpu::{Device, Queue, TextureFormat};

use crate::export::{Export, ExportTexture, gpu};
use crate::render::Renderer;
use crate::scene::Playback;
use std::io::Write;
use std::process::{Command, Stdio};

const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// Defines values for exporting via ffmpeg.
pub struct VideoExport {
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    pub file_name: &'static str,
    pub file_ext: &'static str,

    pub gpu: Option<(Device, Queue)>,

    // TODO Would love for these to be optional in the future and
    // derived from the scene.
    pub start_time: f32,
    pub end_time: f32,
}

impl Default for VideoExport {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            frame_rate: 30,
            file_name: "nelo_scene",
            file_ext: "mp4",
            gpu: None,
            start_time: 0.0,
            end_time: 10.0,
        }
    }
}

impl Export for VideoExport {
    fn export(&self, scene: impl Into<Playback>) -> Result<(), String> {
        let (device, queue) = match self.gpu.as_ref() {
            Some(pair) => pair.clone(),
            None => gpu::create(),
        };

        // Setup the render pipeline.
        let target = ExportTexture::new(&device, FORMAT, self.width, self.height);
        let mut renderer = Renderer::new(device.clone(), queue.clone(), FORMAT, scene);

        // Verify the time.
        if self.frame_rate == 0 {
            return Err("Cannot export scene with frame rate 0".into());
        }

        if self.end_time < self.start_time {
            return Err("Cannot export scene with negative length".into());
        }
        let frames = ((self.end_time - self.start_time) * self.frame_rate as f32).ceil() as u32;

        // Setup our ffmpeg process.
        let Ok(mut process) = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgba",
                "-video_size",
                &format!("{}x{}", self.width, self.height),
                "-framerate",
                &self.frame_rate.to_string(),
                "-i",
                "-",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                &format!("{}.{}", self.file_name, self.file_ext),
            ])
            .stdin(Stdio::piped())
            .spawn()
        else {
            return Err("Failed to spawn ffmpeg process".into());
        };

        if let Some(mut stdin) = process.stdin.take() {
            // Run the render loop. We should parallelize in the future.
            let timestep = 1.0 / self.frame_rate as f32;
            for i in 0..frames {
                let view = target
                    .texture
                    .create_view(&TextureViewDescriptor::default());
                let time = self.start_time + i as f32 * timestep;

                // Render to the texture.
                renderer.render(&view, time);

                // Read back.
                let data = match target.read(&device, &queue) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return Err(e.to_string());
                    }
                };

                // Write out.
                if let Err(x) = stdin.write_all(&data) {
                    return Err(x.to_string());
                }
            }
        } else {
            return Err("Failed to pipe data to ffmpeg".into());
        }

        // Wait and return.
        match process.wait() {
            Ok(_) => log::info!("FFmpeg process finished"),
            Err(x) => return Err(x.to_string()),
        };

        Ok(())
    }
}

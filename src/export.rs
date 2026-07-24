//! Module for exporting via ffmpeg subprocess.
use crate::render::{Gpu, SceneRenderer, Target, TextureTarget};
use crate::scene::Scene;
use std::io::Write;
use std::process::{Command, Stdio};

/// Defines values for exporting via ffmpeg.
pub struct ExportConfig {
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    pub file_name: &'static str,
    pub file_ext: &'static str,

    // TODO Would love for these to be optional in the future and
    // derived from the scene.
    pub start_time: f32,
    pub end_time: f32,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            frame_rate: 30,
            file_name: "nelo_scene",
            file_ext: "mp4",
            start_time: 0.0,
            end_time: 10.0,
        }
    }
}

/// Exports a scene using the given config
pub fn export(scene: Scene, config: &ExportConfig) -> Result<(), String> {
    // Setup the render pipeline.
    let gpu = Gpu::headless_blocking();
    let mut target = TextureTarget::new(&gpu, config.width, config.height);
    let mut renderer = SceneRenderer::new(&gpu, target.format(), scene);

    // Verify the time.
    if config.frame_rate == 0 {
        return Err("Cannot export scene with frame rate 0".into());
    }

    let frames = ((config.end_time - config.start_time) * config.frame_rate as f32).ceil() as u32;
    if frames <= 0 {
        return Err("Cannot export scene with negative length".into());
    }

    // Setup our ffmpeg process.
    let Ok(mut process) = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgba",
            "-video_size",
            &format!("{}x{}", config.width, config.height),
            "-framerate",
            &config.frame_rate.to_string(),
            "-i",
            "-",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            &format!("{}.{}", config.file_name, config.file_ext),
        ])
        .stdin(Stdio::piped())
        .spawn()
    else {
        return Err("Failed to spawn ffmpeg process".into());
    };

    if let Some(mut stdin) = process.stdin.take() {
        // Run the render loop. We should parallelize in the future.
        let timestep = 1.0 / config.frame_rate as f32;
        for i in 0..frames {
            let time = config.start_time + i as f32 * timestep;

            if let Some(frame) = target.acquire(&gpu) {
                renderer.render(&gpu, &frame.view, time);
                if let Err(x) = stdin.write_all(&target.read(&gpu)) {
                    return Err(x.to_string());
                }
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

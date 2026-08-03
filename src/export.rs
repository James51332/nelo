//! Module for exporting via ffmpeg subprocess.

pub mod image;
pub mod video;

pub use image::ImageExport;
pub use video::VideoExport;

use crate::scene::Scene;

pub trait Export {
    fn export(&self, scene: Scene) -> Result<(), String>;
}

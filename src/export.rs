//! Module for exporting via ffmpeg subprocess.

pub mod image;
pub mod video;

mod gpu;
mod texture;

pub use image::ImageExport;
pub use texture::ExportTexture;
pub use video::VideoExport;

use crate::render::Playback;

pub trait Export {
    fn export(&self, playback: impl Into<Playback>) -> Result<(), String>;
}

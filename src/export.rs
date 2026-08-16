//! Module for exporting via ffmpeg subprocess.

pub mod image;
pub mod video;

pub use image::ImageExport;
pub use video::VideoExport;

use crate::render::Playback;

pub trait Export {
    fn export(&self, playback: impl Into<Playback>) -> Result<(), String>;
}

//! Render targets are where a frame goes.
//!
//! We support two types natively. [`TextureTarget`] and [`WindowTarget`].

use crate::render::Gpu;
use std::sync::mpsc;
use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, CompositeAlphaMode,
    CurrentSurfaceTexture, Extent3d, MapMode, Origin3d, PollType, PresentMode, Surface,
    SurfaceConfiguration, SurfaceTexture, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureUsages, TextureView, TextureViewDescriptor,
};

// ----- Target -----

/// A single frame acquired from a [`Target`]: the view to render into, plus any
/// resources that must live until the frame is presented.
pub struct Frame {
    pub view: TextureView,

    /// Present for a swapchain frame; `None` for an offscreen texture.
    surface_texture: Option<SurfaceTexture>,
}

pub trait Target {
    /// Acquire the next frame, or `None` to skip (e.g. surface not ready).
    fn acquire(&mut self, gpu: &Gpu) -> Option<Frame>;
}

// ----- WindowTarget -----

/// Renders to a window swapchain.
pub struct WindowTarget {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl WindowTarget {
    pub fn new(gpu: &Gpu, surface: Surface<'static>, width: u32, height: u32) -> Self {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: gpu.format(),
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::default(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(gpu.device(), &config);
        Self { surface, config }
    }

    pub fn resize(&mut self, gpu: &Gpu, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(gpu.device(), &self.config);
        }
    }

    pub fn present(&mut self, _gpu: &Gpu, frame: Frame) {
        if let Some(texture) = frame.surface_texture {
            texture.present();
        }
    }
}

impl Target for WindowTarget {
    fn acquire(&mut self, gpu: &Gpu) -> Option<Frame> {
        let texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) => t,
            CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(gpu.device(), &self.config);
                t
            }
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(gpu.device(), &self.config);
                return None;
            }
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => return None,
            CurrentSurfaceTexture::Lost => {
                self.surface.configure(gpu.device(), &self.config);
                return None;
            }
        };

        let view_desc = TextureViewDescriptor::default();
        let view = texture.texture.create_view(&view_desc);
        Some(Frame {
            view,
            surface_texture: Some(texture),
        })
    }
}

// ----- Target -----

/// Renders to an offscreen texture and reads it back to the CPU (for export).
pub struct TextureTarget {
    texture: Texture,
    readback: Buffer,
    width: u32,
    height: u32,

    /// Bytes per row rounded up to `COPY_BYTES_PER_ROW_ALIGNMENT`.
    padded_bytes_per_row: u32,
}

impl TextureTarget {
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> Self {
        // Create our texture.
        let texture_desc = TextureDescriptor {
            label: Some("nelo offscreen target"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: gpu.format(),
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = gpu.device().create_texture(&texture_desc);

        // Compute the aligned size.
        let unpadded = width * 4;
        let align = COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded.div_ceil(align) * align;

        // Create our readbck buffer.
        let buffer_desc = BufferDescriptor {
            label: Some("nelo readback"),
            size: (padded_bytes_per_row * height) as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        };
        let readback = gpu.device().create_buffer(&buffer_desc);

        Self {
            texture,
            readback,
            width,
            height,
            padded_bytes_per_row,
        }
    }

    /// Copy the rendered texture to the CPU and return tightly-packed RGBA8.
    pub fn read(&self, gpu: &Gpu) -> Vec<u8> {
        // Encode the command to read back into the buffer.
        let mut encoder = gpu.device().create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue().submit(std::iter::once(encoder.finish()));

        // Read back into the buffer.
        let slice = self.readback.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(MapMode::Read, move |r| {
            let _ = tx.send(r);
        });

        // We are currently waiting until everything on the GPU is done.
        gpu.device().poll(PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();

        // Account for the padding.
        let padded = slice.get_mapped_range();
        let unpadded = (self.width * 4) as usize;
        let mut out = Vec::with_capacity(unpadded * self.height as usize);
        for row in 0..self.height as usize {
            let start = row * self.padded_bytes_per_row as usize;
            out.extend_from_slice(&padded[start..start + unpadded]);
        }

        // Unmap the memory.
        drop(padded);
        self.readback.unmap();

        // Return the read memory.
        out
    }
}

impl Target for TextureTarget {
    fn acquire(&mut self, _gpu: &Gpu) -> Option<Frame> {
        let view = self.texture.create_view(&TextureViewDescriptor::default());
        Some(Frame {
            view,
            surface_texture: None,
        })
    }
}

//! Render targets are where a frame goes.
//!
//! A [`WindowTarget`] presents to a swapchain; a [`TextureTarget`] renders to
//! an offscreen texture that can be read back to the CPU for export. Both hand
//! the driver a [`Frame`] (a view to render into) and both are driven by the
//! same rendering code.

use crate::render::Gpu;

/// A single frame acquired from a [`Target`]: the view to render into, plus any
/// resources that must live until the frame is presented.
pub struct Frame {
    pub view: wgpu::TextureView,
    /// Present for a swapchain frame; `None` for an offscreen texture.
    surface_texture: Option<wgpu::SurfaceTexture>,
}

/// Somewhere a frame can be drawn.
pub trait Target {
    fn format(&self) -> wgpu::TextureFormat;
    fn size(&self) -> (u32, u32);
    /// Acquire the next frame, or `None` to skip (e.g. surface not ready).
    fn acquire(&mut self, gpu: &Gpu) -> Option<Frame>;
    /// Finish the frame (present a swapchain image; no-op for a texture).
    fn present(&mut self, gpu: &Gpu, frame: Frame);
}

/// Renders to a window swapchain.
pub struct WindowTarget {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl WindowTarget {
    pub fn new(gpu: &Gpu, surface: wgpu::Surface<'static>, width: u32, height: u32) -> Self {
        let caps = surface.get_capabilities(gpu.adapter());
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        // Fifo (vsync) is guaranteed supported and caps the frame rate to the
        // display refresh. Without it we can silently pick Immediate/Mailbox,
        // and — paired with an uncapped Poll redraw loop — the CPU outruns the
        // GPU, so per-frame staging and command-buffer allocations pile up
        // faster than they're reclaimed until memory is exhausted.
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
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
}

impl Target for WindowTarget {
    fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    fn acquire(&mut self, gpu: &Gpu) -> Option<Frame> {
        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(gpu.device(), &self.config);
                t
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(gpu.device(), &self.config);
                return None;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return None,
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(gpu.device(), &self.config);
                return None;
            }
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Some(Frame {
            view,
            surface_texture: Some(texture),
        })
    }

    fn present(&mut self, _gpu: &Gpu, frame: Frame) {
        if let Some(texture) = frame.surface_texture {
            texture.present();
        }
    }
}

/// Renders to an offscreen texture and reads it back to the CPU (for export).
pub struct TextureTarget {
    texture: wgpu::Texture,
    readback: wgpu::Buffer,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    /// Bytes per row rounded up to `COPY_BYTES_PER_ROW_ALIGNMENT`.
    padded_bytes_per_row: u32,
}

impl TextureTarget {
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> Self {
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = gpu.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("nelo offscreen target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let unpadded = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded.div_ceil(align) * align;

        let readback = gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("nelo readback"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            texture,
            readback,
            format,
            width,
            height,
            padded_bytes_per_row,
        }
    }

    /// Copy the rendered texture to the CPU and return tightly-packed RGBA8.
    pub fn read(&self, gpu: &Gpu) -> Vec<u8> {
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nelo readback copy"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue().submit(std::iter::once(encoder.finish()));

        let slice = self.readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        gpu.device()
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        rx.recv().unwrap().unwrap();

        let padded = slice.get_mapped_range();
        let unpadded = (self.width * 4) as usize;
        let mut out = Vec::with_capacity(unpadded * self.height as usize);
        for row in 0..self.height as usize {
            let start = row * self.padded_bytes_per_row as usize;
            out.extend_from_slice(&padded[start..start + unpadded]);
        }
        drop(padded);
        self.readback.unmap();
        out
    }
}

impl Target for TextureTarget {
    fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn acquire(&mut self, _gpu: &Gpu) -> Option<Frame> {
        let view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Some(Frame {
            view,
            surface_texture: None,
        })
    }

    fn present(&mut self, _gpu: &Gpu, _frame: Frame) {
        // Nothing to present; the texture is read back via `read`.
    }
}

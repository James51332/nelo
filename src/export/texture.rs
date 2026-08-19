//! Readback utility for texture-backed rendering.

use std::sync::mpsc;

use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, Device, Extent3d,
    MapMode, Origin3d, PollType, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages,
};

/// Renders to an offscreen texture and reads it back to the CPU (for export).
pub struct ExportTexture {
    pub texture: Texture,
    readback: Buffer,

    width: u32,
    height: u32,

    /// Bytes per row rounded up to `COPY_BYTES_PER_ROW_ALIGNMENT`.
    padded_bytes_per_row: u32,
}

impl ExportTexture {
    pub fn new(device: &Device, format: TextureFormat, width: u32, height: u32) -> Self {
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
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_desc);

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
        let readback = device.create_buffer(&buffer_desc);

        Self {
            texture,
            readback,
            width,
            height,
            padded_bytes_per_row,
        }
    }

    /// Copy the rendered texture to the CPU and return tightly-packed RGBA8.
    pub fn read(&self, device: &Device, queue: &Queue) -> Result<Vec<u8>, String> {
        // Encode the command to read back into the buffer.
        let mut encoder = device.create_command_encoder(&Default::default());
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
        queue.submit(std::iter::once(encoder.finish()));

        // Read back into the buffer.
        let slice = self.readback.slice(..);
        let (tx, rx) = mpsc::channel();
        slice.map_async(MapMode::Read, move |r| {
            let _ = tx.send(r);
        });

        // We are currently waiting until everything on the GPU is done.
        device.poll(PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();

        // Account for the padding.
        let padded = match slice.get_mapped_range() {
            Ok(range) => range,
            Err(e) => {
                return Err(e.to_string());
            }
        };
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
        Ok(out)
    }
}

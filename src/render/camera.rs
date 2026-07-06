//! The camera maps world space to clip space and lives at bind group 0.
//!
//! World space is centred at the origin with a fixed vertical extent
//! (`scene_height` world units); the horizontal extent follows the target
//! aspect ratio. Zoom/pan are just animatable inputs later — for now the
//! uniform is uploaded each frame from the current time and target size.

use crate::render::context::Gpu;
use glam::Mat4;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: Mat4,
    time: f32,
    _pad: [f32; 3],
}

/// Camera is currently defined to be exclusively orthographic, but the
/// future will support multiple cameras and more complex APIs without
/// needing to make significant pipeline changes. Cameras are always
/// set to bind group 0.
pub struct Camera {
    scene_height: f32,
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new(gpu: &Gpu, scene_height: f32) -> Self {
        let buffer = gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("nelo camera uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = gpu
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("nelo camera layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nelo camera bind group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            scene_height,
            buffer,
            layout,
            bind_group,
        }
    }

    /// The bind group layout, needed when building renderer pipelines.
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// The bind group, bound at group 0 by the driver each frame.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Recompute and upload the view-projection for this frame.
    pub fn upload(&self, gpu: &Gpu, time: f32, size: (u32, u32)) {
        let (w, h) = size;
        let aspect = if h == 0 { 1.0 } else { w as f32 / h as f32 };
        let half_h = self.scene_height * 0.5;
        let half_w = half_h * aspect;

        // Orthographic projection, column-major. Maps
        let view_proj = glam::camera::lh::proj::directx::orthographic(
            -half_w, half_w, -half_h, half_h, 0.0, 1.0,
        );

        let uniform = CameraUniform {
            view_proj,
            time: time,
            _pad: [0.0; 3],
        };
        gpu.queue()
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

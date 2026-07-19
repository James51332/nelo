//! The camera maps world space to clip space and lives at bind group 0.
//!
//! World space is centred at the origin with a fixed vertical extent
//! (`scene_height` world units); the horizontal extent follows the target
//! aspect ratio. Zoom/pan are just animatable inputs later — for now the
//! uniform is uploaded each frame from the current time and target size.

use crate::render::context::Gpu;
use crate::scene::Camera;
use glam::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: Mat4,
    time: f32,
    _pad: [f32; 3],
}

/// Contains the data needed to upload a CameraUniform to the GPU at bind group
/// 0.
pub struct CameraBuffer {
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
    pub fn new(gpu: &Gpu) -> Self {
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

    /// Sample and upload a camera for this frame.
    pub fn upload(&self, gpu: &Gpu, size: (u32, u32), camera: &Camera, time: f32) {
        // Sample the camera
        let (scene_height, transform) = camera.sample(time);

        // Construct the view matrix to undo the cameras transformation.
        let inverse = transform.inverse();
        let matrix = inverse.matrix2;
        let translation = inverse.translation;
        let view = Mat4::from_cols(
            Vec4::new(matrix.x_axis.x, matrix.x_axis.y, 0.0, 0.0),
            Vec4::new(matrix.y_axis.x, matrix.y_axis.y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(translation.x, translation.y, 0.0, 1.0),
        );

        // Project the world into device space.
        let (width, height) = size;
        let aspect = if height != 0 {
            width as f32 / height as f32
        } else {
            1.0
        };
        let proj = glam::camera::lh::proj::directx::orthographic(
            -scene_height * aspect * 0.5,
            scene_height * aspect * 0.5,
            -scene_height * 0.5,
            scene_height * 0.5,
            0.0,
            1.0,
        );

        let view_proj = proj * view;
        let uniform = CameraUniform {
            view_proj,
            time,
            _pad: [0.0; 3],
        };
        gpu.queue()
            .write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

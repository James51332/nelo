//! The camera maps world space to clip space and lives at bind group 0.
//!
//! World space is centred at the origin with a fixed vertical extent
//! (`scene_height` world units); the horizontal extent follows the target
//! aspect ratio. Zoom/pan are just animatable inputs later — for now the
//! uniform is uploaded each frame from the current time and target size.

use glam::prelude::*;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    Device, Queue, ShaderStages,
};

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
    buffer: Buffer,
    bind_group: BindGroup,
    layout: BindGroupLayout,
}

impl CameraBuffer {
    pub fn new(device: &Device) -> Self {
        // Create our uniform buffer.
        let buffer_desc = BufferDescriptor {
            label: Some("nelo camera uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&buffer_desc);

        // Create our bind group layout.
        let layout_desc = BindGroupLayoutDescriptor {
            label: Some("nelo camera layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        };
        let layout = device.create_bind_group_layout(&layout_desc);

        // Finally, create the bind group.
        let bind_group_desc = BindGroupDescriptor {
            label: Some("nelo camera bind group"),
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        Self {
            buffer,
            layout,
            bind_group,
        }
    }

    /// The bind group, bound at group 0 by the driver each frame.
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    /// Returns the bind group layout for the camera uniform.
    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    /// Sample and upload a camera for this frame.
    pub fn upload(&self, queue: &Queue, view_proj: &Affine2, time: f32) {
        // Do the work manually to define the camera
        let matrix = view_proj.matrix2;
        let trans = view_proj.translation;
        let view_proj = Mat4::from_cols(
            (matrix.x_axis.x, matrix.x_axis.y, 0.0, 0.0).into(),
            (matrix.y_axis.x, matrix.y_axis.y, 0.0, 0.0).into(),
            (0.0, 0.0, 1.0, 0.0).into(),
            (trans.x, trans.y, 0.0, 1.0).into(),
        );

        let uniform = CameraUniform {
            view_proj,
            time,
            _pad: [0.0; 3],
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

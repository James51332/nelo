//! Circle renderer: instanced, signed-distance-field filled circles.

use crate::render::{Gpu, Renderer};
use crate::scene::{Circle, Fill, Scene, Transform};
use bytemuck::{Pod, Zeroable};
use glam::prelude::*;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, RenderPass, RenderPipeline,
    VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CircleInstance {
    fill: Vec4,        // 16 bytes
    matrix2: Mat2,     // 16 bytes
    translation: Vec2, // 8 bytes
    _pad: Vec2,        // 8 bytes (48 total, 16-aligned)
}

const MAX_CIRCLES: u64 = 100_000;

pub struct CircleRenderer {
    pipeline: RenderPipeline,
    instances: Buffer,
    count: u32,
}

impl CircleRenderer {
    pub fn new(gpu: &Gpu, camera_layout: &BindGroupLayout) -> Self {
        // Create our render pipeline.
        let shader = include_str!("../shaders/circle.wgsl");
        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<CircleInstance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x4, // color       (offset 0)
                1 => Float32x2, // matrix col1 (offset 16)
                2 => Float32x2, // matrix col2 (offset 24)
                3 => Float32x2, // translation (offset 32)
            ],
        };
        let bind_group_layouts = &[Some(camera_layout)];
        let pipeline = gpu.create_pipeline(shader, vertex_layout, bind_group_layouts);

        // Create our instance buffer.
        let desc = BufferDescriptor {
            label: Some("circle buffer"),
            size: size_of::<CircleInstance>() as u64 * MAX_CIRCLES,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let instances = gpu.device().create_buffer(&desc);

        Self {
            pipeline,
            instances,
            count: 0,
        }
    }
}

impl Renderer for CircleRenderer {
    fn prepare(&mut self, gpu: &Gpu, _size: (u32, u32), scene: &Scene, t: f32) {
        let items = scene.view_triple::<Transform, Circle, Fill>();
        let capped = items.len().min(MAX_CIRCLES as usize);
        if capped < items.len() {
            log::warn!(
                "CircleRenderer: {} circles exceeds capacity {MAX_CIRCLES}, dropping {}",
                items.len(),
                items.len() - capped
            );
        }

        let data: Vec<CircleInstance> = items[..capped]
            .iter()
            .map(|(_, transform, _, fill)| {
                let affine = transform.sample(t);
                CircleInstance {
                    fill: fill.sample(t),
                    matrix2: affine.matrix2,
                    translation: affine.translation,
                    _pad: Vec2::ZERO,
                }
            })
            .collect();

        gpu.queue()
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&data));
        self.count = capped as u32;
    }

    fn submit(&self, pass: &mut RenderPass) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

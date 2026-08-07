//! Batch for SDF filled circles.

use crate::render::{BatchComponent, Gpu};
use bytemuck::{Pod, Zeroable, cast_slice};
use glam::prelude::*;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, RenderPass, RenderPipeline,
    VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

// ----- CircleBatch -----

pub struct CircleBatch {
    pipeline: RenderPipeline,
    buffer: Buffer,
    instances: Vec<CircleInstance>,
    capacity: usize,
    count: usize,
    submit_count: usize, // Track the count when last submitted
}

impl CircleBatch {
    pub fn new(gpu: &Gpu, capacity: usize, camera_layout: &BindGroupLayout) -> Self {
        // Create our render pipeline.
        let shader = include_str!("shaders/circle.wgsl");
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

        // Create our gpu buffer.
        let desc = BufferDescriptor {
            label: Some("circle buffer"),
            size: (size_of::<CircleInstance>() * capacity) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = gpu.device().create_buffer(&desc);

        // Create our cpu buffer.
        let instances = vec![CircleInstance::zeroed(); capacity];

        Self {
            pipeline,
            buffer,
            instances,
            capacity,
            count: 0,
            submit_count: 0,
        }
    }

    /// Push a new circle instance onto the batch.
    pub fn push(&mut self, transform: Affine2, fill: Vec4) {
        // Make sure we have room.
        if self.count == self.capacity {
            log::warn!(
                "CircleBatch full ({} circles), dropping circle!",
                self.capacity
            );
            return;
        }

        // Move into our buffer.
        self.instances[self.count] = CircleInstance::new(transform, fill);
        self.count += 1;
    }
}

impl BatchComponent for CircleBatch {
    /// Copy our data buffer to the GPU.
    fn prepare(&mut self, gpu: &Gpu) {
        // Write the buffer.
        let queue = gpu.queue();
        let buffer = &self.buffer;
        let instances = &self.instances;
        queue.write_buffer(buffer, 0, cast_slice(instances));

        // Track how many to submit on our draw call.
        self.submit_count = self.count;
        self.count = 0;
    }

    fn submit(&self, _gpu: &Gpu, pass: &mut RenderPass) {
        // No-op.
        if self.submit_count == 0 {
            return;
        }

        // Encode our draw.
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        pass.draw(0..6, 0..self.submit_count as u32);
    }
}

// ----- CircleInstance ------

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CircleInstance {
    fill: Vec4,        // 16 bytes
    matrix2: Mat2,     // 16 bytes
    translation: Vec2, // 8 bytes
    _pad: Vec2,        // 8 bytes (48 total, 16-aligned)
}

impl CircleInstance {
    pub fn new(transform: Affine2, fill: Vec4) -> Self {
        Self {
            fill,
            matrix2: transform.matrix2,
            translation: transform.translation,
            _pad: Vec2::ZERO,
        }
    }
}

//! Batch for SDF filled circles.

use crate::{render::batch::pipeline, scene::Color};
use bytemuck::{Pod, Zeroable, cast_slice};
use glam::prelude::*;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, Device, Queue, RenderPass,
    RenderPipeline, TextureFormat, VertexBufferLayout, VertexStepMode, vertex_attr_array,
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
    pub fn new(
        device: &Device,
        format: TextureFormat,
        capacity: usize,
        camera_layout: &BindGroupLayout,
    ) -> Self {
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
        let pipeline = pipeline::create(device, format, shader, vertex_layout, bind_group_layouts);

        // Create our gpu buffer.
        let desc = BufferDescriptor {
            label: Some("circle buffer"),
            size: (size_of::<CircleInstance>() * capacity) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&desc);

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

    /// Clears the range of the index batch.
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Push a new circle instance onto the batch. Returns the index of the circle
    /// for submission.
    pub fn push(&mut self, transform: Affine2, fill: Color) -> Option<usize> {
        // Make sure we have room.
        if self.count == self.capacity {
            log::warn!(
                "CircleBatch full ({} circles), dropping circle!",
                self.capacity
            );
            return None;
        }

        // Move into our buffer.
        let index = self.count;
        self.instances[index] = CircleInstance::new(transform, fill);
        self.count += 1;
        Some(index)
    }

    /// Copies the data for this batch to the GPU.
    pub fn prepare(&mut self, queue: &Queue) {
        self.submit_count = self.count;

        // Write the buffer.
        let buffer = &self.buffer;
        let instances = &self.instances[..self.submit_count];
        queue.write_buffer(buffer, 0, cast_slice(instances));
    }

    /// Submit a single circle to the batch. We can also support ranged submissions
    /// in the future.
    pub fn submit(&mut self, pass: &mut RenderPass, index: usize) {
        // Encode our draw.
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.buffer.slice(..));

        // Submit the single instance.
        let index = index as u32;
        pass.draw(0..6, index..index + 1);
    }
}

// ----- CircleInstance ------

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CircleInstance {
    fill: [f32; 4],    // 16 bytes
    matrix2: Mat2,     // 16 bytes
    translation: Vec2, // 8 bytes
    _pad: Vec2,        // 8 bytes (48 total, 16-aligned)
}

impl CircleInstance {
    pub fn new(transform: Affine2, fill: Color) -> Self {
        Self {
            fill: fill.to_array(),
            matrix2: transform.matrix2,
            translation: transform.translation,
            _pad: Vec2::ZERO,
        }
    }
}

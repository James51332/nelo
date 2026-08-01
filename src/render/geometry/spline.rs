//! Batch for polylines. Doesn't handle filling in.

use crate::render::{Batch, Gpu};
use bytemuck::{Pod, Zeroable, cast_slice};
use glam::prelude::*;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, RenderPass, RenderPipeline,
    VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

// ----- SplineBatch -----

pub struct SplineBatch {
    pipeline: RenderPipeline,
    buffer: Buffer,
    instances: Vec<SplineSegment>,
    capacity: usize,
    count: usize,
    submit_count: usize,
}

impl SplineBatch {
    pub fn new(gpu: &Gpu, capacity: usize, camera_layout: &BindGroupLayout) -> Self {
        // Create the render pipeline.
        let shader = include_str!("shaders/curve.wgsl");
        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<SplineSegment>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x2, // point1 (offset 0)
                1 => Float32x2, // point2 (offset 8)
                2 => Float32x4, // color1 (offset 16)
                3 => Float32x4, // color2 (offset 32)
                4 => Float32,   // width1 (offset 48)
                5 => Float32,   // width2 (offset 52)
            ],
        };
        let bind_group_layouts = &[Some(camera_layout)];
        let pipeline = gpu.create_pipeline(shader, vertex_layout, bind_group_layouts);

        // Create our gpu buffer.
        let buffer_desc = BufferDescriptor {
            label: Some("nelo circle instances"),
            size: (size_of::<SplineSegment>() * capacity) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let buffer = gpu.device().create_buffer(&buffer_desc);

        // Create our cpu buffer.
        let instances = vec![SplineSegment::zeroed(); capacity];

        Self {
            pipeline,
            buffer,
            instances,
            capacity,
            count: 0,
            submit_count: 0,
        }
    }

    pub fn push(&mut self, spline: &[SplinePoint]) {
        // Make sure we have real data to render.
        let points = spline.len();
        if points < 2 {
            log::warn!("Cannot push single point spline!");
            return;
        }

        // Check if we have space.
        let segments = points - 1;
        if self.count + segments > self.capacity {
            log::warn!("Not enough space to add spline to batch!");
            return;
        }

        // Move the spline points into segments.
        let data: Vec<SplineSegment> = spline
            .windows(2)
            .map(|points| SplineSegment::new(points[0], points[1]))
            .collect();

        // Move the data into our buffer.
        let range = self.count..(self.count + segments);
        self.instances.splice(range, data);
        self.count += segments;
    }
}

impl Batch for SplineBatch {
    fn prepare(&mut self, gpu: &Gpu) {
        // Copy data to the GPU.
        let queue = gpu.queue();
        let buffer = &self.buffer;
        let instances = &self.instances;
        queue.write_buffer(buffer, 0, cast_slice(&instances[..self.count]));

        // Track how many segments to submit.
        self.submit_count = self.count;
        self.count = 0;
    }

    fn submit(&self, _gpu: &Gpu, pass: &mut RenderPass) {
        // No-op.
        if self.submit_count == 0 {
            return;
        }

        // Submit the draw call.
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        pass.draw(0..6, 0..self.submit_count as u32);
    }
}

// ----- SplinePoint -----

#[derive(Clone, Copy)]
pub struct SplinePoint {
    position: Vec2,
    color: Vec4,
    width: f32,
}

impl SplinePoint {
    pub fn new(position: Vec2, color: Vec4, width: f32) -> Self {
        Self {
            position,
            color,
            width,
        }
    }
}

// ----- SplineSegment -----

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SplineSegment {
    point1: Vec2,
    point2: Vec2,
    color1: Vec4,
    color2: Vec4,
    width1: f32,
    width2: f32,
    _pad: Vec2,
}

impl SplineSegment {
    pub fn new(point1: SplinePoint, point2: SplinePoint) -> Self {
        Self {
            point1: point1.position,
            point2: point2.position,
            color1: point1.color,
            color2: point2.color,
            width1: point1.width,
            width2: point2.width,
            _pad: Vec2::ZERO,
        }
    }
}

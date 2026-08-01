//! A model is a set of vertex data and index data.

use crate::render::{Batch, Gpu};
use bytemuck::{Pod, Zeroable, cast_slice};
use glam::prelude::*;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, IndexFormat, RenderPass,
    RenderPipeline, VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

// ----- ModelBatch -----

pub struct ModelBatch {
    // Render Pipeline
    pipeline: RenderPipeline,

    // Vertex Data
    vertex_buffer: Buffer,
    vertices: Vec<ModelVertex>,
    vertex_capacity: usize,
    vertex_count: usize,
    vertex_submit_count: usize,

    // Index Data
    index_buffer: Buffer,
    indices: Vec<u32>,
    index_capacity: usize,
    index_count: usize,
    index_submit_count: usize,
}

impl ModelBatch {
    pub fn new(
        gpu: &Gpu,
        vertex_capacity: usize,
        index_capacity: usize,
        camera_layout: &BindGroupLayout,
    ) -> Self {
        // Create the render pipeline.
        let shader = include_str!("shaders/model.wgsl");
        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<ModelVertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2, // pos
                1 => Float32x2, // uv
                2 => Float32x4, // color
            ],
        };
        let bind_group_layouts = &[Some(camera_layout)];
        let pipeline = gpu.create_pipeline(shader, vertex_layout, bind_group_layouts);

        // Create our vertex GPU and CPU buffers
        let vertex_buffer_desc = BufferDescriptor {
            label: Some("nelo model vertices"),
            size: (size_of::<ModelVertex>() * vertex_capacity) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let vertex_buffer = gpu.device().create_buffer(&vertex_buffer_desc);
        let vertices = vec![ModelVertex::zeroed(); vertex_capacity];

        // Create our index GPU and CPU buffers
        let index_buffer_desc = BufferDescriptor {
            label: Some("nelo model indices"),
            size: (size_of::<u32>() * index_capacity) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let index_buffer = gpu.device().create_buffer(&index_buffer_desc);
        let indices = vec![u32::zeroed(); index_capacity];

        Self {
            pipeline,
            vertex_buffer,
            vertices,
            vertex_capacity,
            vertex_count: 0,
            vertex_submit_count: 0,
            index_buffer,
            indices,
            index_capacity,
            index_count: 0,
            index_submit_count: 0,
        }
    }

    pub fn push(&mut self, vertices: &[ModelVertex], indices: &[u32]) {
        // Make sure that we have appropriate data to submit.
        let vertex_count = vertices.len();
        if self.vertex_count + vertex_count > self.vertex_capacity {
            log::warn!("Not enough vertex space left!");
            return;
        }

        let index_count = indices.len();
        if self.index_count + index_count > self.index_capacity {
            log::warn!("Not enough index space left!");
            return;
        }

        // Index data should be in groups of three.
        if index_count % 3 != 0 {
            log::warn!(
                "Model index count {} is not divisible by three!",
                index_count
            );
            return;
        }

        // Copy the vertex data into the vertex buffer.
        let vertex_range = self.vertex_count..(self.vertex_count + vertex_count);
        self.vertices.splice(vertex_range, vertices.iter().copied());
        self.vertex_count += vertex_count;

        // Copy the index data into the index buffer.
        let index_range = self.index_count..(self.index_count + index_count);
        self.indices.splice(
            index_range,
            indices.iter().map(|&idx| idx + self.index_count as u32),
        );
        self.index_count += index_count;
    }
}

impl Batch for ModelBatch {
    fn prepare(&mut self, gpu: &Gpu) {
        // Copy vertex data to the GPU.
        let queue = gpu.queue();
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            cast_slice(&self.vertices[..self.vertex_count]),
        );

        // Copy index data to the GPU.
        queue.write_buffer(
            &self.index_buffer,
            0,
            cast_slice(&self.indices[..self.index_count]),
        );

        // Track submission values.
        self.vertex_submit_count = self.vertex_count;
        self.index_submit_count = self.index_count;
        self.vertex_count = 0;
        self.index_count = 0;
    }

    fn submit(&self, _gpu: &Gpu, pass: &mut RenderPass) {
        // No-op.
        if self.vertex_submit_count == 0 || self.index_submit_count == 0 {
            return;
        }

        // Submit the draw call.
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..self.index_submit_count as u32, 0, 0..1);
    }
}

// ----- ModelVertex -----

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ModelVertex {
    position: Vec2,
    uv: Vec2,
    color: Vec4,
}

impl ModelVertex {
    pub fn new(position: Vec2, uv: Vec2, color: Vec4) -> Self {
        Self {
            position,
            uv,
            color,
        }
    }
}

//! A mesh is a set of vertex data and index data.

use std::ops::Range;

use crate::render::Gpu;
use bytemuck::{Pod, Zeroable, cast_slice};
use glam::prelude::*;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, IndexFormat, RenderPass,
    RenderPipeline, VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

// ----- MeshBatch -----

pub struct MeshBatch {
    // Render Pipeline
    pipeline: RenderPipeline,

    // Vertex Data
    vertex_buffer: Buffer,
    vertices: Vec<MeshVertex>,
    vertex_capacity: usize,
    vertex_count: usize,

    // Index Data
    index_buffer: Buffer,
    indices: Vec<u32>,
    index_capacity: usize,
    index_count: usize,

    // Submission Data
    submit_count: usize,
    submissions: Vec<Range<usize>>,
}

impl MeshBatch {
    pub fn new(
        gpu: &Gpu,
        vertex_capacity: usize,
        index_capacity: usize,
        camera_layout: &BindGroupLayout,
    ) -> Self {
        // Create the render pipeline.
        let shader = include_str!("shaders/mesh.wgsl");
        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<MeshVertex>() as u64,
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
            size: (size_of::<MeshVertex>() * vertex_capacity) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let vertex_buffer = gpu.device().create_buffer(&vertex_buffer_desc);
        let vertices = vec![MeshVertex::zeroed(); vertex_capacity];

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
            index_buffer,
            indices,
            index_capacity,
            index_count: 0,
            submit_count: 0,
            submissions: Vec::new(),
        }
    }

    pub fn push(&mut self, vertices: &[MeshVertex], indices: &[u32]) -> Option<usize> {
        // Make sure that we have appropriate data to submit.
        let vertex_count = vertices.len();
        if self.vertex_count + vertex_count > self.vertex_capacity {
            log::warn!("Not enough vertex space left!");
            return None;
        }

        let index_count = indices.len();
        if self.index_count + index_count > self.index_capacity {
            log::warn!("Not enough index space left!");
            return None;
        }

        // Index data should be in groups of three.
        if index_count % 3 != 0 {
            log::warn!(
                "Model index count {} is not divisible by three!",
                index_count
            );
            return None;
        }

        // Copy the vertex data into the vertex buffer.
        let vertex_range = self.vertex_count..(self.vertex_count + vertex_count);
        self.vertices.splice(vertex_range, vertices.iter().copied());
        self.vertex_count += vertex_count;

        // Copy the index data into the index buffer.
        let vertex_base = self.vertex_count - vertex_count;
        let index_range = self.index_count..(self.index_count + index_count);
        self.indices.splice(
            index_range.clone(),
            indices.iter().map(|&idx| idx + vertex_base as u32),
        );
        self.index_count += index_count;

        // Encode the range into our submission
        let index = self.submissions.len();
        self.submissions.push(index_range);
        Some(index)
    }

    pub fn clear(&mut self) {
        self.index_count = 0;
        self.vertex_count = 0;
        self.submissions.clear();
    }

    /// Copies mesh data to the GPU.
    pub fn prepare(&mut self, gpu: &Gpu) {
        self.submit_count = self.submissions.len();

        // Copy index data
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
    }

    pub fn submit(&self, _gpu: &Gpu, pass: &mut RenderPass, index: usize) {
        // No-op.
        if self.vertex_count == 0 || self.index_count == 0 {
            return;
        }

        if index >= self.submit_count {
            log::warn!("Invalid submission index");
            return;
        }

        // Set the pipeline state.
        pass.set_pipeline(&self.pipeline);
        let vertex_bytes = (self.vertex_count * size_of::<MeshVertex>()) as u64;
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..vertex_bytes));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        // Submit the draw call.
        let range = &self.submissions[index];
        let range = (range.start as u32)..(range.end as u32);
        pass.draw_indexed(range, 0, 0..1);
    }
}

// ----- MeshVertex -----

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Vec4,
}

impl MeshVertex {
    pub fn new(position: Vec2, uv: Vec2, color: Vec4) -> Self {
        Self {
            position,
            uv,
            color,
        }
    }
}

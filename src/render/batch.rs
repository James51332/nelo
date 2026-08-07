//! A set of geometry (polygons, splines, and sdf shapes) generated from scene

pub mod builder;
pub mod circle;
pub mod mesh;
pub mod polyline;
pub mod shapes;

pub use builder::{FillBuilder, Segment, StrokeBuilder, StrokePoint};
pub use circle::CircleBatch;
pub use mesh::{MeshBatch, MeshVertex};
pub use polyline::Polyline;

use crate::render::Gpu;
use glam::prelude::*;
use wgpu::{BindGroupLayout, RenderPass};

const MAX_CIRCLES: usize = 100_000;
const MAX_VERTICES: usize = 100_000;
const MAX_INDICES: usize = 50_000;
const BUILDER_TOLERANCE: f32 = 0.001;

/// A batch is a reusable geometry renderer.
pub struct Batch {
    circles: CircleBatch,
    meshes: MeshBatch,
}

impl Batch {
    pub fn new(gpu: &Gpu, camera_layout: &BindGroupLayout) -> Self {
        Self {
            circles: CircleBatch::new(gpu, MAX_CIRCLES, camera_layout),
            meshes: MeshBatch::new(gpu, MAX_VERTICES, MAX_INDICES, camera_layout),
        }
    }

    pub fn prepare(&mut self, gpu: &Gpu) {
        self.circles.prepare(gpu);
        self.meshes.prepare(gpu);
    }

    pub fn submit(&mut self, gpu: &Gpu, pass: &mut RenderPass) {
        self.circles.submit(gpu, pass);
        self.meshes.submit(gpu, pass);
    }

    /// Push a new circle instance onto the batch.
    pub fn add_circle(&mut self, transform: Affine2, fill: Vec4) {
        self.circles.push(transform, fill);
    }

    pub fn add_mesh(&mut self, vertices: &[MeshVertex], indices: &[u32]) {
        self.meshes.push(vertices, indices);
    }

    pub fn fill_builder(&mut self, scale: f32) -> FillBuilder<'_> {
        FillBuilder::new(&mut self.meshes, scale * BUILDER_TOLERANCE)
    }

    pub fn stroke_builder(&mut self, start: StrokePoint, scale: f32) -> StrokeBuilder<'_> {
        StrokeBuilder::new(&mut self.meshes, start, scale * BUILDER_TOLERANCE)
    }

    pub fn tolerance(&self) -> f32 {
        BUILDER_TOLERANCE
    }
}

pub trait BatchComponent {
    /// Prepares the renderer to draw. Copies data into buffers.
    fn prepare(&mut self, gpu: &Gpu);

    /// Submits the draw calls into a render pass. Clears the batch.
    fn submit(&self, gpu: &Gpu, pass: &mut RenderPass);
}

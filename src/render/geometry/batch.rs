//! A set of geometry (polygons, splines, and sdf shapes) generated from scene

use crate::render::{CircleBatch, Gpu, SplineBatch};
use wgpu::{BindGroupLayout, RenderPass};

/// This trait is used to enable batches to submit their work to the GPU.
/// In general, batches are not anonymized to this trait.
pub trait Batch {
    /// Prepares the renderer to draw. Copies data into buffers.
    fn prepare(&mut self, gpu: &Gpu);

    /// Submits the draw calls into a render pass. Clears the batch.
    fn submit(&self, gpu: &Gpu, pass: &mut RenderPass);
}

/// Holds all of the implemented batch types
pub struct BatchSet {
    pub circles: CircleBatch,
    pub splines: SplineBatch,
}

const MAX_CIRCLES: usize = 100_000;
const MAX_SEGMENTS: usize = 100_000;

impl BatchSet {
    pub fn new(gpu: &Gpu, camera_layout: &BindGroupLayout) -> Self {
        Self {
            circles: CircleBatch::new(gpu, MAX_CIRCLES, camera_layout),
            splines: SplineBatch::new(gpu, MAX_SEGMENTS, camera_layout),
        }
    }
}

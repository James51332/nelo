use crate::render::Gpu;
use glam::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Geometry {
    Primitive(Primitive),
    Curve, // TODO: Curves
    Custom,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Primitive {
    Circle,
}

#[derive(Clone)]
pub struct Renderable {
    pub geometry: Geometry,
    pub transform: Affine2,
    pub fill: Vec4,
}

/// The `Renderer` trait is required for any type that wishes to render a scene.
/// Renderers which are designed for a specific type of geometry should filter
/// by overriding geometry method. All renderers should be mindful of the camera
/// at bind group one.
pub trait Renderer {
    /// A filter to allow the renderer to only process a certain type of geometry.
    fn geometry(&self) -> Option<&'static [Geometry]> {
        None
    }

    /// Prepares the renderer to draw. Copies data into buffers.
    fn prepare(&mut self, gpu: &Gpu, geometry: &[Renderable]);

    /// Submits the draw calls into a render pass.
    fn submit<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}

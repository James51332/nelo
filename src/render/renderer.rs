use crate::render::Gpu;
use crate::scene::Scene;

/// The `Renderer` trait is required for any type that wishes to render a scene.
/// Renderers which are designed for a specific type of geometry should filter
/// by overriding geometry method. All renderers should be mindful of the camera
/// at bind group one.
pub trait Renderer: 'static {
    /// Prepares the renderer to draw. Copies data into buffers.
    fn prepare(&mut self, gpu: &Gpu, scene: &Scene, t: f32);

    /// Submits the draw calls into a render pass.
    fn submit<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}

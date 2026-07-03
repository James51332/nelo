//! Rendering: the GPU-facing half of nelo.
//!
//! Three responsibilities are kept separate:
//! * [`Gpu`](crate::context::Gpu) — owns the device and queue.
//! * [`Target`] — *where* a frame is drawn (a window swapchain or an offscreen
//!   texture for export). One trait, two implementations.
//! * [`Renderer`] — *what* to draw. Owns its pipeline and buffers; uploads in
//!   [`Renderer::prepare`], records draws in [`Renderer::draw`].
//!
//! Multiple renderers share a single render pass per frame. The driver
//! (see the `native` binary and the `headless` example) samples the scene,
//! calls `prepare` on each renderer, opens one pass, binds the camera, and
//! lets each renderer record its draws.

pub mod camera;
pub mod circle;
pub mod target;

pub use camera::Camera;
pub use circle::{Circle, CircleRenderer};
pub use target::{Frame, Target, TextureTarget, WindowTarget};

use crate::context::Gpu;

/// Per-frame context handed to renderers during [`Renderer::prepare`].
pub struct FrameCtx<'a> {
    pub gpu: &'a Gpu,
    /// Wall-clock time of the frame, in seconds.
    pub time: f64,
    /// Target size in physical pixels.
    pub size: (u32, u32),
}

/// A renderer draws one kind of primitive.
///
/// The lifecycle is split so uploads (which need `&mut self` and the queue)
/// happen before the borrowed render pass:
/// 1. [`prepare`](Renderer::prepare) — pack `items` into GPU buffers.
/// 2. [`draw`](Renderer::draw) — record draw calls into the shared pass.
pub trait Renderer {
    /// What this renderer consumes from the scene each frame.
    type Item;

    /// Upload/update GPU state for this frame. Runs before the pass.
    fn prepare(&mut self, ctx: &FrameCtx, items: &[Self::Item]);

    /// Record draw calls into the shared pass. No clears, no uploads here.
    /// The camera is already bound at group 0 by the driver.
    fn draw<'p>(&'p self, pass: &mut wgpu::RenderPass<'p>);
}

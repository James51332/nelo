//! All render facing code in the animation engine.
//!
//! Three responsibilities are kept separate:
//! * [`Gpu`] — owns the device and queue.
//! * [`Target`] — where a frame is drawn (a window swapchain or an offscreen
//!   texture for export). One trait, two implementations.
//! * [`Renderer`] — *what* to draw. Owns its pipeline and buffers and draws [`Geometry`]

pub mod camera;
pub mod circle;
pub mod context;
pub mod renderer;
pub mod scene;
pub mod target;

pub use camera::CameraBuffer;
pub use circle::CircleRenderer;
pub use context::Gpu;
pub use renderer::Renderer;
pub use scene::SceneRenderer;
pub use target::{Frame, Target, TextureTarget, WindowTarget};

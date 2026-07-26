//! All render facing code in the animation engine.

pub mod camera;
pub mod geometry;
pub mod gpu;
pub mod scene;

pub use camera::CameraBuffer;
pub use geometry::{CircleRenderer, CurveRenderer, Renderer, tesselate};
pub use gpu::{Frame, Gpu, Target, TextureTarget, WindowTarget};
pub use scene::SceneRenderer;

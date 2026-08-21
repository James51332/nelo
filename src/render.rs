//! All render facing code in the animation engine.

pub mod batch;
pub mod camera;
pub mod renderer;

pub use batch::{Batch, Encoder, FillBuilder, MeshVertex, Polyline, RenderCommand, StrokeVertex};
pub use camera::CameraBuffer;
pub use renderer::{ComponentRenderer, Renderer};

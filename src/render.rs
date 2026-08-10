//! All render facing code in the animation engine.

pub mod batch;
pub mod camera;
pub mod color;
pub mod gpu;
pub mod renderer;

pub use batch::{
    Batch, CircleBatch, FillBuilder, MeshBatch, MeshVertex, Polyline, RenderCommand, Segment,
    StrokePoint,
};
pub use camera::CameraBuffer;
pub use color::Color;
pub use gpu::{Frame, Gpu, Target, TextureTarget, WindowTarget};
pub use renderer::{ComponentRenderer, Renderer};

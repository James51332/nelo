//! All render facing code in the animation engine.

pub mod batch;
pub mod camera;
pub mod gpu;
pub mod renderer;

pub use batch::{
    Batch, BatchComponent, CircleBatch, FillBuilder, MeshBatch, MeshVertex, Polyline, Segment,
    StrokePoint,
};
pub use camera::CameraBuffer;
pub use gpu::{Frame, Gpu, Target, TextureTarget, WindowTarget};
pub use renderer::{ComponentRenderer, Renderer};

//! Each render command holds all data needed to complete a full render operation.

use crate::render::{Color, MeshVertex, StrokePoint};
use glam::prelude::*;

/// A render command holds all of the data needs to render a geometry.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// A circle with initial radius of one transformed by specified affine.
    Circle {
        transform: Affine2,
        color: Color,
    },

    /// A stroke with variable width.
    Stroke {
        vertices: Vec<StrokePoint>,
        close: bool,
    },

    Polygon {
        vertices: Vec<MeshVertex>,
    },

    Mesh {
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
    },
}

//! Each render command holds all data needed to complete a full render operation.

use crate::render::Color;
use bytemuck::{Pod, Zeroable};
use glam::prelude::*;

// ----- StrokePoint -----

#[derive(Debug, Clone)]
pub struct StrokeVertex {
    pub position: Vec2,
    pub color: [f32; 4],
    pub width: f32,
}

impl StrokeVertex {
    pub fn new(position: Vec2, color: Color, width: f32) -> Self {
        Self {
            position,
            color: color.to_array(),
            width,
        }
    }
}

// ----- MeshVertex -----

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: [f32; 4],
}

impl MeshVertex {
    pub fn new(position: Vec2, uv: Vec2, color: Color) -> Self {
        Self {
            position,
            uv,
            color: color.to_array(),
        }
    }
}

// ----- RenderCommand -----

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
        vertices: Vec<StrokeVertex>,
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

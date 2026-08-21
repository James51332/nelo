//! Geometry builder for tesselation.

use crate::render::{MeshVertex, RenderCommand, StrokeVertex};
use crate::scene::Color;
use glam::prelude::*;
use lyon::{
    math::Point,
    path::{BuilderWithAttributes, LineCap, LineJoin, Path},
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
        TessellationError, VertexBuffers,
    },
};

/// This shouldn't matter since we are only rendering polylines. However, lyon does sometimes try to
/// aggregate for us, which we don't like. Therefore, we keep it relatively small.
const BUILDER_TOLERANCE: f32 = 0.001;

// ----- FillBuilder -----

pub struct FillBuilder {
    builder: BuilderWithAttributes,
    last_point: Option<MeshVertex>,
}

impl Default for FillBuilder {
    fn default() -> Self {
        Self {
            builder: Path::builder_with_attributes(4),
            last_point: None,
        }
    }
}

impl FillBuilder {
    /// Performs a tesselation of this path and adds to the batch if successful.
    /// Closes any open, subpath with looping if open.
    pub fn finish(mut self) -> Result<RenderCommand, TessellationError> {
        // Close if open.
        if self.last_point.is_some() {
            self.builder.end(true);
        }

        // Build the path and our buffers.
        let path = self.builder.build();
        let mut buffers: VertexBuffers<MeshVertex, u32> = VertexBuffers::new();
        let mut geometry = BuffersBuilder::new(&mut buffers, |mut vertex: FillVertex| {
            let position = vertex.position().to_array().into();
            let color = Color::from_slice(vertex.interpolated_attributes());
            MeshVertex::new(position, Vec2::ZERO, color)
        });

        // Perform the tesselation
        let mut tesselator = FillTessellator::new();
        let options = FillOptions::tolerance(BUILDER_TOLERANCE);
        let result = tesselator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut geometry,
        );

        // Handle success and failure.
        result.map(|_| RenderCommand::Mesh {
            vertices: buffers.vertices,
            indices: buffers.indices,
        })
    }

    /// Begins a new subpath, ending previous (closing loop) if needed.
    pub fn begin_subpath(&mut self, start: MeshVertex) {
        if self.last_point.is_some() {
            self.builder.end(true);
        }

        self.builder.begin(point(start.position), &start.color);
        self.last_point = Some(start);
    }

    /// Adds a line segment to the current subpath, or no-op.
    pub fn line_to(&mut self, end: MeshVertex) {
        if self.last_point.is_some() {
            self.builder.line_to(point(end.position), &end.color);
            self.last_point = Some(end);
        }
    }

    /// Ends the current subpath, or no-op.
    pub fn end_subpath(&mut self, close: bool) {
        if self.last_point.is_some() {
            self.builder.end(close);
            self.last_point = None;
        }
    }
}

// ----- StrokeBuilder -----

pub struct StrokeBuilder {
    builder: BuilderWithAttributes,
}

impl StrokeBuilder {
    pub fn new(start: StrokeVertex) -> Self {
        let mut builder = Path::builder_with_attributes(5);
        builder.begin(point(start.position), &stroke(start));
        Self { builder }
    }

    /// Performs a tesselation of this path and adds to the batch if successful.
    pub fn finish(mut self, close: bool) -> Result<RenderCommand, TessellationError> {
        // End the spline, but don't close the loop.
        self.builder.end(close);

        // Build the path and our buffers.
        let path = self.builder.build();
        let mut buffers: VertexBuffers<MeshVertex, u32> = VertexBuffers::new();
        let mut geometry = BuffersBuilder::new(
            &mut buffers,
            |mut vertex: lyon::tessellation::StrokeVertex| {
                let position = vertex.position().to_array().into();
                let color = Color::from_slice(vertex.interpolated_attributes().into());
                MeshVertex::new(position, Vec2::ZERO, color)
            },
        );

        // Perform the tesselation
        let mut tesselator = StrokeTessellator::new();
        let options = StrokeOptions::tolerance(BUILDER_TOLERANCE)
            .with_start_cap(LineCap::Round)
            .with_end_cap(LineCap::Round)
            .with_line_join(LineJoin::Miter)
            .with_variable_line_width(4);
        let result = tesselator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut geometry,
        );

        // Return our render command.
        result.map(|_| RenderCommand::Mesh {
            vertices: buffers.vertices,
            indices: buffers.indices,
        })
    }

    pub fn line_to(&mut self, end: StrokeVertex) {
        self.builder.line_to(point(end.position), &stroke(end));
    }
}

// ----- Helper -----

fn point(pos: Vec2) -> Point {
    Point::new(pos.x, pos.y)
}

/// Returns the stroke attributes. Width is index 4.
fn stroke(point: StrokeVertex) -> [f32; 5] {
    [
        point.color[0],
        point.color[1],
        point.color[2],
        point.color[3],
        point.width,
    ]
}

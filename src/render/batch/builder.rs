//! Geometry builder for tesselation.

use crate::render::{Color, MeshVertex, batch::RenderCommand};
use glam::prelude::*;
use lyon::{
    math::Point,
    path::{BuilderWithAttributes, LineCap, LineJoin, Path},
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
        StrokeVertex, TessellationError, VertexBuffers,
    },
};

// ----- Segment -----

pub enum Segment {
    Line(MeshVertex, MeshVertex),
    Quad(MeshVertex, Vec2, MeshVertex),
    Cubic(MeshVertex, Vec2, Vec2, MeshVertex),
}

impl Segment {
    /// Returns the first point along this segment.
    pub fn start(&self) -> MeshVertex {
        match *self {
            Self::Line(s, _) | Self::Quad(s, _, _) | Self::Cubic(s, _, _, _) => s,
        }
    }
}

// ----- FillBuilder -----

pub struct FillBuilder {
    builder: BuilderWithAttributes,
    last_point: Option<MeshVertex>,
    tolerance: f32,
}

impl FillBuilder {
    pub fn new(tolerance: f32) -> Self {
        Self {
            builder: Path::builder_with_attributes(4),
            last_point: None,
            tolerance,
        }
    }

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
        let options = FillOptions::tolerance(self.tolerance);
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

    /// Add a segment to this geometry. If the segment doesn't align with the
    /// last point, begin a new subpath.
    pub fn add_segment(&mut self, segment: Segment) {
        let start = segment.start();

        // Optionally start a new segment.
        match self.last_point {
            Some(end) if end.position != start.position => {
                self.builder.end(true);
                self.builder.begin(point(start.position), &start.color);
            }
            None => {
                self.builder.begin(point(start.position), &start.color);
            }
            _ => (),
        };

        // Emit the segment.
        let end = match segment {
            Segment::Line(_, end) => {
                self.builder.line_to(point(end.position), &end.color);
                end
            }
            Segment::Quad(_, c1, end) => {
                self.builder
                    .quadratic_bezier_to(point(c1), point(end.position), &end.color);
                end
            }
            Segment::Cubic(_, c1, c2, end) => {
                self.builder
                    .cubic_bezier_to(point(c1), point(c2), point(end.position), &end.color);
                end
            }
        };

        self.last_point = Some(end);
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
    tolerance: f32,
}

impl StrokeBuilder {
    pub fn new(start: StrokePoint, tolerance: f32) -> Self {
        let mut builder = Path::builder_with_attributes(5);
        builder.begin(point(start.position), &stroke(start));
        Self { builder, tolerance }
    }

    /// Performs a tesselation of this path and adds to the batch if successful.
    pub fn finish(mut self, close: bool) -> Result<RenderCommand, TessellationError> {
        // End the spline, but don't close the loop.
        self.builder.end(close);

        // Build the path and our buffers.
        let path = self.builder.build();
        let mut buffers: VertexBuffers<MeshVertex, u32> = VertexBuffers::new();
        let mut geometry = BuffersBuilder::new(&mut buffers, |mut vertex: StrokeVertex| {
            let position = vertex.position().to_array().into();
            let color = Color::from_slice(vertex.interpolated_attributes().into());
            MeshVertex::new(position, Vec2::ZERO, color)
        });

        // Perform the tesselation
        let mut tesselator = StrokeTessellator::new();
        let options = StrokeOptions::tolerance(self.tolerance)
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

    pub fn line_to(&mut self, end: StrokePoint) {
        self.builder.line_to(point(end.position), &stroke(end));
    }
}

// ----- StrokePoint -----

#[derive(Debug, Clone)]
pub struct StrokePoint {
    pub position: Vec2,
    pub color: [f32; 4],
    pub width: f32,
}

impl StrokePoint {
    pub fn new(position: Vec2, color: Color, width: f32) -> Self {
        Self {
            position,
            color: color.to_array(),
            width,
        }
    }
}

// ----- Helper -----

fn point(pos: Vec2) -> Point {
    Point::new(pos.x, pos.y)
}

/// Returns the stroke attributes. Width is index 4.
fn stroke(point: StrokePoint) -> [f32; 5] {
    [
        point.color[0],
        point.color[1],
        point.color[2],
        point.color[3],
        point.width,
    ]
}

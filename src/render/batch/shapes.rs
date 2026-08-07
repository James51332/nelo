//! Some basic shapes renderers for batch.

use crate::render::{Batch, MeshVertex, StrokePoint};

impl Batch {
    /// Render a filled triangle.
    pub fn traingle(&mut self, p0: MeshVertex, p1: MeshVertex, p2: MeshVertex) {
        let vertices = vec![p0, p1, p2];
        let indices = [0, 1, 2];
        self.add_mesh(&vertices, &indices);
    }

    /// Renders a line. Use `Batch::stroke_builder()` for complex splines.
    pub fn line(&mut self, p0: StrokePoint, p1: StrokePoint) {
        let mut builder = self.stroke_builder(p0, 1.0);
        builder.line_to(p1);
        if let Err(e) = builder.finish(false) {
            log::warn!("Failed to render line: {e}");
        }
    }
}

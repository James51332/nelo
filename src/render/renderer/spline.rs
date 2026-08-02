//! Helper methods to render splines

use crate::render::{BatchSet, ModelVertex, SplinePoint, tesselate};
use crate::scene::{Fill, Scene, Spline, Stroke, Transform};
use glam::prelude::*;
use lyon::tessellation::{
    FillOptions, FillTessellator, VertexBuffers, geometry_builder::simple_builder, math::Point,
};

pub fn filled_splines(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_triple::<Transform, Spline, Fill>();
    items.iter().for_each(|(_, transform, spline, fill)| {
        // Apply the curves transformation.
        let affine = transform.sample(t);
        let spline_path = spline
            .spline_path
            .sample(t)
            .timeline()
            .map(move |x| affine.matrix2 * x + affine.translation)
            .along();

        // Subdivide the curve into a polyline.
        let polyline = tesselate::generate_polyline(
            &spline_path,
            spline.start_alpha.sample(t),
            spline.end_alpha.sample(t),
        );

        // Skip this curve
        if polyline.len() < 3 {
            return;
        }

        // Tesselate the polyline using lyon-rs.
        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut vertex_builder = simple_builder(&mut buffers);
        let mut tesselator = FillTessellator::new();
        let options = FillOptions::default();

        let mut builder = tesselator.builder(&options, &mut vertex_builder);
        let start = polyline[0].1;

        builder.begin(Point::new(start.x, start.y));
        for (_, point) in polyline[1..].iter() {
            builder.line_to(Point::new(point.x, point.y));
        }
        builder.end(true);

        let result = builder.build();
        if let Err(msg) = result {
            log::error!("Tesselation failed: {}", msg);
            return;
        }

        // Submit our triangle data to the GPU.
        let color = fill.color.sample(t);
        let vertices: Vec<_> = buffers
            .vertices
            .into_iter()
            .map(|point| ModelVertex::new(Vec2::new(point.x, point.y), Vec2::ZERO, color))
            .collect();

        let indices: Vec<_> = buffers
            .indices
            .into_iter()
            .map(|idx: u16| idx as u32)
            .collect();

        batches.models.push(&vertices, &indices);
    });
}

pub fn stroked_splines(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_triple::<Transform, Spline, Stroke>();

    items.iter().for_each(|(_, transform, spline, stroke)| {
        // Apply the transformation.
        let affine = transform.sample(t);
        let spline_path = spline
            .spline_path
            .sample(t)
            .timeline()
            .map(move |x| affine.matrix2 * x + affine.translation)
            .along();

        // Subdivide the curve into a polyline.
        let polyline = tesselate::generate_polyline(
            &spline_path,
            spline.start_alpha.sample(t),
            spline.end_alpha.sample(t),
        );

        // Convert the (alpha, pos) values into renderable points.
        let spline_points: Vec<_> = polyline
            .into_iter()
            .map(|(a, pos)| {
                SplinePoint::new(
                    pos,
                    stroke.color.sample(t).sample(a),
                    stroke.weight.sample(t).sample(a),
                )
            })
            .collect();

        // Submit the curve to the batch.
        batches.splines.push(&spline_points);
    });
}

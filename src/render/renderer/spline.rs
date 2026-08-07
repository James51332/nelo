//! Helper methods to render splines

use crate::render::{Batch, MeshVertex, Polyline};
use crate::scene::{Arrow, Fill, Scene, Spline, Stroke, Transform, Visibility};
use glam::{Mat2, Vec2};

pub fn splines(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_pair::<Transform, Spline>();
    items.into_iter().for_each(|(id, transform, spline)| {
        // Visibility short circuit.
        let visibility = scene
            .component::<Visibility>(id)
            .map_or(1.0, |v| v.amount.sample(time));

        if visibility <= 0.005 {
            return;
        }

        // Apply the curves transformation before subdivision.
        let affine = transform.sample(time);
        let spline_path = spline
            .spline_path
            .sample(time)
            .timeline()
            .map(move |x| affine.matrix2 * x + affine.translation)
            .along();

        // Subdivide the curve into a polyline of at least three points.
        let polyline = Polyline::flatten(
            &spline_path,
            spline.start_alpha.sample(time),
            spline.end_alpha.sample(time) * visibility,
            batch.tolerance(),
        );

        // Render fill and stroke appropriately.
        if let Some(fill) = scene.component::<Fill>(id) {
            let mut color = fill.color.sample(time);
            color.w *= visibility * visibility * visibility;
            batch.fill(polyline.clone(), |_| color);
        };

        if let Some(stroke) = scene.component::<Stroke>(id) {
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            batch.stroke(polyline, move |a| (color.sample(a), weight.sample(a)));
        };
    });
}

/// Mostly the same as the spline renderer. However, we never fill arrows, and we add
/// a triangle at the end.
pub fn arrows(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    let items = scene.view_triple::<Transform, Arrow, Stroke>();
    items
        .into_iter()
        .for_each(|(id, transform, arrow, stroke)| {
            // Visibility short circuit.
            let visibility = scene
                .component::<Visibility>(id)
                .map_or(1.0, |v| v.amount.sample(time));

            if visibility <= 0.005 {
                return;
            }

            // Sample the arrow
            let affine = transform.sample(time);
            let spline = &arrow.spline;
            let start = spline.start_alpha.sample(time);
            let end = spline.end_alpha.sample(time) * visibility;
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            let spline_path = spline
                .spline_path
                .sample(time)
                .timeline()
                .map(move |x| affine.matrix2 * x + affine.translation)
                .along();

            // Subdivide the curve into a polyline of at least three points.
            let polyline = Polyline::flatten(&spline_path, start, end, batch.tolerance());

            // Compute how we move our triangle. Triangle isn't affected by transform.
            let points = polyline.points();
            let num_points = points.len();
            let rotate = if num_points >= 2 {
                let dir = points[num_points - 1].1 - points[num_points - 2].1;
                let norm = dir.normalize_or(Vec2::X);
                Mat2::from_cols(norm, Vec2::new(-norm.y, norm.x))
            } else {
                Mat2::default()
            };
            let last = points.last().map_or(Vec2::ZERO, |v| v.1);

            // Add a triangle in the same direction as the spline to the last point.
            const SCALE: f32 = 2.5;
            const VERTICES: [Vec2; 3] = [
                Vec2::new(1.0, 0.0),
                Vec2::new(-1.0, 1.0),
                Vec2::new(-1.0, -1.0),
            ];
            let size = SCALE * weight.sample(end);
            let map = |v| MeshVertex::new(rotate * size * v + last, Vec2::ZERO, color.sample(end));
            batch.traingle(map(VERTICES[0]), map(VERTICES[1]), map(VERTICES[2]));

            // Add the spline to the render.
            batch.stroke(polyline, move |a| (color.sample(a), weight.sample(a)));
        });
}

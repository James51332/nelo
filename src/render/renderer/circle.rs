//! Helper methods to render circles

use crate::render::{BatchSet, SplinePoint, tesselate};
use crate::scene::{Circle, Fill, Scene, Stroke, Transform, path};

pub fn filled_circles(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all elements with the required components.
    let items = scene.view_triple::<Transform, Circle, Fill>();

    // Submit a circle for each one.
    items.iter().for_each(|(_, transform, _, fill)| {
        batches
            .circles
            .push(transform.sample(t), fill.color.sample(t));
    });
}

pub fn stroked_circles(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Find the circles with transform and stroke.
    let items = scene.view_triple::<Transform, Circle, Stroke>();

    items.iter().for_each(|(_, transform, _, stroke)| {
        let affine = transform.sample(t);
        let spline = path::circle().map(move |x| affine.matrix2 * x + affine.translation);
        let polyline = tesselate::generate_polyline(&spline.along(), 0.0, 1.0);
        let points: Vec<SplinePoint> = polyline
            .into_iter()
            .map(|(a, pos)| {
                SplinePoint::new(
                    pos,
                    stroke.color.sample(t).sample(a),
                    stroke.weight.sample(t).sample(a),
                )
            })
            .collect();

        batches.splines.push(&points);
    });
}

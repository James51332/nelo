//! Helper methods to render splines

use crate::render::{Batch, Polyline};
use crate::scene::{Fill, Scene, Spline, Stroke, Transform};

pub fn splines(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_pair::<Transform, Spline>();
    items.into_iter().for_each(|(id, transform, spline)| {
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
            spline.end_alpha.sample(time),
            batch.tolerance(),
        );

        // Render fill and stroke appropriately.
        if let Some(fill) = scene.component::<Fill>(id) {
            let color = fill.color.sample(time);
            batch.fill(polyline.clone(), |_| color);
        };

        if let Some(stroke) = scene.component::<Stroke>(id) {
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            batch.stroke(polyline, move |a| (color.sample(a), weight.sample(a)));
        };
    });
}

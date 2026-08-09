//! Helper methods to render splines

use crate::render::{Batch, MeshVertex, Polyline, RenderCommand};
use crate::scene::{Arrow, EntityId, Fill, Scene, Spline, Stroke, Transform, Visibility};
use glam::{Mat2, Vec2};

pub fn splines(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_pair::<Transform, Spline>();
    items.into_iter().for_each(|(id, transform, spline)| {
        // Visibility short circuit.
        let visibility = scene.component::<Visibility>(id);
        let vis_amount = visibility.map_or(1.0, |v| v.amount.sample(time).clamp(0.0, 1.0));
        let z_index = visibility.map_or(0.0, |v| v.z_index.sample(time));
        if vis_amount <= 0.005 {
            return;
        }

        // Check if we have a fill or stroke.
        let fill = scene.component::<Fill>(id);
        let stroke = scene.component::<Stroke>(id);
        if fill.is_none() && stroke.is_none() {
            return;
        }

        // Apply the curves transformation before subdivision.
        let affine = transform.sample(time);
        let spline_path = spline
            .spline_path
            .sample(time)
            .map(move |x| affine.matrix2 * x + affine.translation);

        // Subdivide the curve into a polyline of at least three points.
        let start = spline.start_alpha.sample(time);
        let end = spline.end_alpha.sample(time);
        let delta = end - start;
        let polyline = Polyline::flatten(
            &spline_path,
            start,
            start + delta * vis_amount,
            batch.tolerance(),
        );

        handle_polyline(batch, id, polyline, vis_amount, z_index, fill, stroke, time);
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
            let visibility = scene.component::<Visibility>(id);
            let vis_amount = visibility.map_or(1.0, |v| v.amount.sample(time).clamp(0.0, 1.0));
            let z_index = visibility.map_or(0.0, |v| v.z_index.sample(time));
            if vis_amount <= 0.005 {
                return;
            }

            // Sample the arrow
            let affine = transform.sample(time);
            let spline = &arrow.spline;
            let start = spline.start_alpha.sample(time);
            let end = spline.end_alpha.sample(time) * vis_amount;
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            let spline_path = spline
                .spline_path
                .sample(time)
                .map(move |x| affine.matrix2 * x + affine.translation);

            // Subdivide the curve into a polyline of at least three points.
            let delta = end - start;
            let new_end = start + delta * vis_amount;
            let polyline = Polyline::flatten(&spline_path, start, new_end, batch.tolerance());

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
            batch.add_command(
                RenderCommand::Polygon {
                    vertices: VERTICES.into_iter().map(map).collect(),
                },
                id,
                z_index,
            );

            // Render our polyline.
            handle_polyline(
                batch,
                id,
                polyline,
                vis_amount,
                z_index,
                None,
                Some(stroke),
                time,
            );
        });
}

/// Render fill and stroke appropriately for spline.
fn handle_polyline(
    batch: &mut Batch,
    id: EntityId,
    polyline: Polyline,
    visibility: f32,
    z_index: f32,
    fill: Option<&Fill>,
    stroke: Option<&Stroke>,
    time: f32,
) {
    match (fill, stroke) {
        // We have both and need to clone the polyline.
        (Some(fill), Some(stroke)) => {
            let mut color = fill.color.sample(time);
            color.w *= visibility * visibility * visibility;
            batch.add_command(polyline.clone().to_fill(|_| color), id, z_index);

            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            batch.add_command(
                polyline.to_stroke(move |a| (color.sample(a), weight.sample(a)), false),
                id,
                z_index,
            );
        }

        // We just have fill
        (Some(fill), None) => {
            let mut color = fill.color.sample(time);
            color.w *= visibility * visibility * visibility;
            batch.add_command(polyline.to_fill(|_| color), id, 0.0);
        }

        // We just have stroke.
        (None, Some(stroke)) => {
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            batch.add_command(
                polyline.to_stroke(move |a| (color.sample(a), weight.sample(a)), false),
                id,
                z_index,
            );
        }

        // We don't have either.
        (None, None) => (),
    };
}

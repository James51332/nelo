//! Helper methods to render splines

use crate::render::{Batch, MeshVertex, Polyline, RenderCommand};
use crate::scene::{Arrow, Fill, Scene, Spline, Stroke, Transform, Visibility};
use glam::{Mat2, Vec2};

// ----- RenderCommand -----

impl RenderCommand {
    /// Converts a spline and corresponding data into render commands.
    pub fn spline(
        spline: &Spline,
        transform: &Transform,
        tolerance: f32,
        time: f32,
        fill: Option<&Fill>,
        stroke: Option<&Stroke>,
        visibility: Option<&Visibility>,
    ) -> Vec<Self> {
        // Short circuit if we have no stroke or fill.
        if fill.is_none() && stroke.is_none() {
            return Vec::new();
        }

        // Get the visibility.
        let vis = visibility.map_or(1.0, |v| v.amount.sample(time).clamp(0.0, 1.0));
        if vis <= 0.005 {
            return Vec::new();
        }

        // Flatten into a polyline and return commands for that.
        let affine = transform.sample(time);
        let start = spline.start_alpha.sample(time);
        let end = spline.end_alpha.sample(time);
        let end = start + vis * (end - start);
        let path = spline
            .spline_path
            .sample(time)
            .map(move |v| affine.matrix2 * v + affine.translation);
        let polyline = Polyline::flatten(&path, start, end, tolerance);

        Self::polyline(polyline, spline.close.sample(time), time, fill, stroke)
    }

    /// Polyline can convert to command using `to_stroke` and `to_fill`, but this
    /// method doesn't clone the polyline unless required.
    pub fn polyline(
        polyline: Polyline,
        close: bool,
        time: f32,
        fill: Option<&Fill>,
        stroke: Option<&Stroke>,
    ) -> Vec<Self> {
        // Closures to consume the polyline into command.
        let fill_command = |cmds: &mut Vec<Self>, polyline: Polyline, fill: &Fill| {
            cmds.push(polyline.to_fill(|_| fill.color.sample(time)));
        };

        let stroke_command = move |cmds: &mut Vec<Self>, polyline: Polyline, stroke: &Stroke| {
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            let map = |a| (color.sample(a), weight.sample(a));
            cmds.push(polyline.to_stroke(map, close));
        };

        // Use these closures. Clone polyline only if we have both.
        let mut commands = Vec::new();
        match (fill, stroke) {
            (Some(fill), Some(stroke)) => {
                fill_command(&mut commands, polyline.clone(), fill);
                stroke_command(&mut commands, polyline, stroke);
            }
            (Some(fill), None) => fill_command(&mut commands, polyline, fill),
            (None, Some(stroke)) => stroke_command(&mut commands, polyline, stroke),
            _ => (),
        }

        commands
    }
}

// ----- Spline Render -----

pub(crate) fn splines(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_pair::<Transform, Spline>();
    items.into_iter().for_each(|(id, transform, spline)| {
        // Build the render command.
        let fill = scene.component(id);
        let stroke = scene.component(id);
        let visibility = scene.component(id);
        let commands = RenderCommand::spline(
            spline,
            transform,
            batch.tolerance(),
            time,
            fill,
            stroke,
            visibility,
        );

        // Submit them.
        let z_index = visibility.map_or(0.0, |v| v.z_index.sample(time));
        for command in commands.into_iter() {
            batch.add_command(command, id, z_index);
        }
    });
}

/// Mostly the same as the spline renderer. However, we never fill arrows, and we add
/// a triangle at the end.
pub(crate) fn arrows(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    let items = scene.view_triple::<Transform, Arrow, Stroke>();
    items
        .into_iter()
        .for_each(|(id, transform, arrow, stroke)| {
            // Build the render command for the spline.
            let fill = None; // Never fill in an arrow.
            let vis = scene.component(id);
            let spline = &arrow.spline;
            let commands = RenderCommand::spline(
                spline,
                transform,
                batch.tolerance(),
                time,
                fill,
                Some(stroke),
                vis,
            );

            // Submit them too.
            let z_index = vis.map_or(0.0, |v| v.z_index.sample(time));
            for command in commands.into_iter() {
                batch.add_command(command, id, z_index);
            }

            // Compute how we rotate and scale our triangle.
            let end = spline.end_alpha.sample(time);
            let spline_path = spline.spline_path.sample(time);

            // Compute the direction of our alphate the direction of our alpha.
            const DELTA_ALPHA: f32 = 0.001;
            let last = spline_path.sample(end);
            let prev = spline_path.sample(end - DELTA_ALPHA);
            let dir = (last - prev).normalize_or(Vec2::X);
            let rotate = Mat2::from_cols(dir, Vec2::new(-dir.y, dir.x));

            // Add the triangle geometry.
            const SCALE: f32 = 2.5;
            const VERTICES: [Vec2; 3] = [
                Vec2::new(1.0, 0.0),
                Vec2::new(-1.0, 1.0),
                Vec2::new(-1.0, -1.0),
            ];
            let size = SCALE * stroke.weight.sample(time).sample(end);
            let color = stroke.color.sample(time);
            let map = |v| MeshVertex::new(rotate * size * v + last, Vec2::ZERO, color.sample(end));
            let vertices = VERTICES.into_iter().map(map).collect();
            let command = RenderCommand::Polygon { vertices };
            batch.add_command(command, id, z_index);
        });
}

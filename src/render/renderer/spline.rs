//! Helper methods to render splines

use crate::render::{Encoder, MeshVertex, Polyline, RenderCommand, StrokeVertex};
use crate::scene::{Arrow, Fill, Scene, Spline, Stroke, Visibility};
use glam::{Affine2, Mat2, Vec2};

// ----- RenderCommand -----

impl RenderCommand {
    /// Converts a spline and corresponding data into render commands.
    pub fn spline(
        spline: &Spline,
        affine: Affine2,
        scale: f32,
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
        let start = spline.start_alpha.sample(time);
        let end = spline.end_alpha.sample(time);
        let end = start + vis * (end - start);
        let path = spline
            .spline_path
            .sample(time)
            .map(move |v| affine.matrix2 * v + affine.translation);
        let polyline = Polyline::flatten(&path, start, end, scale);

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
        let fill_command = |polyline: Polyline, fill: &Fill| {
            let color = fill.color.sample(time);
            let map = |(_, pos)| MeshVertex::new(pos, Vec2::ZERO, color);
            let vertices = polyline.into_iter().map(map).collect();
            RenderCommand::Polygon { vertices }
        };

        let stroke_command = move |polyline: Polyline, stroke: &Stroke| {
            let color = stroke.color.sample(time);
            let weight = stroke.weight.sample(time);
            let map = |(a, pos)| StrokeVertex::new(pos, color.sample(a), weight.sample(a));
            let vertices = polyline.into_iter().map(map).collect();
            RenderCommand::Stroke { vertices, close }
        };

        // Use these closures. Clone polyline only if we have both.
        let mut commands = Vec::new();
        match (fill, stroke) {
            (Some(fill), Some(stroke)) => {
                commands.push(fill_command(polyline.clone(), fill));
                commands.push(stroke_command(polyline, stroke));
            }
            (Some(fill), None) => commands.push(fill_command(polyline, fill)),
            (None, Some(stroke)) => commands.push(stroke_command(polyline, stroke)),
            _ => (),
        }

        commands
    }
}

// ----- Spline Render -----

pub(crate) fn splines(encoder: &mut Encoder, scene: &Scene, time: f32, _size: (u32, u32)) {
    let height = scene.sample_height(time);

    // Get a view of all curves with a stroke.
    let items = scene.view::<Spline>();
    items.into_iter().for_each(|(id, spline)| {
        // Build the render command.
        let fill = scene.component(id);
        let stroke = scene.component(id);
        let visibility = scene.component(id);
        let affine = scene.world_transform(id, time);
        let commands =
            RenderCommand::spline(spline, affine, height, time, fill, stroke, visibility);

        // Submit them.
        let z_index = visibility.map_or(0.0, |v| v.z_index.sample(time));
        for command in commands.into_iter() {
            encoder.add_command(id, command, z_index);
        }
    });
}

/// Mostly the same as the spline renderer. However, we never fill arrows, and we add
/// a triangle at the end.
pub(crate) fn arrows(encoder: &mut Encoder, scene: &Scene, time: f32, _size: (u32, u32)) {
    let height = scene.sample_height(time);
    let items = scene.view_pair::<Arrow, Stroke>();
    items.into_iter().for_each(|(id, arrow, stroke)| {
        // Build the render command for the spline.
        let fill = None; // Never fill in an arrow.
        let vis = scene.component(id);
        let spline = &arrow.spline;
        let affine = scene.world_transform(id, time);
        let commands = RenderCommand::spline(spline, affine, height, time, fill, Some(stroke), vis);

        // Submit them too.
        let z_index = vis.map_or(0.0, |v| v.z_index.sample(time));
        for command in commands.into_iter() {
            encoder.add_command(id, command, z_index);
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
        encoder.add_command(id, command, z_index);
    });
}

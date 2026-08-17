//! Tool for rendering glyphs.

use crate::render::{Batch, FillBuilder, MeshVertex, Polyline, RenderCommand};
use crate::scene::{Fill, Glyph, Scene, Stroke, Transform, Visibility};
use crate::timeline::{Easing, Timeline};
use glam::Vec2;

const STROKE_TIME: f32 = 0.8;
const STROKE_WEIGHT: f32 = 0.02;

/// Renders a set of contours.
///
/// Visibility behavior:
/// * Fill w/ Stroke:
///     1. Stroke first (0 -> 0.8)
///     2. Fill (0.8 -> 1.0)
/// * Fill w/o Stroke:
///     1. Stroke first (0 -> 0.8)
///     2a. Fill (0.8 -> 1.0)
///     2b. Stroke thins (0.8 -> 1.0)
/// * Stroke only:
///     1. Stroke write in (0 -> 1.0)
///
pub(crate) fn glyphs(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    let items = scene.view_pair::<Transform, Glyph>();
    items.into_iter().for_each(|(id, transform, glyph)| {
        // Visibility short circuit
        let visibility: Option<&Visibility> = scene.component(id);
        let vis_amount = visibility.map_or(1.0, |v| v.amount.sample(time));
        if vis_amount <= 0.005 {
            return;
        }

        // No fill and no stroke short circuit.
        let fill = scene.component::<Fill>(id);
        let mut stroke = scene.component(id);
        if fill.is_none() && stroke.is_none() {
            return;
        }

        // Artificial stroke for filled glyphs with partial visibility.
        let new_stroke;
        if stroke.is_none()
            && vis_amount < 0.999
            && let Some(fill) = fill.as_ref()
        {
            let weight = Timeline::keyframes(STROKE_WEIGHT * 0.5)
                .ease_at(STROKE_TIME, STROKE_WEIGHT, Easing::CubicIn)
                .ease_at(1.0, 0.0, Easing::CubicOut)
                .build()
                .sample(vis_amount);

            new_stroke = Some(Stroke {
                color: Timeline::constant(fill.color.clone().along()),
                weight: Timeline::constant(weight.into()),
            });

            stroke = new_stroke.as_ref();
        }

        let stroke_vis = if fill.is_none() {
            vis_amount
        } else {
            (vis_amount / STROKE_TIME).clamp(0.0, 1.0)
        };

        // Convert the splines into polylines.
        let mut polylines: Vec<(Polyline, bool)> = Vec::new();
        let affine = transform.sample(time);
        for spline in glyph.contours.iter() {
            let start = spline.start_alpha.sample(time);
            let end = spline.end_alpha.sample(time);
            let end = start + (end - start) * stroke_vis;
            let path = spline
                .spline_path
                .sample(time)
                .map(move |v| affine.matrix2 * v + affine.translation);
            let close = spline.close.sample(time);
            let polyline = Polyline::flatten(&path, start, end, batch.tolerance());
            polylines.push((polyline, close));
        }

        // Get the z-index.
        let z_index = visibility.map_or(0.0, |v| v.z_index.sample(time));

        // Handle filled glyphs.
        if let Some(fill) = fill {
            // Implement our fill scaling for partial visibility.
            let mut color = fill.color.sample(time);
            let alpha = Timeline::keyframes(0.0)
                .at(STROKE_TIME, 0.0)
                .at(1.0, 1.0)
                .build()
                .sample(vis_amount);
            color.alpha *= alpha;
            let map = |pos| MeshVertex::new(pos, Vec2::ZERO, color);

            // Build the mesh geometry.
            let mut builder = FillBuilder::new(0.001);
            for (polyline, close) in polylines.iter() {
                let mut points = polyline.points().iter();
                if let Some(pair) = points.next() {
                    builder.begin_subpath(map(pair.1));
                    points.for_each(|pair| builder.line_to(map(pair.1)));
                    builder.end_subpath(*close);
                }
            }

            // Submit the command.
            match builder.finish() {
                Ok(command) => batch.add_command(command, id, z_index),
                Err(e) => log::error!("Failed to tesselate glyph: {e}"),
            };
        }

        // Then submit each splines geometry if we have a stroke.
        if stroke.is_some() {
            for (polyline, close) in polylines.into_iter() {
                // Don't fill this since we've handle manually.
                let commands = RenderCommand::polyline(polyline, close, time, None, stroke);

                // Submit them.
                for command in commands.into_iter() {
                    batch.add_command(command, id, z_index);
                }
            }
        }
    });
}

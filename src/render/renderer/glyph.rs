//! Tool for rendering glyphs.

use crate::render::{Batch, FillBuilder, MeshVertex, Segment};
use crate::scene::{Fill, Glyph, Scene, Transform, Visibility};
use ab_glyph::{Font, OutlineCurve, Point};
use glam::prelude::*;

pub fn filled_glyphs(batch: &mut Batch, scene: &Scene, time: f32, _size: (u32, u32)) {
    // Font outlines are in design units; normalize to em space (~0..1) so a
    // glyph is a sane size relative to the rest of the scene.
    let font = scene.font();
    let scale = font.units_per_em().unwrap_or(1000.0);

    // Let's get a view of all glyphs to rasterize.
    let items = scene.view_triple::<Transform, Glyph, Fill>();
    items.into_iter().for_each(|(id, transform, glyph, fill)| {
        // Visibility
        let visibility = scene.component::<Visibility>(id);
        let vis_amount = visibility.map_or(1.0, |v| v.amount.sample(time));
        let z_index = visibility.map_or(0.0, |v| v.amount.sample(time));

        // Get the basic info about our glyph.
        let glyph_id = font.glyph_id(glyph.character);
        let mut transform = transform.sample(time);
        transform.matrix2 /= scale;
        let mut color = fill.color.sample(time);
        color.w = vis_amount * vis_amount * vis_amount;

        // Get the outline for our character. Skip spaces or other unsupported.
        let Some(outline) = font.outline(glyph_id) else {
            return;
        };

        // Run our tesselation. We are scaling to world space first, so scale is one.
        let mut builder = FillBuilder::new(batch.tolerance());
        outline
            .curves
            .into_iter()
            .for_each(|x| builder.add_segment(convert_outline(x, transform, color)));
        let result = builder.finish();

        // Print if we fail.
        match result {
            Ok(command) => batch.add_command(command, z_index),
            Err(e) => log::info!("Failed to triangulate glyph: {e}"),
        };
    });
}

/// Converts an OutlineCurve to a FillBuilder Segment.
fn convert_outline(outline: OutlineCurve, transform: Affine2, color: Vec4) -> Segment {
    let point = |p: Point| transform.matrix2 * Vec2::new(p.x, p.y) + transform.translation;
    let vertex = |p: Point| MeshVertex::new(point(p), Vec2::ZERO, color);
    match outline {
        OutlineCurve::Line(p1, p2) => Segment::Line(vertex(p1), vertex(p2)),
        OutlineCurve::Quad(p1, p2, p3) => Segment::Quad(vertex(p1), point(p2), vertex(p3)),
        OutlineCurve::Cubic(p1, p2, p3, p4) => {
            Segment::Cubic(vertex(p1), point(p2), point(p3), vertex(p4))
        }
    }
}

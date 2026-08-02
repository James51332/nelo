//! Tool for rendering glyphs.

use crate::render::{BatchSet, ComponentRenderer, ModelVertex};
use crate::scene::{Fill, Glyph, Scene, Transform};
use ab_glyph::{Font, FontArc, OutlineCurve};
use glam::prelude::*;
use lyon::tessellation::{
    FillOptions, FillTessellator, VertexBuffers, geometry_builder::simple_builder, math::Point,
};

/// Converts ab_glyph::Point to lyon::math::Point
fn convert_point(point: ab_glyph::Point) -> Point {
    Point::new(point.x, point.y)
}

/// The start point of an outline segment, regardless of its kind.
fn segment_start(curve: &OutlineCurve) -> ab_glyph::Point {
    match *curve {
        OutlineCurve::Line(start, _)
        | OutlineCurve::Quad(start, _, _)
        | OutlineCurve::Cubic(start, _, _, _) => start,
    }
}

pub fn get_filled_renderer(font: FontArc) -> ComponentRenderer {
    // Font outlines are in design units; normalize to em space (~0..1) so a
    // glyph is a sane size relative to the rest of the scene.
    let scale = 1.0 / font.units_per_em().unwrap_or(1000.0);

    let renderer = move |batches: &mut BatchSet, scene: &Scene, time: f32, _size: (u32, u32)| {
        // Let's get a view of all glyphs to rasterize.
        let items = scene.view_triple::<Transform, Glyph, Fill>();

        items.iter().for_each(|(_, transform, glyph, fill)| {
            // Get the basic info about our glyph.
            let glyph_id = font.glyph_id(glyph.character);

            // Convert our glyph into an outline to triangulate.
            let Some(outline) = font.outline(glyph_id) else {
                log::info!("Skipping unsupported character: {}", glyph.character);
                return;
            };

            // Use lyon to fill our glyph.
            let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();
            let mut vertex_builder = simple_builder(&mut buffers);
            let mut tesselator = FillTessellator::new();
            let options = FillOptions::default();
            let mut builder = tesselator.builder(&options, &mut vertex_builder);

            // A glyph outline is a flat list of curves that may span several
            // contours (e.g. the counter of an 'o', or the dot of a '!').
            // ab_glyph gives us no explicit contour markers, so a new contour
            // begins whenever a segment's start no longer matches the previous
            // segment's end. Each contour is its own closed subpath.
            let mut open = false;
            let mut last_end: Option<ab_glyph::Point> = None;

            for segment in outline.curves.iter() {
                let start = segment_start(segment);

                // Start a fresh contour if this segment doesn't continue the last.
                let continues = last_end.is_some_and(|end| end == start);
                if !continues {
                    if open {
                        builder.end(true);
                    }
                    builder.begin(convert_point(start));
                    open = true;
                }

                // Emit the segment, preserving curvature.
                let end = match *segment {
                    OutlineCurve::Line(_, end) => {
                        builder.line_to(convert_point(end));
                        end
                    }
                    OutlineCurve::Quad(_, ctrl, end) => {
                        builder.quadratic_bezier_to(convert_point(ctrl), convert_point(end));
                        end
                    }
                    OutlineCurve::Cubic(_, c1, c2, end) => {
                        builder.cubic_bezier_to(
                            convert_point(c1),
                            convert_point(c2),
                            convert_point(end),
                        );
                        end
                    }
                };
                last_end = Some(end);
            }

            // Close the final contour.
            if open {
                builder.end(true);
            }

            let result = builder.build();
            if let Err(msg) = result {
                log::error!("Tesselation failed: {}", msg);
                return;
            }

            // Submit our triangle data to the GPU.
            let transform = transform.sample(time);
            let color = fill.color.sample(time);
            let vertices: Vec<_> = buffers
                .vertices
                .into_iter()
                .map(|point| {
                    let converted = Vec2::new(point.x, point.y) * scale;
                    let position = transform.matrix2 * converted + transform.translation;
                    ModelVertex::new(position, Vec2::ZERO, color)
                })
                .collect();

            let indices: Vec<_> = buffers
                .indices
                .into_iter()
                .map(|idx: u16| idx as u32)
                .collect();

            batches.models.push(&vertices, &indices);
        });
    };

    Box::new(renderer)
}

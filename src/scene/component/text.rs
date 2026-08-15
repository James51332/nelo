//! Text is a group of glyphs.

use crate::{
    scene::{Fill, Glyph, GroupRef, Scene, Spline, Transformable},
    timeline::{Path, PathBuilder},
};
use ab_glyph::{Font, OutlineCurve, Point};
use glam::prelude::*;
use glyph_brush_layout::{
    BuiltInLineBreaker, GlyphPositioner, HorizontalAlign, Layout, SectionGeometry, SectionText,
    VerticalAlign,
};

impl Scene {
    /// Create a group of characters from a string slice.
    pub fn text(&mut self, text: &str) -> GroupRef<'_> {
        // Create the arrangement before borrowing scene.
        let section = SectionText {
            text,
            ..SectionText::default()
        };
        let layout = Layout::SingleLine {
            line_breaker: BuiltInLineBreaker::default(),
            h_align: HorizontalAlign::Center,
            v_align: VerticalAlign::Bottom, // Note vertical align isn't currently supported.
        };
        let geometry = SectionGeometry::default();
        let arrangement = layout.calculate_glyphs(&[self.default_font()], &geometry, &[section]);

        // Get the scaling values.
        let font = crate::scene::Font::default();
        let font_ref = self.font(font);
        let k = font_ref.height_unscaled() / font_ref.units_per_em().unwrap_or(1000.0);

        // Create the text glyphs.
        let mut group = self.group();
        for sg in arrangement.iter() {
            let character = text[sg.byte_index..].chars().next().unwrap();
            if character == ' ' {
                continue;
            }

            let offset = Vec2::new(
                sg.glyph.position.x / sg.glyph.scale.x * k,
                -sg.glyph.position.y / sg.glyph.scale.y * k,
            );

            group = group.create_once(move |s| {
                let letter = letter(s, character, font);
                s.create()
                    .attach(letter)
                    .attach(Fill::solid())
                    .translate(offset)
            });
        }
        group
    }
}

pub fn letter(scene: &Scene, character: char, font: crate::scene::Font) -> Glyph {
    // Basic glyph information
    let font = scene.font(font);
    let glyph_id = font.glyph_id(character);
    let scale = 1.0 / font.units_per_em().unwrap_or(1000.0);

    // Get the outline for the curve.
    let Some(outline) = font.outline(glyph_id) else {
        log::warn!("Failed to get outline for {character}!");
        return Glyph {
            contours: Vec::new(),
        };
    };

    // Keep track of the contours in the outline.
    let mut contours = Vec::new();

    // Add the points to the contour.
    let mut builder: Option<PathBuilder> = None;
    let mut last_point: Option<Vec2> = None;
    let map = |p: Point| Vec2::new(p.x, p.y) * scale;
    let flush = |builder: Option<PathBuilder>, contours: &mut Vec<Spline>| {
        if let Some(builder) = builder {
            contours.push(Spline {
                spline_path: builder.build().into(),
                start_alpha: 0.0.into(),
                end_alpha: 1.0.into(),
                close: false.into(),
            });
        }
    };

    for segment in outline.curves.into_iter() {
        // Get the first point on this segment.
        let first_point = start(&segment) * scale;

        // Start a new contour when this segment doesn't continue the last one.
        if last_point != Some(first_point) {
            flush(builder.take(), &mut contours);
            builder = Some(Path::bezier(first_point));
        }

        // Add the segment to the builder, tracking where it ends.
        if let Some(builder) = builder.as_mut() {
            last_point = Some(match segment {
                OutlineCurve::Line(_, end) => {
                    builder.line_to(map(end));
                    map(end)
                }
                OutlineCurve::Quad(_, c0, end) => {
                    builder.quad_to(map(c0), map(end));
                    map(end)
                }
                OutlineCurve::Cubic(_, c0, c1, end) => {
                    builder.cubic_to(map(c0), map(c1), map(end));
                    map(end)
                }
            });
        }
    }

    // Don't drop the final contour.
    flush(builder.take(), &mut contours);

    Glyph { contours }
}

fn start(outline: &OutlineCurve) -> Vec2 {
    let vec = |p: &Point| Vec2::new(p.x, p.y);
    match outline {
        OutlineCurve::Line(start, _) => vec(start),
        OutlineCurve::Quad(start, _, _) => vec(start),
        OutlineCurve::Cubic(start, _, _, _) => vec(start),
    }
}

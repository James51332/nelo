//! Text is a group of glyphs.

use crate::scene::{Fill, GroupRef, Scene, Transformable};
use glam::prelude::*;
use glyph_brush_layout::{
    BuiltInLineBreaker, GlyphPositioner, HorizontalAlign, Layout, SectionGeometry, SectionText,
    VerticalAlign,
};

// ----- Glyph -----

const PADDING: f32 = 0.12;

pub struct Glyph {
    pub character: char,
}

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
        let arrangement = layout.calculate_glyphs(&[self.font()], &geometry, &[section]);

        // Create the text glyphs.
        let mut group = self.group();
        let num_chars = arrangement.len();
        for (i, sg) in arrangement.iter().enumerate() {
            let character = text[sg.byte_index..].chars().next().unwrap();
            let padding_offset = (i as f32 - (num_chars - 1) as f32 / 2.0) * PADDING;
            let offset = Vec2::new(
                sg.glyph.position.x / sg.glyph.scale.x + padding_offset,
                sg.glyph.position.y / sg.glyph.scale.y,
            );
            group = group.create_once(move |s| {
                s.create()
                    .attach(Glyph { character })
                    .attach(Fill::default())
                    .translate(offset)
            });
        }
        group
    }
}

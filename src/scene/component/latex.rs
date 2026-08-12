//! Render latex into splines.

use crate::{
    scene::{Fill, Glyph, GroupRef, Scene, Transformable},
    timeline::Path,
};
use glam::Vec2;
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parse;
use ratex_types::DisplayItem;

impl Scene {
    pub fn latex(&mut self, tex: &str) -> GroupRef<'_> {
        let mut group = self.group();

        // Parse our latex or return an empty group.
        let nodes = match parse(tex) {
            Ok(nodes) => nodes,
            Err(e) => {
                log::error!("Failed to parse latex: {e}");
                return group;
            }
        };

        // Layout the resulting nodes.
        let options = LayoutOptions::default();
        let layout = layout(&nodes, &options);
        let display_list = to_display_list(&layout);

        // Compute the width and height of the text.
        let width = display_list.width as f32;
        let height = display_list.height as f32 + display_list.depth as f32;
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        // Convert the nodes into entities.
        for item in display_list.items.into_iter() {
            match item {
                DisplayItem::GlyphPath {
                    x,
                    y,
                    scale,
                    font: _font,
                    char_code,
                    color: _color,
                } => {
                    let x = x as f32 - half_width;
                    let y = half_height - y as f32;
                    if let Some(character) = char::from_u32(char_code) {
                        group = group.create_once(|s| {
                            s.create()
                                .attach(Fill::solid())
                                .attach(Glyph { character })
                                .scale(scale as f32)
                                .translate(Vec2::new(x as f32, y))
                        });
                    }
                }
                DisplayItem::Line {
                    x,
                    y,
                    width,
                    thickness,
                    color: _color,
                    dashed: _dashed,
                } => {
                    let x = x as f32 - half_width;
                    let y = half_height - y as f32;
                    let thickness = thickness as f32;
                    let start = Vec2::new(x, y);
                    let end = Vec2::new(x + width as f32, y);
                    let line = Path::line(start, end);
                    group = group.create_once(|s| s.spline(line).stroke_weight(thickness));
                }
                _ => (),
            }
        }

        group
    }
}

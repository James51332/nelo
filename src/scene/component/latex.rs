//! Render latex into splines.

use crate::{
    scene::{Fill, Font, Glyph, GroupRef, Scene, Spline, Transformable, component::text::letter},
    timeline::{Path, PathBuilder},
};
use glam::{Mat2, Vec2};
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parse;
use ratex_types::{DisplayItem, PathCommand};

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
                    font,
                    char_code,
                    color: _color,
                } => {
                    let x = x as f32 - half_width;
                    let y = half_height - y as f32;
                    if let Some(character) = char::from_u32(char_code) {
                        let font = Font::try_from(font).unwrap_or_default();
                        group = group.create_once(|s| {
                            let letter = letter(s, character, font);
                            s.create()
                                .attach(Fill::solid())
                                .attach(letter)
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
                DisplayItem::Rect {
                    x,
                    y,
                    width,
                    height,
                    color: _color,
                } => {
                    // Translate our x and y position into world space.
                    let x = x as f32 - half_width;
                    let y = half_height - y as f32;

                    // Compute the center coordinate (given bottom left)
                    let half_width = width as f32 / 2.0;
                    let half_height = height as f32 / 2.0;
                    let x = x + half_width;
                    let y = y + half_height;

                    // Scale our square and apply the transformation.
                    group = group.create_once(|s| {
                        s.square()
                            .matrix(Mat2::from_cols(Vec2::X * half_width, Vec2::Y * half_height))
                            .translate(Vec2::new(x, y))
                    });
                }
                DisplayItem::Path {
                    x,
                    y,
                    commands,
                    fill: _fill,
                    color: _color,
                } => {
                    let map = |px: f64, py: f64| {
                        Vec2::new(px as f32 - half_width, half_height - py as f32)
                    };
                    let mut builder = None;
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

                    let mut contours = Vec::new();
                    for command in commands.into_iter() {
                        match command {
                            PathCommand::MoveTo { x, y } => {
                                flush(builder.take(), &mut contours);
                                builder = Some(Path::bezier(map(x, y)));
                            }
                            PathCommand::LineTo { x, y } => {
                                if let Some(builder) = builder.as_mut() {
                                    builder.line_to(map(x, y));
                                }
                            }
                            PathCommand::QuadTo { x1, y1, x, y } => {
                                if let Some(builder) = builder.as_mut() {
                                    builder.quad_to(map(x1, y1), map(x, y));
                                }
                            }
                            PathCommand::CubicTo {
                                x1,
                                y1,
                                x2,
                                y2,
                                x,
                                y,
                            } => {
                                if let Some(builder) = builder.as_mut() {
                                    builder.cubic_to(map(x1, y1), map(x2, y2), map(x, y));
                                }
                            }
                            PathCommand::Close => flush(builder.take(), &mut contours),
                        }
                    }

                    // Flush the last open contour.
                    flush(builder.take(), &mut contours);

                    group = group.create_once(|s| {
                        s.create()
                            .attach(Glyph { contours })
                            .attach(Fill::solid())
                            .translate(Vec2::new(x as f32, y as f32))
                    });
                }
            }
        }

        group
    }
}

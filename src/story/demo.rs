//! A small demo story

use crate::{
    scene::{Color, Scene, Transformable},
    story::Story,
};
use glam::Vec2;

impl Story {
    pub fn demo() -> Story {
        let mut scene = Scene::new();
        scene.camera().height = 15.0.into();

        let axes = scene.axes().id();
        let plot = scene
            .plot(|x: f32| 0.05 * x * x * x - 0.1 * x * x - 0.9 * x, -9.0, 9.0)
            .stroke(Color::srgb(0.8, 0.0, 0.0))
            .stroke_weight(0.06)
            .id();

        let latex = scene
            .latex(r"\begin{align*} f(x) &= (x + 3)^2 \\ &= x^2 + 6x + 9 \nonumber \end{align*}")
            .split_after('=')
            .children();

        let text = scene
            .text("Binomial Expansion")
            .translate(Vec2::Y * 3.0)
            .id();

        let top = scene.top();
        let mut donuts = scene.group();
        donuts
            .create(6, |i, s| {
                s.circle()
                    .scale(0.6)
                    .no_fill()
                    .stroke_weight(0.75)
                    .stroke(Color::lch(0.4, 0.3, i as f32 / 6.0 * 360.0))
            })
            .row(2.2);
        let group = donuts.id();
        let donuts = donuts.children();

        let mut story = scene.story();

        // Act 1. Show some donuts
        donuts.iter().for_each(|&id| {
            story.show(id).wait(0.1);
        });

        story.translate(top, group);

        // Act 2. Do some plotting.
        story.wait(0.5).hide_all();
        story.show(axes);
        story.show(plot);

        // Act 3. Show an equation
        story.wait(1.0).hide_all().wait(0.25);
        story.show(text);
        for &id in latex.iter() {
            story.show(id).wait(1.0);
        }

        story
    }
}

//! A small demo scene

use glam::Vec2;

use crate::{
    scene::{Color, Scene, Transformable},
    timeline::{Path, Timeline},
};

impl Scene {
    /// Returns a small demo scene.
    pub fn demo() -> Scene {
        let mut scene = Scene::new();

        // Set the background color.
        scene.camera().background(Color::srgb(0.4, 0.3, 0.5));

        // Timeline to sample to repeat animations.
        let repeat = Timeline::triangle(6.0).ease();

        // Some circles which go back and forth from spiral to a line.
        let line = Path::line(Vec2::X * 2.0, Vec2::X * 4.0);
        scene
            .group()
            .create(12, |_, s| s.dot().scale(0.08))
            .arrange(line)
            .for_each(|i, e| e.rotate(repeat.clone().add(0.2).multiply(i as f32)));

        // Create some elements in a group.
        scene
            .group()
            .create_once(|s| s.triangle())
            .create_once(|s| s.square())
            .create_once(|s| s.circle())
            .for_each(|_, e| e.scale(0.5))
            .row(1.25);

        // Render some text.
        scene.text("Hello, Nelo!").translate(Vec2::Y * 4.0);

        // Wavy path.
        scene.spline_with_range(
            |t: f32, x: f32| Vec2::new(x, -4.0 - 0.6 * (x - 4.0 * t).sin()),
            -10.0,
            10.0,
        );

        scene
    }
}

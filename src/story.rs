//! Sequence tool for actions on entities in a scene

pub mod action;
pub mod show;

pub use action::{Action, Stage};
use glam::Vec2;
use palette::num::Clamp;
pub use show::{Hide, Show};

use crate::{
    render::{Color, Playback},
    scene::{EntityId, Scene, Transformable},
    timeline::Easing,
};

// ----- Story -----

/// A story is an optional utility to sequence animations within a scene.
/// Create it from a Scene using the `.story()` function.
pub struct Story {
    scene: Scene,
    cursor: f32,
}

impl Story {
    pub fn wait(&mut self, time: f32) -> &mut Self {
        self.cursor += time;
        self
    }

    pub fn action(&mut self, action: impl Action, ids: &[EntityId]) -> &mut Self {
        let stage = Stage::new(&mut self.scene, self.cursor);
        self.cursor += action.apply(stage, ids).unwrap_or(0.0).clamp_min(0.0);
        self
    }

    pub fn show(&mut self, id: EntityId) -> &mut Self {
        self.action(Show::default(), &[id])
    }

    pub fn hide(&mut self, id: EntityId) -> &mut Self {
        self.action(Hide::default(), &[id])
    }

    /// Hides all the entities in the scene.
    pub fn hide_all(&mut self) -> &mut Self {
        self.action(Hide::default(), &self.scene.entities())
    }

    pub fn clear(&mut self) -> &mut Self {
        let clear = Hide {
            step: 0.0,
            group_step: 0.0,
            duration: 0.0,
            easing: Easing::Step,
        };
        self.action(clear, &self.scene.entities())
    }

    pub fn demo() -> Story {
        let mut scene = Scene::new();

        let axes = scene.axes().id();
        let plot = scene
            .plot(|x: f32| 0.04 * x * x - 0.2 * x + 0.5, -7.0, 7.0)
            .stroke(Color::srgb(1.0, 0.0, 0.0))
            .stroke_weight(0.04)
            .id();
        let latex = scene.latex(r"\sum_{i=0}^n = \frac{n(n+1)}{2}").id();
        let text = scene
            .text("Sum of consecutive integers")
            .translate(Vec2::Y * 3.0)
            .id();

        let donuts = scene
            .group()
            .create(6, |i, s| {
                s.circle()
                    .scale(0.6)
                    .no_fill()
                    .stroke_weight(0.75)
                    .stroke(Color::lch(0.4, 0.3, i as f32 / 6.0 * 360.0))
            })
            .row(2.2)
            .id();

        let mut story = scene.story();

        // Act 1. Show some donuts
        story.show(donuts);

        // Act 2. Do some plotting.
        story.wait(2.5);
        story.hide(donuts);
        story.show(axes);
        story.show(plot);

        // Act 3. Show an equation
        story.wait(1.0).hide_all().wait(1.0).show(latex).show(text);

        story
    }
}

impl Scene {
    /// Creates a new story using this scene. Clears all entities by default.
    pub fn story(self) -> Story {
        let mut story = Story {
            scene: self,
            cursor: 0.0,
        };

        story.clear();
        story
    }
}

impl Into<Playback> for Story {
    fn into(self) -> Playback {
        Playback::new(self.scene).with_length(self.cursor + 1.0)
    }
}

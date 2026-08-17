//! Sequence tool for actions on entities in a scene

pub mod action;
pub mod show;

pub use action::{Action, Stage};
use glam::Vec2;
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
    prev_cursor: Option<f32>,
}

impl Story {
    pub fn wait(&mut self, time: f32) -> &mut Self {
        self.prev_cursor = Some(self.cursor);

        // Disallow negative waits.
        self.cursor += time.max(0.0);
        self
    }

    pub fn apply(&mut self, action: impl Action, ids: &[EntityId]) -> &mut Self {
        // Set the stage, so to speak.
        let stage = Stage::new(&mut self.scene, self.cursor);

        // Wait even if the action doesn't have a length so together knows where to go.
        let length = action.apply(stage, ids).unwrap_or(0.0);
        self.wait(length);

        self
    }

    // Moves the cursor to the last position to enable simultaneous animations.
    pub fn together(&mut self) -> &mut Self {
        if let Some(prev_cursor) = self.prev_cursor {
            self.cursor = prev_cursor;
            self.prev_cursor = None;
        }

        self
    }

    // Start the next animation with an offset from the previous.
    pub fn delay(&mut self, delay: f32) -> &mut Self {
        self.together().wait(delay)
    }

    pub fn show(&mut self, id: EntityId) -> &mut Self {
        self.show_slice(&[id])
    }

    pub fn show_all(&mut self) -> &mut Self {
        self.show_slice(&self.scene.entities())
    }

    pub fn show_slice(&mut self, ids: &[EntityId]) -> &mut Self {
        self.apply(Show::default(), ids)
    }

    pub fn hide(&mut self, id: EntityId) -> &mut Self {
        self.hide_slice(&[id])
    }

    pub fn hide_all(&mut self) -> &mut Self {
        self.hide_slice(&self.scene.entities())
    }

    pub fn hide_slice(&mut self, ids: &[EntityId]) -> &mut Self {
        self.apply(Hide::default(), ids)
    }

    pub fn clear(&mut self) -> &mut Self {
        let clear = Hide {
            step: 0.0,
            group_step: 0.0,
            duration: 0.0,
            easing: Easing::Step,
        };
        self.apply(clear, &self.scene.entities())
    }

    pub fn demo() -> Story {
        let mut scene = Scene::new();

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
            .children();

        let mut story = scene.story();

        // Act 1. Show some donuts
        donuts.iter().for_each(|&id| {
            story.show(id).wait(0.4);
        });

        // Act 2. Do some plotting.
        story.hide_all();
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

impl Scene {
    /// Creates a new story using this scene. Clears all entities by default.
    pub fn story(self) -> Story {
        let mut story = Story {
            scene: self,
            cursor: 0.0,
            prev_cursor: None,
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

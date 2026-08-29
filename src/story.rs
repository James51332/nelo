//! Sequence tool for actions on entities in a scene

pub mod action;
pub mod demo;
pub mod show;
pub mod transform;

pub use action::{Action, Stage};
pub use show::{Hide, Show};

use crate::{
    scene::{EntityId, Playback, Scene},
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

    pub fn clear(&mut self) -> &mut Self {
        let clear = Hide {
            step: 0.0,
            group_step: 0.0,
            duration: 0.0,
            easing: Easing::Step,
        };
        self.apply(clear, &self.scene.entities())
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

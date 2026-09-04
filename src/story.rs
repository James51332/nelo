//! Sequence tool for actions on entities in a scene

pub mod action;
pub mod color;
pub mod demo;
pub mod show;
pub mod transform;

pub use action::{Action, Stage, Target, Timing};
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

    pub fn apply_with_timing<T: Action>(
        &mut self,
        action: &T,
        ids: &[EntityId],
        timing: &Timing,
    ) -> &mut Self {
        // Determine how we are applying. Specified target takes precedent, but usually None.
        let target = timing.target.unwrap_or(T::target());
        let initial_time = self.cursor;

        // Iterate over the given entity list.
        for (i, &id) in ids.iter().enumerate() {
            let has_children = self.scene.hierarchy().has_children(id);

            match target {
                // If we are group and asked to split, then apply recursively to children.
                Target::Leaves if has_children => {
                    // Applying with timing will add the steps and the duration of last.
                    self.apply_with_timing(action, &self.scene.hierarchy().children(id), timing);

                    // Remove the duration of the last.
                    self.cursor -= timing.duration;

                    // If we have another id, add step.
                    if i + 1 < ids.len() {
                        self.cursor += timing.group_step;
                    }
                }

                // Otherwise, just apply to this entity and move the cursor.
                _ => {
                    let duration = timing.duration;
                    let easing = timing.easing;
                    let stage = Stage::new(&mut self.scene, self.cursor, duration, easing);
                    action.apply(id, stage);

                    if i + 1 < ids.len() {
                        self.cursor += timing.step;
                    }
                }
            }
        }

        // Add the duration of the last.
        if !ids.is_empty() {
            self.cursor += timing.duration;
        }

        // Set the last cursor at the end (overwrite any recursive calls).
        self.prev_cursor = Some(initial_time);

        self
    }

    /// Apply an action with the default timing.
    pub fn apply(&mut self, action: impl Action, ids: &[EntityId]) -> &mut Self {
        self.apply_with_timing(&action, ids, &Timing::default())
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
        let timing = Timing {
            step: 0.0,
            group_step: 0.0,
            duration: 0.0,
            easing: Easing::Step,
            target: Some(Target::Leaves),
        };
        self.apply_with_timing(&Hide, &self.scene.entities(), &timing)
    }
}

impl Scene {
    /// Creates a new story using this scene.
    pub fn story(self) -> Story {
        Story {
            scene: self,
            cursor: 0.0,
            prev_cursor: None,
        }
    }
}

impl Into<Playback> for Story {
    fn into(self) -> Playback {
        Playback::new(self.scene).with_length(self.cursor + 1.0)
    }
}

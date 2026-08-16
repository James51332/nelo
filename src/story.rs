//! Sequence tool for actions on entities in a scene

pub mod action;
pub mod show;

pub use action::{Action, Stage};
pub use show::Show;

use crate::{
    render::Playback,
    scene::{EntityId, Scene},
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

    pub fn action(&mut self, action: impl Action, id: EntityId) -> &mut Self {
        let stage = Stage::new(&mut self.scene, self.cursor);
        action.apply(stage, &[id]);
        self
    }

    pub fn show(&mut self, id: EntityId) -> &mut Self {
        let action = Show::default();
        self.action(action, id)
    }
}

impl Scene {
    /// Creates a new story using this scene.
    pub fn story(self) -> Story {
        Story {
            scene: self,
            cursor: 0.0,
        }
    }
}

impl Into<Playback> for Story {
    fn into(self) -> Playback {
        Playback::new(self.scene)
    }
}

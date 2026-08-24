//! An action is a predefined animation which can be applied to a scene.

use crate::scene::{EntityId, Scene};

// ----- Action -----

/// An action is a predefined animation that can be applied to an entity at
/// a specific time. Actions may create new entities, or otherwise modify the
/// scene as needed to implement their desired behavior. Modifying an entity
/// after applying an action is undefined behavior.
pub trait Action {
    fn apply(&self, stage: Stage<'_>, ids: &[EntityId]) -> Option<f32>;
}

// ----- Stage -----

/// A stage is a wrapper around a scene with a cursor time.
pub struct Stage<'a> {
    scene: &'a mut Scene,
    cursor: f32,
}

impl<'a> Stage<'a> {
    pub fn new(scene: &'a mut Scene, cursor: f32) -> Self {
        Self { scene, cursor }
    }

    pub fn scene(&mut self) -> &mut Scene {
        self.scene
    }

    pub fn cursor(&self) -> f32 {
        self.cursor
    }
}

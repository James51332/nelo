//! An action is a predefined animation which can be applied to a scene.

use crate::{
    scene::{EntityId, Scene},
    timeline::Easing,
};

// ----- Action -----

#[derive(Debug, Clone, Copy, Default)]
pub enum Target {
    #[default]
    Roots,
    Leaves,
}

/// An action is a predefined animation that can be applied to an entity at
/// a specific time. Actions may create new entities, or otherwise modify the
/// scene as needed to implement their desired behavior.
pub trait Action {
    /// Apply the action to a specific entity. This should not split groups, as this should be
    /// handled by story, which can choose to respect or ignore the desired target.
    fn apply(&self, id: EntityId, stage: Stage<'_>);

    /// Determines how the action should be split. Can be overriden on application.
    fn target() -> Target {
        Target::Leaves
    }
}

// ----- Timing -----

/// Defines the timing for how an action is applied. This is not seen by actions, but instead
/// applied by the story.
pub struct Timing {
    pub step: f32,
    pub group_step: f32,
    pub duration: f32,
    pub easing: Easing,

    // Optional override for the target of the action.
    pub target: Option<Target>,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            step: 0.1,
            group_step: 0.9,
            duration: 0.9,
            easing: Easing::CubicInOut,
            target: None,
        }
    }
}

impl Timing {
    /// Everything animates at once. No stagger at either level, and groups are not split,
    /// so an already-flat entity list is applied exactly once per entity.
    pub fn parallel() -> Self {
        Self {
            step: 0.0,
            group_step: 0.0,
            duration: 0.9,
            easing: Easing::CubicInOut,
            target: Some(Target::Roots),
        }
    }
}

// ----- Stage -----

/// A stage is the context that an action need to apply an action.
pub struct Stage<'a> {
    scene: &'a mut Scene,
    cursor: f32,
    duration: f32,
    easing: Easing,
}

impl<'a> Stage<'a> {
    pub fn new(scene: &'a mut Scene, cursor: f32, duration: f32, easing: Easing) -> Self {
        Self {
            scene,
            cursor,
            duration,
            easing,
        }
    }

    pub fn scene(&mut self) -> &mut Scene {
        self.scene
    }

    pub fn cursor(&self) -> f32 {
        self.cursor
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn easing(&self) -> Easing {
        self.easing
    }
}

//! Animations to display and hide entities.

use crate::scene::{EntityId, Scene, Visibility};
use crate::story::{Action, Stage};
use crate::timeline::{Easing, Timeline};

// ----- Show -----

pub struct Show {
    pub step: f32,
    pub rate: f32,
    pub easing: Easing,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            step: 0.04,
            rate: 0.95,
            easing: Easing::CubicInOut,
        }
    }
}

impl Show {
    fn show(&self, scene: &mut Scene, time: &mut f32, id: EntityId) {
        // Groups aren't rendered themselves, so they only stagger their children.
        let children = scene
            .get(id)
            .and_then(|e| e.as_group())
            .map(|g| g.children());

        if let Some(children) = children {
            for id in children.into_iter() {
                self.show(scene, time, id);
            }
            return;
        }

        // Get the entity or return if there is not one.
        let Some(mut entity) = scene.get(id) else {
            return;
        };

        // Show this entity. Entities without a `Visibility` are fully visible by
        // default, so one is attached here rather than skipping the entity.
        let visibility = entity.get_or_default::<Visibility>();
        let multiplier = Timeline::ramp()
            .shift(*time)
            .multiply(self.rate)
            .clamp(0.0, 1.0)
            .then(self.easing.clone());

        visibility.amount = visibility.amount.clone().multiply(multiplier);
        *time += self.step;
    }
}

impl Action for Show {
    fn apply(&self, mut stage: Stage, ids: &[EntityId]) {
        let mut cursor = stage.cursor();
        let scene = stage.scene_mut();
        for &id in ids.iter() {
            self.show(scene, &mut cursor, id);
        }
    }
}

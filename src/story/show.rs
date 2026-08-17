//! Animations to display and hide entities.

use crate::scene::{EntityId, Fill, Scene, Stroke, Visibility};
use crate::story::{Action, Stage};
use crate::timeline::{Easing, Timeline};

// ----- Show -----

pub struct Show {
    pub step: f32,
    pub group_step: f32,
    pub duration: f32,
    pub easing: Easing,
}

impl Default for Show {
    fn default() -> Self {
        Self {
            step: 0.1,
            group_step: 0.9,
            duration: 0.9,
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
            *time += self.group_step;
            return;
        }

        // Get the entity or return if there is not one.
        let Some(mut entity) = scene.get(id) else {
            return;
        };

        // Show this entity. Entities without a `Visibility` are fully visible by
        // default, so one is attached here rather than skipping the entity.
        let visibility = entity.get_or_default::<Visibility>();
        let before = visibility.amount.clone();
        let after = Timeline::keyframes(0.0)
            .at(*time, 0.0)
            .ease_at(*time + self.duration, 1.0, self.easing)
            .build();

        visibility.amount = Timeline::branch(*time, before, after);
        *time += self.step;
    }
}

impl Action for Show {
    fn apply(&self, mut stage: Stage, ids: &[EntityId]) -> Option<f32> {
        let mut cursor = stage.cursor();
        let scene = stage.scene();
        for &id in ids.iter() {
            self.show(scene, &mut cursor, id);
        }

        // Returns the length of this animation.
        if !ids.is_empty() {
            cursor += self.duration;
        }

        Some(cursor - stage.cursor())
    }
}

// ----- Hide -----

pub struct Hide {
    pub step: f32,
    pub group_step: f32,
    pub duration: f32,
    pub easing: Easing,
}

impl Default for Hide {
    fn default() -> Self {
        Self {
            step: 0.0,
            group_step: 0.0,
            duration: 0.3,
            easing: Easing::CubicInOut,
        }
    }
}

impl Hide {
    fn hide(&self, scene: &mut Scene, time: &mut f32, id: EntityId) {
        let children = scene
            .get(id)
            .and_then(|e| e.as_group())
            .map(|g| g.children());

        if let Some(children) = children {
            for id in children.into_iter() {
                self.hide(scene, time, id);
            }
            *time += self.group_step;
            return;
        }

        // Get the entity or return if there is not one.
        let Some(mut entity) = scene.get(id) else {
            return;
        };

        // Multiply alpha for fill and stroke during the interval of this hide.
        // After the animation, visibility takes over.
        let multiply = Timeline::keyframes(1.0)
            .at(*time, 1.0)
            .ease_at(*time + self.duration, 0.0, self.easing)
            .step_at(*time + self.duration, 1.0)
            .build();

        // Operate on fill and stroke. We fade for hiding to avoid quick unwrites.
        if let Some(fill) = entity.get::<Fill>() {
            let multiply = multiply.clone();
            let old = fill.color.clone();
            fill.color = Timeline::dynamic(move |t: f32| {
                let multiplier = multiply.clone();
                let color = old.sample(t);
                color.with_alpha(color.alpha * multiplier.sample(t))
            });
        }

        if let Some(stroke) = entity.get::<Stroke>() {
            let old = stroke.color.clone();
            stroke.color = Timeline::dynamic(move |t: f32| {
                let multiplier = multiply.clone();
                old.sample(t)
                    .map(move |c| c.with_alpha(c.alpha * multiplier.sample(t)))
            });
        }

        // After the fill and stroke are animated away, they reset, but visibility
        // jumps to zero.
        let visibility = entity.get_or_default::<Visibility>();
        let before = visibility.amount.clone();
        let after = Timeline::keyframes(1.0)
            .at(*time + self.duration, 1.0)
            .step_at(*time + self.duration, 0.0)
            .build();

        visibility.amount = before.multiply(after);
        *time += self.step;
    }
}

impl Action for Hide {
    fn apply(&self, mut stage: Stage, ids: &[EntityId]) -> Option<f32> {
        let mut cursor = stage.cursor();
        let scene = stage.scene();
        for &id in ids.iter() {
            self.hide(scene, &mut cursor, id);
        }

        // Returns the length of this animation.
        if !ids.is_empty() {
            cursor += self.duration;
        }

        Some(cursor - stage.cursor())
    }
}

//! Animations to display and hide entities.

use crate::scene::{EntityId, Fill, Stroke, Visibility};
use crate::story::{Action, Stage, Story, Timing};
use crate::timeline::Timeline;

// ----- Show -----

#[derive(Debug, Clone, Copy, Default)]
pub struct Show;

impl Action for Show {
    fn apply(&self, id: EntityId, mut stage: Stage) {
        let time = stage.cursor();
        let duration = stage.duration();
        let easing = stage.easing();

        if let Some(mut entity) = stage.scene().get(id) {
            let visibility = entity.get_or_default::<Visibility>();
            let before = visibility.amount.clone();
            let after = Timeline::keyframes(0.0)
                .at(time, 0.0)
                .ease_at(time + duration, 1.0, easing)
                .build();

            entity.visibility(Timeline::branch(time, before, after));
        }
    }
}

// ----- Hide -----

#[derive(Debug, Clone, Copy, Default)]
pub struct Hide;

impl Action for Hide {
    fn apply(&self, id: EntityId, mut stage: Stage) {
        let time = stage.cursor();
        let duration = stage.duration();
        let easing = stage.easing();

        // Get the entity or return if there is not one.
        let Some(mut entity) = stage.scene().get(id) else {
            return;
        };

        // Multiply alpha for fill and stroke during the interval of this hide.
        // After the animation, visibility takes over.
        let multiply = Timeline::keyframes(1.0)
            .at(time, 1.0)
            .ease_at(time + duration, 0.0, easing)
            .step_at(time + duration, 1.0)
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
            .at(time + duration, 1.0)
            .step_at(time + duration, 0.0)
            .build();

        visibility.amount = before.multiply(after);
    }
}

// ----- Story -----

impl Story {
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
        let ids = self.scene.entities();
        self.apply_with_timing(&Hide, &ids, &Timing::parallel())
    }

    pub fn hide_slice(&mut self, ids: &[EntityId]) -> &mut Self {
        self.apply(Hide, ids)
    }
}

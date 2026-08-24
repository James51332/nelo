//! Actions for transforming entities within the scene.

use crate::{
    scene::{EntityId, Step, Transform},
    story::{Action, Stage, Story},
    timeline::{Easing, Timeline},
};
use glam::{Mat2, Vec2};

pub struct ApplyTransform {
    pub duration: f32,
    pub step: Step,
    pub easing: Easing,
}

impl ApplyTransform {
    pub fn new(duration: f32, step: Step, easing: Easing) -> Self {
        Self {
            duration,
            step,
            easing,
        }
    }

    pub fn step(step: Step) -> Self {
        Self::new(1.0, step, Easing::CubicInOut)
    }
}

impl Action for ApplyTransform {
    fn apply(&self, mut stage: Stage, ids: &[EntityId]) -> Option<f32> {
        let t = stage.cursor();

        let step = match self.step.clone() {
            Step::Matrix(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(Mat2::IDENTITY))
                    .ease_at(self.duration, timeline, self.easing)
                    .build()
                    .flatten();
                Step::Matrix(blended.shift(t))
            }
            Step::Translate(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(Vec2::ZERO))
                    .ease_at(self.duration, timeline, self.easing)
                    .build()
                    .flatten();
                Step::Translate(blended.shift(t))
            }
            Step::Scale(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(0.0))
                    .ease_at(self.duration, timeline, self.easing)
                    .build()
                    .flatten();
                Step::Scale(blended.shift(t))
            }
            Step::Rotate(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(0.0))
                    .ease_at(self.duration, timeline, self.easing)
                    .build()
                    .flatten();
                Step::Rotate(blended.shift(t))
            }
        };

        for &id in ids.iter() {
            if let Some(mut entity) = stage.scene().get(id) {
                let transform: &mut Transform = entity.get_or_default();
                transform.push(step.clone());
            }
        }

        // Return the duration
        Some(self.duration)
    }
}

// ----- Story -----

impl Story {
    pub fn matrix(&mut self, delta: impl Into<Timeline<Mat2>>, id: EntityId) {
        let step = Step::Matrix(delta.into());
        self.apply(ApplyTransform::step(step), &[id]);
    }

    pub fn translate(&mut self, delta: impl Into<Timeline<Vec2>>, id: EntityId) {
        let step = Step::Translate(delta.into());
        self.apply(ApplyTransform::step(step), &[id]);
    }

    pub fn scale(&mut self, delta: impl Into<Timeline<f32>>, id: EntityId) {
        let step = Step::Scale(delta.into());
        self.apply(ApplyTransform::step(step), &[id]);
    }

    pub fn rotate(&mut self, delta: impl Into<Timeline<f32>>, id: EntityId) {
        let step = Step::Rotate(delta.into());
        self.apply(ApplyTransform::step(step), &[id]);
    }
}

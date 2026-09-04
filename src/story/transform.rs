//! Actions for transforming entities within the scene.

use crate::{
    scene::{EntityId, Step, Transform},
    story::{Action, Stage, Story, Target},
    timeline::Timeline,
};
use glam::{Mat2, Vec2};

impl Action for Step {
    fn apply(&self, id: EntityId, mut stage: Stage) {
        let time = stage.cursor();
        let duration = stage.duration();
        let easing = stage.easing();

        let step = match self.clone() {
            Step::Matrix(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(Mat2::IDENTITY))
                    .ease_at(duration, timeline, easing)
                    .build()
                    .flatten();
                Step::Matrix(blended.shift(time))
            }
            Step::Translate(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(Vec2::ZERO))
                    .ease_at(duration, timeline, easing)
                    .build()
                    .flatten();
                Step::Translate(blended.shift(time))
            }
            Step::Scale(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(0.0))
                    .ease_at(duration, timeline, easing)
                    .build()
                    .flatten();
                Step::Scale(blended.shift(time))
            }
            Step::Rotate(timeline) => {
                let blended = Timeline::keyframes(Timeline::constant(0.0))
                    .ease_at(duration, timeline, easing)
                    .build()
                    .flatten();
                Step::Rotate(blended.shift(time))
            }
        };

        if let Some(mut entity) = stage.scene().get(id) {
            let transform: &mut Transform = entity.get_or_default();
            transform.push(step.clone());
        }
    }

    fn target() -> Target {
        Target::Roots
    }
}

// ----- Story -----

impl Story {
    pub fn matrix(&mut self, delta: impl Into<Timeline<Mat2>>, id: EntityId) {
        self.apply(Step::Matrix(delta.into()), &[id]);
    }

    pub fn translate(&mut self, delta: impl Into<Timeline<Vec2>>, id: EntityId) {
        self.apply(Step::Translate(delta.into()), &[id]);
    }

    pub fn scale(&mut self, delta: impl Into<Timeline<f32>>, id: EntityId) {
        self.apply(Step::Scale(delta.into()), &[id]);
    }

    pub fn rotate(&mut self, delta: impl Into<Timeline<f32>>, id: EntityId) {
        self.apply(Step::Rotate(delta.into()), &[id]);
    }
}

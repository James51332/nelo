//! Helper for changing the geometry of any object.

use crate::{
    scene::{EntityId, Spline},
    story::{Action, Stage, Story},
    timeline::Timeline,
};

impl Action for Spline {
    fn apply(&self, id: EntityId, mut stage: Stage) {
        let start = stage.cursor();
        let duration = stage.duration();
        let end = start + duration;
        let easing = stage.easing();

        let Some(mut entity) = stage.scene().get(id) else {
            return;
        };

        // For now we only update the geometry if we are a spline.
        if let Some(spline) = entity.get::<Spline>() {
            // Update the path.
            spline.spline_path = Timeline::keyframes(spline.spline_path.clone())
                .at(start, spline.spline_path.clone())
                .ease_at(end, self.spline_path.clone().shift(start), easing)
                .build()
                .flatten();

            // Update the alpha.
            spline.start_alpha = Timeline::keyframes(spline.start_alpha.clone())
                .at(start, spline.start_alpha.clone())
                .ease_at(end, self.start_alpha.clone(), easing)
                .build()
                .flatten();

            spline.end_alpha = Timeline::keyframes(spline.end_alpha.clone())
                .ease_at(duration, self.end_alpha.clone(), easing)
                .build()
                .flatten()
                .shift(start);

            spline.close = Timeline::branch(end, spline.close.clone(), self.close.clone());
        }
    }
}

// ----- Story -----

impl Story {
    pub fn spline(&mut self, spline: impl Into<Spline>, id: EntityId) {
        self.apply(spline.into(), &[id]);
    }
}

//! Actions for changing color of objects

use crate::{
    scene::{Color, EntityId, Fill, Stroke},
    story::{Action, Stage, Story},
    timeline::{Along, Timeline, TimelineAlong},
};

// ----- Fill -----

impl Action for Fill {
    fn apply(&self, id: EntityId, mut stage: Stage<'_>) {
        let time = stage.cursor();
        let duration = stage.duration();
        let easing = stage.easing();

        if let Some(mut entity) = stage.scene().get(id) {
            // Get the fill for the entity.
            let fill = entity.get::<Fill>();

            match fill {
                // If we have a fill then we need to bridge to the new timeline.
                Some(fill) => {
                    fill.color = Timeline::keyframes(fill.color.clone())
                        .at(time, fill.color.clone())
                        .ease_at(time + duration, self.color.clone().shift(time), easing)
                        .build()
                        .flatten();
                }
                // If we don't, then we fade in from nothing.
                None => {
                    let initial = Timeline::constant(self.color.sample(0.0).with_alpha(0.0));

                    let color = Timeline::keyframes(initial)
                        .ease_at(duration, self.color.clone(), easing)
                        .build()
                        .flatten()
                        .shift(time);

                    entity.attach(Fill::new(color));
                }
            };
        }
    }
}

// ----- Stroke -----

impl Action for Stroke {
    fn apply(&self, id: EntityId, stage: Stage) {
        ApplyStroke::new(self.color.clone(), self.weight.clone()).apply(id, stage);
    }
}

// ----- ApplyStroke -----

/// Helper to apply an action to only weight or color of strokes. Strokes can still be
/// applied directly also.
pub struct ApplyStroke {
    pub color: Option<Timeline<Along<Color>>>,
    pub weight: Option<Timeline<Along<f32>>>,
}

impl ApplyStroke {
    pub fn new<T, U>(color: T, weight: U) -> Self
    where
        T: Into<TimelineAlong<Color>>,
        U: Into<TimelineAlong<f32>>,
    {
        Self {
            color: Some(color.into().inner()),
            weight: Some(weight.into().inner()),
        }
    }

    pub fn color<T>(color: T) -> Self
    where
        T: Into<TimelineAlong<Color>>,
    {
        Self {
            color: Some(color.into().inner()),
            weight: None,
        }
    }

    pub fn weight<T>(weight: T) -> Self
    where
        T: Into<TimelineAlong<f32>>,
    {
        Self {
            color: None,
            weight: Some(weight.into().inner()),
        }
    }
}

impl Action for ApplyStroke {
    fn apply(&self, id: EntityId, mut stage: Stage) {
        let time = stage.cursor();
        let duration = stage.duration();
        let easing = stage.easing();

        if let Some(mut entity) = stage.scene().get(id) {
            // Either clone the stroke before, or build the default.
            match entity.get::<Stroke>() {
                Some(stroke) => {
                    // If we have a color blend it.
                    if let Some(color) = self.color.clone() {
                        let blended = Timeline::keyframes(stroke.color.clone())
                            .ease_at(duration, color, easing)
                            .build()
                            .flatten()
                            .shift(time);

                        stroke.color = blended;
                    }

                    // If we have a weight, blend it.
                    if let Some(weight) = self.weight.clone() {
                        let blended = Timeline::keyframes(stroke.weight.clone())
                            .ease_at(duration, weight, easing)
                            .build()
                            .flatten()
                            .shift(time);

                        stroke.weight = blended;
                    }
                }
                None => {
                    // The color is either the given one or a default.
                    let color = self.color.clone().unwrap_or(Stroke::default().color);

                    // The weight blends from zero, to default or specified.
                    let zero_weight = Timeline::constant(Timeline::constant(0.0).along());
                    let weight = self.weight.clone().unwrap_or(Stroke::default().weight);

                    let blended_weight = Timeline::keyframes(zero_weight)
                        .ease_at(duration, weight, easing)
                        .build()
                        .flatten()
                        .shift(time);

                    entity.attach(Stroke::new(color, blended_weight));
                }
            };
        }
    }
}

// ----- Story -----

impl Story {
    pub fn fill(&mut self, id: EntityId, color: impl Into<Timeline<Color>>) -> &mut Self {
        self.fill_slice(&[id], color)
    }

    pub fn fill_slice(&mut self, ids: &[EntityId], color: impl Into<Timeline<Color>>) -> &mut Self {
        self.apply(Fill::new(color), ids)
    }

    pub fn stroke(&mut self, id: EntityId, color: Color) -> &mut Self {
        self.stroke_slice(&[id], color)
    }

    pub fn stroke_slice(&mut self, ids: &[EntityId], color: Color) -> &mut Self {
        self.apply(ApplyStroke::color(color), ids)
    }

    pub fn stroke_weight(&mut self, id: EntityId, weight: f32) -> &mut Self {
        self.stroke_weight_slice(&[id], weight)
    }

    pub fn stroke_weight_slice(&mut self, ids: &[EntityId], weight: f32) -> &mut Self {
        self.apply(ApplyStroke::weight(weight), ids)
    }
}

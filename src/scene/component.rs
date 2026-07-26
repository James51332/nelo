//! A collection of components which are used by the renderers.

use crate::scene::{EntityRef, Scene};
use crate::timeline::{Along, Timeline, TimelineSpline};
use glam::prelude::*;

// ----- Spline -----

pub struct Spline {
    pub spline_path: Timeline<Along<Vec2>>,
    pub start_alpha: Timeline<f32>,
    pub end_alpha: Timeline<f32>,
}

impl Scene {
    pub fn spline<T>(&mut self, spline_path: T) -> EntityRef<'_>
    where
        T: Into<TimelineSpline>,
    {
        self.spline_with_range(spline_path, 0.0, 1.0)
    }

    pub fn spline_with_range<T, U, V>(&mut self, spline_path: T, start: U, end: V) -> EntityRef<'_>
    where
        T: Into<TimelineSpline>,
        U: Into<Timeline<f32>>,
        V: Into<Timeline<f32>>,
    {
        self.create()
            .attach(Spline {
                spline_path: spline_path.into().0.0,
                start_alpha: start.into(),
                end_alpha: end.into(),
            })
            .attach(Stroke::default())
    }
}

// ----- Circle -----

/// A circle has no attached data. It has a radius of 1 but can
/// be scaled using a transform.
pub struct Circle;

impl Scene {
    /// Returns an `EntityRef` with circle geometry attached. The default
    /// circle is at the world origin with a radius of one and white fill.
    pub fn circle(&mut self) -> EntityRef<'_> {
        self.create().attach(Circle).attach(Fill::default())
    }
}

// ----- Fill -----

/// A fill is a color over time.
pub struct Fill {
    pub color: Timeline<Vec4>,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            color: Vec4::ONE.into(),
        }
    }
}

// ----- Stroke -----

pub struct Stroke {
    pub color: Timeline<Along<Vec4>>,
    pub weight: Timeline<Along<f32>>,
}

impl Stroke {
    pub fn sample(&self, t: f32) -> (Along<Vec4>, Along<f32>) {
        (self.color.sample(t), self.weight.sample(t))
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            weight: Timeline::constant(0.1).along().into(),
            color: Timeline::constant(Vec4::ONE).along().into(),
        }
    }
}

//! Types and traits for defining values which vary both w.r.t time and spline parameter.

use crate::timeline::{Lerp, Timeline};
use glam::prelude::*;

// ----- Along<T> -----

/// Along defines a value "along" a parametric curve. It's sampled over alpha, which
/// usually varies from zero to one.
#[derive(Clone)]
pub struct Along<T: 'static>(Timeline<T>);

impl<T> Along<T> {
    /// Sample this instance with parameter value alpha.
    pub fn sample(&self, alpha: f32) -> T
    where
        T: Clone,
    {
        self.0.sample(alpha)
    }

    pub fn timeline(self) -> Timeline<T> {
        self.0
    }

    pub fn map<F, U>(self, map: F) -> Along<U>
    where
        T: Clone,
        U: Clone + 'static,
        F: Fn(T) -> U + Clone + 'static,
    {
        self.timeline().map(map).along()
    }
}

impl<T> Timeline<T> {
    /// Converts this timeline into an `Along`. This maps t to alpha.
    pub fn along(self) -> Along<T> {
        Along(self)
    }
}

impl<T: Clone + Lerp> Lerp for Along<T> {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(Timeline::interpolate(&a.0, &b.0, t))
    }
}

// ----- TimelineSpline -----

/// Helper type used for building curves from parametrics.
#[derive(Clone)]
pub struct TimelineSpline(pub(crate) TimelineAlong<Vec2>);

impl Lerp for TimelineSpline {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(TimelineAlong::interpolate(&a.0, &b.0, t))
    }
}

// ----- TimelineAlong -----

#[derive(Clone)]
pub struct TimelineAlong<T: 'static>(pub(crate) Timeline<Along<T>>);

impl<T: Clone + Lerp> Lerp for TimelineAlong<T> {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(Timeline::interpolate(&a.0, &b.0, t))
    }
}

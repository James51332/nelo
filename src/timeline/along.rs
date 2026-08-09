//! Types and traits for defining values which vary both w.r.t time and spline parameter.

use crate::timeline::{Lerp, Timeline};
use glam::prelude::*;

// ----- Along<T> -----

/// Along defines a value "along" a parametric curve. It's sampled over alpha, which
/// usually varies from zero to one.
#[derive(Clone)]
pub struct Along<T: Clone + 'static>(Timeline<T>);

impl<T: Clone> Along<T> {
    pub fn length(self) -> Option<f32> {
        self.0.length()
    }

    pub fn sample(&self, alpha: f32) -> T
    where
        T: Clone,
    {
        self.0.sample(alpha)
    }

    pub fn with_length(self, length: f32) -> Self {
        Along(self.0.with_length(length))
    }

    pub fn repeat(self) -> Self {
        Along(self.0.repeat())
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
pub struct TimelineAlong<T: Clone + 'static>(pub(crate) Timeline<Along<T>>);

impl<T: Clone + Lerp> Lerp for TimelineAlong<T> {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(Timeline::interpolate(&a.0, &b.0, t))
    }
}

// ----- Conversion -----

impl<T: Clone> Along<T> {
    pub fn timeline(self) -> Timeline<T> {
        self.0
    }
}

impl<T: Clone> Timeline<T> {
    /// Converts this timeline into an `Along`. This maps t to alpha.
    pub fn along(self) -> Along<T> {
        Along(self)
    }
}

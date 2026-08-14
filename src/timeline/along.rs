//! Types and traits for defining values which vary both w.r.t time and spline parameter.

use crate::timeline::Timeline;
use glam::prelude::*;

// ----- Along -----

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

    pub fn is_constant(&self) -> bool {
        self.0.is_constant()
    }

    pub fn repeat(self) -> Self {
        Along(self.0.repeat())
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

// ----- TimelineAlong -----

#[derive(Clone)]
pub struct TimelineAlong<T: Clone + 'static> {
    inner: Timeline<Along<T>>,
}

impl<T: Clone> TimelineAlong<T> {
    pub fn new(inner: Timeline<Along<T>>) -> Self {
        Self { inner }
    }

    pub fn inner(self) -> Timeline<Along<T>> {
        self.inner
    }
}

// ----- TimelineSpline -----

/// Helper type used for building curves from parametrics.
#[derive(Clone)]
pub struct TimelineSpline {
    inner: TimelineAlong<Vec2>,
}

impl TimelineSpline {
    pub fn new(inner: TimelineAlong<Vec2>) -> Self {
        Self { inner }
    }

    pub fn inner(self) -> Timeline<Along<Vec2>> {
        self.inner.inner()
    }
}

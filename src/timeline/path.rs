//! A path is a Along<Vec2> in the interval [0, 1].

use crate::timeline::{Along, Lerp, Timeline};
use glam::prelude::*;

/// A path is a parameter which varies w/ respect to alpha. It can convert to a
/// `Timeline<Vec2>` using `Path::timeline()` or `Timeline::from()`. We also
/// add a few convenience methods so that we can interface with paths.
pub type Path = Along<Vec2>;

impl Timeline<Vec2> {
    /// Alias for .along() on Timeline<Vec2>
    pub fn path(self) -> Path {
        self.along()
    }
}

impl Path {
    /// Returns a path around the unit circle with length one second.
    /// Loops if sampled out of bounds.
    pub fn circle() -> Self {
        Timeline::dynamic(move |t: f32| {
            let theta = t * std::f32::consts::TAU;
            Vec2::new(theta.cos(), theta.sin())
        })
        .with_length(1.0)
        .path()
    }

    /// Traces a path CCW around a square with vertices at (+/- 1, +/- 1) in one second.
    /// Loops if sampled out of bounds.
    pub fn square() -> Self {
        Timeline::dynamic(move |t: f32| {
            let t = t.rem_euclid(1.0);

            if t < 0.125 {
                Vec2::interpolate(&Vec2::X, &Vec2::ONE, t * 8.0)
            } else if t <= 0.375 {
                Vec2::interpolate(&Vec2::ONE, &Vec2::new(-1.0, 1.0), (t - 0.125) * 4.0)
            } else if t <= 0.625 {
                Vec2::interpolate(&Vec2::new(-1.0, 1.0), &Vec2::NEG_ONE, (t - 0.375) * 4.0)
            } else if t <= 0.875 {
                Vec2::interpolate(&Vec2::NEG_ONE, &Vec2::new(1.0, -1.0), (t - 0.625) * 4.0)
            } else {
                Vec2::interpolate(&Vec2::new(1.0, -1.0), &Vec2::X, (t - 0.875) * 8.0)
            }
        })
        .with_length(1.0)
        .path()
    }

    /// Returns a triangle path.
    pub fn triangle() -> Self {
        // We spend equal time on each side even though we aren't equilateral.
        Timeline::keyframes(Vec2::new(0.5, 0.0))
            .at(1.0 / 6.0, Vec2::new(0.0, 1.0))
            .at(3.0 / 6.0, Vec2::new(-1.0, -1.0))
            .at(5.0 / 6.0, Vec2::new(1.0, -1.0))
            .at(6.0 / 6.0, Vec2::new(0.5, 0.0))
            .build()
            .repeat()
            .path()
    }

    /// Returns a path from a to b over the course of one second. Continues with constant
    /// velocity if sampled out of bounds.
    pub fn line(a: Vec2, b: Vec2) -> Self {
        Timeline::dynamic(move |t| Vec2::interpolate(&a, &b, t))
            .with_length(1.0)
            .path()
    }

    pub fn add(self, path: impl Into<Path>) -> Self {
        self.timeline().add(path.into().timeline()).path()
    }

    pub fn multiply(self, scale: impl Into<Along<f32>>) -> Self {
        self.timeline().multiply(scale.into().timeline()).path()
    }

    pub fn map<T: Clone + 'static, U: Clone + 'static>(self, map: T) -> Along<U>
    where
        T: Fn(Vec2) -> U,
    {
        self.timeline().map(map).along()
    }
}

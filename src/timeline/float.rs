//! Helper methods for timelines of f32.

use std::f32::consts::TAU;

use crate::timeline::{Easing, Timeline};

impl Timeline<f32> {
    /// Returns an f32 timeline which forwards the input time.
    pub fn time() -> Self {
        Self::dynamic(|t| t)
    }

    /// Returns `Timeline::time()` or zero if time() < 0.
    pub fn ramp() -> Self {
        Self::dynamic(|t: f32| t.max(0.0))
    }

    /// Returns an f32 timeline which forwards its input at a multiplied rate.
    pub fn rate(rate: f32) -> Self {
        Self::dynamic(move |t| t * rate)
    }

    /// Returns an f32 timeline which goes from zero to one with no interpolation
    /// each `period`.
    pub fn sawtooth(period: f32) -> Self {
        Self::dynamic(move |t: f32| (t / period).rem_euclid(1.0)).with_length(period)
    }

    /// Returns a timeline which goes from zero to one, and then from one to zero
    /// each period.
    pub fn triangle(period: f32) -> Self {
        let half_period = 0.5 * period;
        Self::dynamic(move |t: f32| {
            let shifted = ((t + half_period) / half_period).rem_euclid(2.0);
            (shifted - 1.0).abs()
        })
        .with_length(period)
    }

    /// Returns a repeating timeline which jumps from 0 to 1 at period / 2.0.
    pub fn square(period: f32) -> Self {
        let half_period = period / 2.0;
        Self::dynamic(move |t: f32| {
            let t = t.rem_euclid(period);
            if t < half_period { 0.0 } else { 1.0 }
        })
        .with_length(period)
    }

    /// Returns a cosine wave with given `period`.
    pub fn cos(period: f32) -> Self {
        Self::dynamic(move |t: f32| (t / period * TAU).cos()).with_length(period)
    }

    /// Returns a sine wave with given `period`.
    pub fn sin(period: f32) -> Self {
        Self::dynamic(move |t: f32| (t / period * TAU).sin()).with_length(period)
    }

    /// Adds `Easing::CubicInOut` to this timeline
    pub fn ease(self) -> Self {
        self.then(Easing::CubicInOut)
    }

    /// Returns an f32 timeline that clamps the value of `self` to [min, max].
    pub fn clamp(self, min: f32, max: f32) -> Self {
        self.map(move |t| t.clamp(min, max))
    }

    /// Mirror of composition. Evaluates `self` and uses output as time for `outer`.
    pub fn then<U: Clone + 'static>(self, outer: impl Into<Timeline<U>>) -> Timeline<U> {
        outer.into().compose(self)
    }
}

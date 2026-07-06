//! Helper module to implement composition features for `Timeline`s

use crate::timeline::Timeline;
use crate::timeline::signal::Signal;

impl<T: Clone + 'static> Timeline<T> {
    /// Resamples this timeline with the inner parameter. Length is set to
    /// that of inner timeline.
    pub fn compose(self, inner: Timeline<f32>) -> Self {
        Self::dynamic(Compose { outer: self, inner })
    }

    /// Delays this timeline by `delay` seconds and creates a new timeline.
    pub fn shift(self, delay: f32) -> Self {
        Self::dynamic(Shift {
            timeline: self,
            delay,
        })
    }
}

impl<T: Clone + 'static + std::ops::Add<T>> Timeline<T> {
    /// Adds this timeline to the timeline. The length is the maximum of the two.
    pub fn add(self, other: Timeline<T>) -> Timeline<<T as std::ops::Add<T>>::Output> {
        Timeline::<<T as std::ops::Add<T>>::Output>::dynamic(Sum {
            first: self,
            second: other,
        })
    }
}

impl<T: Clone + 'static + std::ops::Mul<T>> Timeline<T> {
    /// Multiply this timeline (LHS) with other timeline (RHS). The length is the maximum
    /// of the two lengths.
    pub fn multiply(self, other: Timeline<T>) -> Timeline<<T as std::ops::Mul<T>>::Output> {
        Timeline::<<T as std::ops::Mul<T>>::Output>::dynamic(Product {
            first: self,
            second: other,
        })
    }
}

/// Type-erased struct that implements signal to support `Timeline::compose`
/// method.
struct Compose<T> {
    outer: Timeline<T>,
    inner: Timeline<f32>,
}

impl<T: Clone + 'static> Signal for Compose<T> {
    type Output = T;

    fn sample(&self, t: f32) -> Self::Output {
        self.outer.sample(self.inner.sample(t))
    }

    fn length(&self) -> Option<f32> {
        self.inner.length()
    }
}

/// Type-erased struct that implements signal to support `Timeline::shift`
/// method.
struct Shift<T> {
    timeline: Timeline<T>,
    delay: f32,
}

impl<T: Clone + 'static> Signal for Shift<T> {
    type Output = T;

    fn sample(&self, t: f32) -> Self::Output {
        self.timeline.sample(t - self.delay)
    }

    fn length(&self) -> Option<f32> {
        if self.delay >= 0.0 {
            self.timeline.length().map(|t| t + self.delay)
        } else {
            None
        }
    }
}

/// Type-erased struct that implements signal to support `Timeline::add`
/// method.
struct Sum<T> {
    first: Timeline<T>,
    second: Timeline<T>,
}

impl<T: Clone + std::ops::Add<T> + 'static> Signal for Sum<T> {
    type Output = <T as std::ops::Add<T>>::Output;

    fn sample(&self, t: f32) -> Self::Output {
        self.first.sample(t) + self.second.sample(t)
    }

    fn length(&self) -> Option<f32> {
        match (self.first.length(), self.second.length()) {
            (Some(x), Some(y)) => Some(x.max(y)),
            _ => None,
        }
    }
}

/// Type-erased struct that implements signal to support `Timeline::multiply`
/// method.
struct Product<T> {
    first: Timeline<T>,
    second: Timeline<T>,
}

impl<T: Clone + std::ops::Mul<T> + 'static> Signal for Product<T> {
    type Output = <T as std::ops::Mul<T>>::Output;

    fn sample(&self, t: f32) -> Self::Output {
        self.first.sample(t) * self.second.sample(t)
    }

    fn length(&self) -> Option<f32> {
        match (self.first.length(), self.second.length()) {
            (Some(x), Some(y)) => Some(x.max(y)),
            _ => None,
        }
    }
}

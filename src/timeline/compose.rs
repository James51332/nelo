//! Helper module to implement composition features for `Timeline`s

use crate::timeline::Timeline;
use crate::timeline::signal::Signal;
use std::ops::{Add, Mul};

impl Timeline<f32> {
    /// For f32 timelines, we can reverse the compose logic. Since multiple
    /// composes require that we apply the innermost sampling last, we introduce
    /// this API to reverse the logic, and apply in the natural order.
    pub fn then<U: Clone + 'static>(self, outer: impl Into<Timeline<U>>) -> Timeline<U> {
        Timeline::dynamic(Compose {
            outer: outer.into(),
            inner: self,
        })
    }

    /// Returns a f32 timeline which forwards it's input at a multiplied rate.
    /// Useful for [`Timeline<f32>::then`] API.
    pub fn rate(rate: f32) -> Self {
        Self::dynamic(move |t| t * rate)
    }

    /// Returns an f32 timeline which goes from zero to one with no interpolation
    /// and given period.
    pub fn sawtooth(period: f32) -> Self {
        Self::dynamic(move |t: f32| (t / period).rem_euclid(1.0)).with_length(period)
    }
}

impl<T: Clone + 'static> Timeline<Timeline<T>> {
    /// Reduces depth of timeline of timelines by one by sampling both at the time
    /// with the same input parameter.
    ///
    /// To use this with a resampled outer timeline, use `.compose().flatten()`. To
    /// resample the inner timeline, use `.map(|x| x.compose())`, and to resample both,
    /// use `.flatten().compose()`.
    pub fn flatten(self) -> Timeline<T> {
        Timeline::dynamic(move |t| self.sample(t).sample(t))
    }
}

impl<T: Clone + 'static> Timeline<T> {
    /// Resamples this timeline with the inner parameter. Length is set to
    /// that of inner timeline.
    pub fn compose(self, inner: impl Into<Timeline<f32>>) -> Self {
        Self::dynamic(Compose {
            outer: self,
            inner: inner.into(),
        })
    }

    /// Delays this timeline by `delay` seconds and creates a new timeline.
    pub fn shift(self, delay: f32) -> Self {
        Self::dynamic(Shift {
            timeline: self,
            delay,
        })
    }

    /// Adds this timeline to the timeline. The length is the maximum of the two.
    pub fn add<U: Clone + 'static>(
        self,
        other: impl Into<Timeline<U>>,
    ) -> Timeline<<T as Add<U>>::Output>
    where
        T: Add<U>,
        <T as Add<U>>::Output: Clone,
    {
        Timeline::dynamic(Sum {
            first: self,
            second: other.into(),
        })
    }

    /// Multiply this timeline (LHS) with other timeline (RHS). The length is the maximum
    /// of the two lengths.
    pub fn multiply<U: Clone + 'static>(
        self,
        other: impl Into<Timeline<U>>,
    ) -> Timeline<<T as Mul<U>>::Output>
    where
        T: Mul<U>,
        <T as Mul<U>>::Output: Clone,
    {
        Timeline::dynamic(Product {
            first: self,
            second: other.into(),
        })
    }

    /// Consume this timeline and return a new timeline with a mapped output value. The length
    /// of the timeline is unchanged.
    pub fn map<U: Clone + 'static>(self, map: impl Fn(T) -> U + Clone + 'static) -> Timeline<U> {
        Timeline::dynamic(Map {
            inner: self,
            map: Box::new(map),
        })
    }
}

/// Type-erased struct that implements signal to support `Timeline::compose` method.
#[derive(Clone)]
struct Compose<T: Clone + 'static> {
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
#[derive(Clone)]
struct Shift<T: Clone + 'static> {
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
#[derive(Clone)]
struct Sum<T: Clone + 'static, U: Clone + 'static> {
    first: Timeline<T>,
    second: Timeline<U>,
}

impl<T: Clone + Add<U> + 'static, U: Clone + 'static> Signal for Sum<T, U>
where
    <T as Add<U>>::Output: Clone,
{
    type Output = <T as Add<U>>::Output;

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
#[derive(Clone)]
struct Product<T: Clone + 'static, U: Clone + 'static> {
    first: Timeline<T>,
    second: Timeline<U>,
}

impl<T: Clone + Mul<U> + 'static, U: Clone + 'static> Signal for Product<T, U>
where
    <T as Mul<U>>::Output: Clone,
{
    type Output = <T as Mul<U>>::Output;

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

/// Type-erased struct that implements signal to support `Timeline::map`
struct Map<T: Clone + 'static, U: Clone + 'static> {
    inner: Timeline<T>,
    map: Box<dyn MapClone<T, U>>,
}

impl<T: Clone + 'static, U: Clone + 'static> Signal for Map<T, U> {
    type Output = U;

    fn sample(&self, t: f32) -> Self::Output {
        (self.map)(self.inner.sample(t))
    }

    fn length(&self) -> Option<f32> {
        self.inner.length()
    }
}

/// We have to use this workaround to support map being clone.
trait MapClone<T: Clone + 'static, U: Clone + 'static>: Fn(T) -> U + 'static {
    fn clone_box(&self) -> Box<dyn MapClone<T, U>>;
}

// If a closure is clone, then it's map clone.
impl<S, T: Clone + 'static, U: Clone + 'static> MapClone<T, U> for S
where
    S: Fn(T) -> U + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn MapClone<T, U>> {
        Box::new(self.clone())
    }
}

impl<T: Clone + 'static, U: Clone + 'static> Clone for Map<T, U> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            map: self.map.clone_box(),
        }
    }
}

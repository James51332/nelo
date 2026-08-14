//! Helper module to implement composition features for `Timeline`s

use crate::timeline::{Along, Signal, Timeline};
use std::ops::{Add, Mul};

// ----- Timeline -----

impl<T: Clone> Timeline<T> {
    /// Resamples this timeline with the inner parameter. Length is set to
    /// that of inner timeline.
    pub fn compose(self, inner: impl Into<Timeline<f32>>) -> Self {
        let inner = inner.into();
        if self.is_constant() {
            self
        } else if inner.is_constant() {
            Self::constant(self.sample(inner.sample(0.0)))
        } else {
            Self::dynamic(Compose {
                outer: self,
                inner: inner,
            })
        }
    }

    /// Delays this timeline by `delay` seconds and creates a new timeline.
    pub fn shift(self, delay: f32) -> Self {
        if self.is_constant() {
            self
        } else {
            Self::dynamic(Shift {
                timeline: self,
                delay,
            })
        }
    }

    /// Adds this timeline to the timeline. The length is the maximum of the two.
    pub fn add<U: Clone + 'static>(
        self,
        rhs: impl Into<Timeline<U>>,
    ) -> Timeline<<T as Add<U>>::Output>
    where
        T: Add<U>,
        <T as Add<U>>::Output: Clone,
    {
        let rhs = rhs.into();
        if self.is_constant() && rhs.is_constant() {
            Timeline::constant(self.sample(0.0) + rhs.sample(0.0))
        } else {
            Timeline::dynamic(Sum {
                first: self,
                second: rhs,
            })
        }
    }

    /// Multiply this timeline (LHS) with other timeline (RHS). The length is the maximum
    /// of the two lengths.
    pub fn multiply<U: Clone + 'static>(
        self,
        rhs: impl Into<Timeline<U>>,
    ) -> Timeline<<T as Mul<U>>::Output>
    where
        T: Mul<U>,
        <T as Mul<U>>::Output: Clone,
    {
        let rhs = rhs.into();
        if self.is_constant() && rhs.is_constant() {
            Timeline::constant(self.sample(0.0) * rhs.sample(0.0))
        } else {
            Timeline::dynamic(Product {
                first: self,
                second: rhs,
            })
        }
    }

    /// Consume this timeline and return a new timeline with a mapped output value. The length
    /// of the timeline is unchanged.
    pub fn map<U: Clone + 'static>(self, map: impl Fn(T) -> U + Clone + 'static) -> Timeline<U> {
        if self.is_constant() {
            Timeline::constant(map(self.sample(0.0)))
        } else {
            Timeline::dynamic(Map {
                inner: self,
                map: Box::new(map),
            })
        }
    }
}

// ----- Along -----

impl<T: Clone> Along<T> {
    pub fn compose(self, inner: impl Into<Along<f32>>) -> Self {
        self.timeline().compose(inner.into().timeline()).along()
    }

    pub fn shift(self, delay: f32) -> Self {
        self.timeline().shift(delay).along()
    }

    pub fn add<U: Clone + 'static>(self, rhs: impl Into<Along<U>>) -> Along<<T as Add<U>>::Output>
    where
        T: Add<U>,
        <T as Add<U>>::Output: Clone + 'static,
    {
        self.timeline().add(rhs.into().timeline()).along()
    }

    pub fn multiply<U: Clone + 'static>(
        self,
        rhs: impl Into<Along<U>>,
    ) -> Along<<T as Mul<U>>::Output>
    where
        T: Mul<U>,
        <T as Mul<U>>::Output: Clone + 'static,
    {
        self.timeline().multiply(rhs.into().timeline()).along()
    }

    pub fn map<F, U>(self, map: F) -> Along<U>
    where
        U: Clone + 'static,
        F: Fn(T) -> U + Clone + 'static,
    {
        self.timeline().map(map).along()
    }
}

// ----- Compose -----

/// Type-erased struct that implements signal to support `Timeline::compose` method.
#[derive(Clone)]
struct Compose<T: Clone + 'static> {
    outer: Timeline<T>,
    inner: Timeline<f32>,
}

impl<T: Clone> Signal for Compose<T> {
    type Output = T;

    fn sample(&self, t: f32) -> Self::Output {
        self.outer.sample(self.inner.sample(t))
    }

    fn length(&self) -> Option<f32> {
        self.inner.length()
    }
}

// ----- Shift -----

/// Type-erased struct that implements signal to support `Timeline::shift`
/// method.
#[derive(Clone)]
struct Shift<T: Clone + 'static> {
    timeline: Timeline<T>,
    delay: f32,
}

impl<T: Clone> Signal for Shift<T> {
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

// ----- Shift -----

/// Type-erased struct that implements signal to support `Timeline::add`
/// method.
#[derive(Clone)]
struct Sum<T: Clone + 'static, U: Clone + 'static> {
    first: Timeline<T>,
    second: Timeline<U>,
}

impl<T: Clone + Add<U>, U: Clone> Signal for Sum<T, U>
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

// ----- Product -----

/// Type-erased struct that implements signal to support `Timeline::multiply`
/// method.
#[derive(Clone)]
struct Product<T: Clone + 'static, U: Clone + 'static> {
    first: Timeline<T>,
    second: Timeline<U>,
}

impl<T: Clone + Mul<U>, U: Clone> Signal for Product<T, U>
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

// ----- Map -----

/// Type-erased struct that implements signal to support `Timeline::map`
struct Map<T: Clone + 'static, U: Clone + 'static> {
    inner: Timeline<T>,
    map: Box<dyn MapClone<T, U>>,
}

impl<T: Clone, U: Clone> Signal for Map<T, U> {
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
impl<F, T: Clone + 'static, U: Clone + 'static> MapClone<T, U> for F
where
    F: Fn(T) -> U + Clone + 'static,
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

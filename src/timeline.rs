//! Timelines: a concrete, sampleable instance of a [`Signal`].
//!
//! A [`Signal`] is the abstract *description* of a time-varying value; a
//! [`Timeline`] is the handle you actually store on a scene object and sample
//! each frame. It folds the two cases you hit in practice into one type:
//!
//! * [`Constant`](Timeline::Constant) — a value that never changes, held inline
//!   with no allocation and no dynamic dispatch.
//! * [`Dynamic`](Timeline::Dynamic) — a real signal, type-erased behind an
//!   `Arc` so a timeline stays `Clone` and can be shared cheaply.
//!
//! Callers sample both through the same [`sample`](Timeline::sample) call; the
//! constant case is purely an optimisation, not a separate API.

use crate::signal::Signal;
use std::sync::Arc;

/// A sampleable value over time: either a fixed constant or a shared [`Signal`].
pub enum Timeline<T> {
    /// A value that does not vary with time — stored inline, no allocation.
    Constant(T),
    /// A type-erased signal, shared by reference count.
    Dynamic(Arc<dyn Signal<Output = T>>),
}

impl<T: 'static> Timeline<T> {
    /// Wrap a fixed value that never changes.
    pub fn constant(s: T) -> Self {
        Self::Constant(s)
    }

    /// Erase a concrete signal (e.g. a closure) into a shareable timeline.
    pub fn dynamic(s: impl Signal<Output = T> + 'static) -> Self {
        Self::Dynamic(Arc::new(s))
    }

    /// The timeline's finite duration, or `None` if it runs forever. A constant
    /// has no inherent end; a dynamic one defers to its [`Signal::length`].
    pub fn length(&self) -> Option<f64> {
        match self {
            Self::Constant(_) => None,
            Self::Dynamic(s) => s.length(),
        }
    }
}

impl<T: Clone + 'static> Timeline<T> {
    /// The value at time `t`. A constant clones its held value; a dynamic one
    /// samples the underlying signal. (Sampling needs `Clone` so the constant
    /// case can hand back an owned value.)
    pub fn sample(&self, t: f64) -> T {
        match self {
            Self::Constant(s) => s.clone(),
            Self::Dynamic(s) => s.sample(t),
        }
    }

    /// Collapse either variant into a single `Arc<dyn Signal>` by promoting a
    /// constant to a [`Const`] signal. Lets wrappers like [`WithLength`] treat
    /// both variants uniformly without a branch of their own.
    fn into_signal(self) -> Arc<dyn Signal<Output = T>> {
        match self {
            Self::Constant(s) => Arc::new(Const(s)),
            Self::Dynamic(s) => s,
        }
    }

    /// Attach a finite duration, returning a dynamic timeline whose
    /// [`length`](Timeline::length) reports `length`. Sampling is unchanged —
    /// only the reported duration is added — so this works on a constant too.
    pub fn with_length(self, length: f64) -> Timeline<T> {
        Self::Dynamic(Arc::new(WithLength {
            inner: self.into_signal(),
            length,
        }))
    }
}

/// A constant lifted into a [`Signal`], so a [`Timeline::Constant`] can be
/// wrapped by signal adaptors. Ignores `t` and clones its value.
struct Const<T>(T);
impl<T: Clone + 'static> Signal for Const<T> {
    type Output = T;
    fn sample(&self, _t: f64) -> Self::Output {
        self.0.clone()
    }
}

/// A signal adaptor that forwards sampling to `inner` but overrides its
/// reported duration. Built by [`Timeline::with_length`] and immediately erased
/// into a [`Timeline::Dynamic`], so it stays private like [`Const`].
struct WithLength<T: 'static> {
    inner: Arc<dyn Signal<Output = T>>,
    length: f64,
}

impl<T: 'static> Signal for WithLength<T> {
    type Output = T;
    fn sample(&self, t: f64) -> Self::Output {
        self.inner.sample(t)
    }
    fn length(&self) -> Option<f64> {
        Some(self.length)
    }
}

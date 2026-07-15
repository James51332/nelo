//! Timelines are the are a deterministic stream of data over time.
//!
//! They have two flavors:
//! * [`Constant`](Timeline::Constant) — a value that never changes, held inline
//!   with no allocation and no dynamic dispatch.
//! * [`Dynamic`](Timeline::Dynamic) — a real [`Signal`], type-erased behind a
//!   `Box` so a timeline stays `Clone` and can be shared cheaply.
//!
//! Callers sample both through the same [`sample`](Timeline::sample) call; the
//! constant case is purely an optimisation, not a separate API.

pub mod compose;
pub mod from;
pub mod keyframe;
pub mod signal;

pub use keyframe::{Easing, Lerp};
pub use signal::Signal;

/// A sampleable value over time, either a fixed constant or a shared [`Signal`].
pub enum Timeline<T> {
    Constant(T),
    Dynamic(Box<dyn Signal<Output = T>>),
}

impl<T: 'static> Timeline<T> {
    /// Wrap a fixed value that never changes.
    pub fn constant(s: T) -> Self {
        Self::Constant(s)
    }

    /// Erase a concrete signal (e.g. a closure) into a shareable timeline.
    pub fn dynamic(s: impl Signal<Output = T> + 'static) -> Self {
        Self::Dynamic(Box::new(s))
    }

    /// The timeline's finite duration, or `None` if it runs forever. A constant
    /// has no inherent end; a dynamic one defers to its [`Signal::length`].
    pub fn length(&self) -> Option<f32> {
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
    pub fn sample(&self, t: f32) -> T {
        match self {
            Self::Constant(s) => s.clone(),
            Self::Dynamic(s) => s.sample(t),
        }
    }

    /// Attach a finite duration, creating a new timeline whose
    /// [`length`](Timeline::length) reports `length`.
    pub fn with_length(self, length: f32) -> Self {
        Self::dynamic(WithLength {
            timeline: self,
            length,
        })
    }
}

/// A signal adaptor that forwards sampling to `inner` but overrides its
/// reported duration. Built by [`Timeline::with_length`] and immediately erased
/// into a [`Timeline::Dynamic`].
struct WithLength<T: Clone + 'static> {
    timeline: Timeline<T>,
    length: f32,
}

impl<T: Clone + 'static> Signal for WithLength<T> {
    type Output = T;

    fn sample(&self, t: f32) -> Self::Output {
        self.timeline.sample(t)
    }

    fn length(&self) -> Option<f32> {
        Some(self.length)
    }
}

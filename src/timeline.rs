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

pub mod along;
pub mod compose;
pub mod easing;
pub mod float;
pub mod from;
pub mod keyframe;
pub mod path;
pub mod signal;

pub use along::{Along, TimelineAlong, TimelineSpline};
pub use easing::Easing;
pub use keyframe::Lerp;
pub use path::{Path, PathBuilder};
pub use signal::{Signal, SignalClone};

// ----- Timeline -----

/// A sampleable value over time, either a fixed constant or a shared [`Signal`].
pub enum Timeline<T: Clone + 'static> {
    Constant(T),
    Dynamic(Box<dyn SignalClone<Output = T>>),
}

impl<T: Clone> Timeline<T> {
    /// Wrap a fixed value that never changes.
    pub fn constant(s: T) -> Self {
        Self::Constant(s)
    }

    /// Erase a concrete signal (e.g. a closure) into a shareable timeline.
    pub fn dynamic(s: impl SignalClone<Output = T>) -> Self {
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

    /// Returns true iff this timeline is constant wrt time.
    pub fn is_constant(&self) -> bool {
        matches!(self, Self::Constant(..))
    }

    /// Repeats over this timelines length, or does nothing if this timeline
    /// has no length.
    pub fn repeat(self) -> Self {
        let Some(length) = self.length() else {
            return self;
        };

        self.compose(move |t: f32| t.rem_euclid(length))
    }
}

impl<T: Clone + 'static> Clone for Timeline<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Constant(s) => Self::Constant(s.clone()),
            Self::Dynamic(s) => Self::Dynamic(s.clone_box()),
        }
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

/// A signal adaptor that forwards sampling to `inner` but overrides its
/// reported duration. Built by [`Timeline::with_length`] and immediately erased
/// into a [`Timeline::Dynamic`].
#[derive(Clone)]
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

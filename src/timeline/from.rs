//! Helper traits to allow types to be converted into timelines.
use crate::timeline::{Along, Easing, Timeline, TimelineAlong, TimelineSpline};
use glam::prelude::*;

// ----- Timeline -----

/// Closures implement signal automatically, so they can become dynamic timelines.
impl<T: Clone, F: Fn(f32) -> T + Clone + 'static> From<F> for Timeline<T> {
    fn from(f: F) -> Self {
        Timeline::dynamic(f)
    }
}

macro_rules! timeline_from {
    ($($t:ty),*) => {
        $(impl From<$t> for Timeline<$t> {
            fn from(t: $t) -> Self {
                Timeline::constant(t)
            }
        })*
    };
}

timeline_from!(bool, f32, i32, u32, usize, Vec2, Vec3, Vec4, Mat2, Affine2);

impl<T: Clone> From<Along<T>> for Timeline<T> {
    fn from(t: Along<T>) -> Self {
        t.timeline()
    }
}

impl From<Easing> for Timeline<f32> {
    fn from(easing: Easing) -> Self {
        Timeline::dynamic(easing)
    }
}

impl<T: Clone> From<Along<T>> for Timeline<Along<T>> {
    fn from(t: Along<T>) -> Self {
        Timeline::constant(t)
    }
}

// ----- Along -----

impl<T: Clone, F: Fn(f32) -> T + Clone + 'static> From<F> for Along<T> {
    /// Alongs can also be generated from parameters. This isn't used by our public API since
    /// we always want to enable the value to change over time, so we use `TimelineAlong<T>`.
    /// This adds one more layer of indirection.
    fn from(f: F) -> Self {
        Timeline::dynamic(f).along()
    }
}

macro_rules! along_from {
    ($($t:ty),*) => {
        $(impl From<$t> for Along<$t> {
            fn from(t: $t) -> Self {
                Timeline::constant(t).along()
            }
        })*
    };
}

along_from!(f32, i32, u32, usize, Vec2, Vec3, Vec4, Mat2, Affine2);

impl<T: Clone> From<Timeline<T>> for Along<T> {
    fn from(t: Timeline<T>) -> Self {
        t.along()
    }
}

// ----- TimelineAlong -----

impl<T: Clone, F: Fn(f32, f32) -> T + Clone + 'static> From<F> for TimelineAlong<T> {
    /// Takes a closure over time and alpha and converts it to a timeline.
    fn from(f: F) -> Self {
        Self::new(Timeline::dynamic(move |t| {
            let inner = f.clone();
            Timeline::dynamic(move |a| inner(t, a)).along()
        }))
    }
}

macro_rules! timeline_along_from {
    ($($t:ty),*) => {
        $(impl From<$t> for TimelineAlong<$t> {
            fn from(t: $t) -> Self {
                Self::new(Timeline::constant(t).along().into())
            }
        })*
    };
}

timeline_along_from!(f32, Vec2, Vec3, Vec4, Mat2, Affine2);

impl<T: Clone> From<Timeline<T>> for TimelineAlong<T> {
    /// Converts a `Timeline<T>` to a time-varying TimelineAlong whose value
    /// is uniform along the curve.
    fn from(t: Timeline<T>) -> Self {
        Self::new(t.map(|v| Timeline::constant(v).along()))
    }
}

impl<T: Clone> From<Timeline<Along<T>>> for TimelineAlong<T> {
    fn from(s: Timeline<Along<T>>) -> Self {
        Self::new(s)
    }
}

impl<T: Clone> From<Timeline<Timeline<T>>> for TimelineAlong<T> {
    fn from(s: Timeline<Timeline<T>>) -> Self {
        Self::new(s.map(|p| p.along()))
    }
}

impl<T: Clone> From<Along<T>> for TimelineAlong<T> {
    fn from(along: Along<T>) -> Self {
        Self::new(Timeline::constant(along))
    }
}

// ----- TimelineSpline -----

impl<F: Fn(f32, f32) -> Vec2 + Clone + 'static> From<F> for TimelineSpline {
    fn from(f: F) -> Self {
        Self::new(f.into())
    }
}

impl From<Timeline<Vec2>> for TimelineSpline {
    /// Converts a `Timeline<Vec2>` to a constant spline.
    ///
    /// The big difference from TimelineAlong is that timeline along assumes
    /// that a timeline defines the time dependent value (e.g. color varying
    /// with time). TimelineSpline merits the one exception.
    fn from(s: Timeline<Vec2>) -> Self {
        Self::new(s.along().into())
    }
}

impl From<Timeline<Along<Vec2>>> for TimelineSpline {
    fn from(s: Timeline<Along<Vec2>>) -> Self {
        Self::new(s.into())
    }
}

impl From<Timeline<Timeline<Vec2>>> for TimelineSpline {
    fn from(s: Timeline<Timeline<Vec2>>) -> Self {
        Self::new(s.map(|p| p.along()).into())
    }
}

impl From<Along<Vec2>> for TimelineSpline {
    fn from(s: Along<Vec2>) -> Self {
        Self::new(s.into())
    }
}

//! Implementation for creating timelines from keyframes.

use crate::timeline::{Along, Easing, Timeline, signal::Signal};
use glam::{Mat2, Quat, Vec2, Vec3, Vec4};

// ----- Keyframe -----

/// Keyframes are a convenient way to create a `Timeline`.
/// They define state at a fixed point in time for a timeline.
/// Anytype which wishes to be part of a keyframe must implement
/// the `Lerp` trait.
#[derive(Clone)]
struct Keyframe<T: Lerp + Clone + 'static> {
    time: f32,
    value: T,
    easing: Easing,
}

/// Keyframes struct implements Signal trait. We require that the keyframes
/// are sorted by their end time. This struct is a private, intermediate
/// which implements Signal so it can be converted into a timeline.
#[derive(Clone)]
struct Keyframes<T: Lerp + Clone + 'static>(Vec<Keyframe<T>>);

impl<T: Lerp + Clone> Signal for Keyframes<T> {
    type Output = T;

    fn sample(&self, t: f32) -> T {
        // Number of keyframes at or before `t`. Because the list is sorted,
        // this is also the index of the first keyframe strictly after `t`, so
        // `[i - 1, i]` is the pair that brackets `t`.
        let i = self.0.partition_point(|k| k.time <= t);

        // Before the first keyframe: hold the first value.
        if i == 0 {
            return self.0[0].value.clone();
        }

        // At or after the last keyframe: hold the last value.
        if i == self.0.len() {
            return self.0[i - 1].value.clone();
        }

        // Interpolate across the bracket. Previous is <= t, and next is > t, so
        // the denominator is strictly positive (no divide-by-zero). Easing is
        // read from the next keyframe.
        let prev = self.0[i - 1].clone();
        let next = self.0[i].clone();
        let progress = (t - prev.time) / (next.time - prev.time);
        T::interpolate(prev.value, next.value, next.easing.sample(progress))
    }

    fn length(&self) -> Option<f32> {
        self.0.last().map(|t| t.time)
    }
}

// ----- KeyframeBuilder -----

/// Keyframe builder is used to build a keyframe that meets the requirements for the
/// API. Therefore, this is the only way to access the API.
pub struct KeyframeBuilder<T: Lerp + Clone + 'static> {
    frames: Vec<Keyframe<T>>,
}

impl<T: Lerp + Clone> KeyframeBuilder<T> {
    fn new(anchor: T) -> Self {
        Self {
            frames: vec![Keyframe {
                time: 0.0,
                value: anchor,
                easing: Easing::Step,
            }],
        }
    }

    pub fn at(self, time: f32, value: T) -> Self {
        self.ease_at(time, value, Easing::Linear)
    }

    pub fn step_at(self, time: f32, value: T) -> Self {
        self.ease_at(time, value, Easing::Step)
    }

    pub fn ease_at(mut self, time: f32, value: T, easing: Easing) -> Self {
        self.frames.push(Keyframe {
            time,
            value,
            easing,
        });

        self
    }

    pub fn build(mut self) -> Timeline<T> {
        self.frames.sort_by(|a, b| a.time.total_cmp(&b.time));

        // The length is the time of the last keyframe.
        let length = self.frames.last().map(|x| x.time);
        let timeline = Timeline::dynamic(Keyframes(self.frames));

        // Attach a length if we have one. Useful for repeating.
        if let Some(len) = length {
            if len >= 0.0 {
                return timeline.with_length(len);
            }
        }

        timeline
    }
}

// ----- Timeline -----

impl<T: Clone + Lerp> Timeline<T> {
    /// Use this method to create a timeline where you know the value
    /// at a fixed number of points in time. These can be specified
    /// directly using the builder's `at` and `ease_at` methods. Requires
    /// `T` to implement `Lerp` trait, which is enabled for some of the
    /// common types within the engine.
    pub fn keyframes(anchor: T) -> KeyframeBuilder<T> {
        KeyframeBuilder::new(anchor)
    }
}

// ----- Lerp -----

/// All types that wish to use the keyframe system
/// must implement the lerp trait.
pub trait Lerp {
    fn interpolate(a: Self, b: Self, t: f32) -> Self;
}

macro_rules! impl_lerp {
    ($($t:ty),*) => { $(
        impl Lerp for $t {
            fn interpolate(a: Self, b: Self, t: f32) -> Self { a + (b - a) * t }
        }
    )* };
}
impl_lerp!(f32, Vec2, Vec3, Vec4, Mat2);

impl Lerp for Quat {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        a.slerp(b, t)
    }
}

impl<T: Clone + Lerp> Lerp for Timeline<T> {
    fn interpolate(a: Self, b: Self, progress: f32) -> Self {
        Timeline::dynamic(move |t: f32| T::interpolate(a.sample(t), b.sample(t), progress))
    }
}

impl<T: Clone + Lerp> Lerp for Along<T> {
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        Timeline::interpolate(a.timeline(), b.timeline(), t).along()
    }
}

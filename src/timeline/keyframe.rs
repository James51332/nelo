use std::f32::consts::PI;

///! Implementation for creating timelines from keyframes.
use crate::timeline::{Timeline, signal::Signal};
use glam::{Quat, Vec2, Vec3, Vec4};

/// All types that wish to use the keyframe system
/// must implement the lerp trait.
pub trait Lerp {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self;
}

macro_rules! impl_lerp {
    ($($t:ty),*) => { $(
        impl Lerp for $t {
            fn interpolate(a: &Self, b: &Self, t: f32) -> Self { a + (b - a) * t }
        }
    )* };
}
impl_lerp!(f32, Vec2, Vec3, Vec4);

impl Lerp for Quat {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a.clone().slerp(b.clone(), t)
    }
}

impl<T: Lerp + Clone + 'static> Lerp for Timeline<T> {
    fn interpolate(a: &Self, b: &Self, alpha: f32) -> Self {
        let (a, b) = (a.clone(), b.clone());
        Timeline::dynamic(move |t: f32| T::interpolate(&a.sample(t), &b.sample(t), alpha))
    }
}

/// Easing functions are simply maps from `f32` -> `f32`
/// Use the [`sample`](Easing::sample) method to manually
/// map a linear time variable from [0, 1] -> [0, 1]. Only
/// a handful are implemented right now, but all of penner's
/// will be implemented soon.
#[derive(Clone)]
pub enum Easing {
    Step,
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SineIn,
    SineOut,
    SineInOut,
}

impl Signal for Easing {
    type Output = f32;

    /// Implementation for penner's easing function.
    ///
    /// Based on [easings.net](https://easings.net/)
    fn sample(&self, t: f32) -> Self::Output {
        let base = t.floor();
        let t = t - base;
        match self {
            Self::Step => 0.0,
            Self::Linear => base + t,
            Self::QuadIn => base + t * t,
            Self::QuadOut => base + 1.0 - (1.0 - t) * (1.0 - t),
            Self::QuadInOut => {
                if t <= 0.5 {
                    base + 2.0 * t * t
                } else {
                    base + 1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
            Self::CubicIn => t.powi(3),
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut => {
                if t <= 0.5 {
                    base + 4.0 * t * t * t
                } else {
                    base + 1.0 - 4.0 * (1.0 - t).powi(3)
                }
            }
            Self::SineIn => base + (0.5 * t * PI).sin(),
            Self::SineOut => base + 1.0 - (0.5 * t * PI).cos(),
            Self::SineInOut => base + 1.0 - 0.5 * (t * PI).cos(),
        }
    }
}

impl Into<Timeline<f32>> for Easing {
    fn into(self) -> Timeline<f32> {
        Timeline::dynamic(self)
    }
}

/// Keyframes are a convenient way to create a `Timeline`.
/// They define state at a fixed point in time for a timeline.
/// Anytype which wishes to be part of a keyframe must implement
/// the `Lerp` trait.
#[derive(Clone)]
struct Keyframe<T: Lerp + Clone> {
    time: f32,
    value: T,
    easing: Easing,
}

/// Keyframes struct implements Signal trait. We require that the keyframes
/// are sorted by their end time. This struct is a private, intermediate
/// which implements Signal so it can be converted into a timeline.
#[derive(Clone)]
struct Keyframes<T: Lerp + Clone>(Vec<Keyframe<T>>);

impl<T: Lerp + Clone + 'static> Signal for Keyframes<T> {
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
        let prev = &self.0[i - 1];
        let next = &self.0[i];
        let progress = (t - prev.time) / (next.time - prev.time);
        T::interpolate(&prev.value, &next.value, next.easing.sample(progress))
    }

    fn length(&self) -> Option<f32> {
        self.0.last().map(|t| t.time)
    }
}

/// Keyframe builder is used to build a keyframe that meets the requirements for the
/// API. Therefore, this is the only way to access the API.
pub struct KeyframeBuilder<T: Lerp + Clone + 'static> {
    frames: Vec<Keyframe<T>>,
}

impl<T: Lerp + Clone + 'static> KeyframeBuilder<T> {
    fn new(anchor: T) -> Self {
        Self {
            frames: vec![Keyframe {
                time: 0.0,
                value: anchor,
                easing: Easing::Step,
            }],
        }
    }

    pub fn at(mut self, time: f32, value: T) -> Self {
        self.frames.push(Keyframe {
            time,
            value,
            easing: Easing::Linear,
        });

        self
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

impl<T: Lerp + Clone + 'static> Timeline<T> {
    /// Use this method to create a timeline where you know the value
    /// at a fixed number of points in time. These can be specified
    /// directly using the builder's `at` and `ease_at` methods. Requires
    /// `T` to implement `Lerp` trait, which is enabled for some of the
    /// common types within the engine.
    pub fn keyframes(anchor: T) -> KeyframeBuilder<T> {
        KeyframeBuilder::new(anchor)
    }
}

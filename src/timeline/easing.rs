//! Easing functions

use std::f32::consts::PI;

use crate::timeline::Signal;

/// Easing functions are simply maps from `f32` -> `f32`
/// Use the [`sample`](Easing::sample) method to manually
/// map a linear time variable from [0, 1] -> [0, 1]. Only
/// a handful are implemented right now, but all of penner's
/// will be implemented soon.
#[derive(Debug, Clone, Copy)]
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

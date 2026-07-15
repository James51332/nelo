//! Helper traits to allow types to be converted into timelines.
use crate::timeline::Timeline;
use glam::prelude::*;

/// Closures implement signal automatically, so they can become dynamic timelines.
impl<T: 'static, F: Fn(f32) -> T + 'static> From<F> for Timeline<T> {
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

timeline_from!(f32, Vec2, Vec3, Vec4, Mat2, Affine2);

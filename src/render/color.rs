//! Colors are stored as sRgb values, but interpolation is in oklch.

use crate::timeline::{Lerp, Timeline};
use palette::{IntoColor, Mix, Oklcha, Srgba};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    srgba: Srgba<f32>,
}

impl Color {
    pub const WHITE: Self = Color::srgba(1.0, 1.0, 1.0, 1.0);

    pub const fn srgb(r: f32, g: f32, b: f32) -> Self {
        Self::srgba(r, g, b, 1.0)
    }

    /// Creates a new color using srgba encoding.
    pub const fn srgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            srgba: Srgba::new(r, g, b, a),
        }
    }

    pub fn lch(l: f32, c: f32, h: f32) -> Self {
        Self::lcha(l, c, h, 1.0)
    }

    pub fn lcha(l: f32, c: f32, h: f32, a: f32) -> Self {
        Self {
            srgba: Oklcha::new(l, c, h, a).into_color(),
        }
    }

    /// Creates an srgb(a) color from the given slice. Panics if length < 3.
    pub fn from_slice(slice: &[f32]) -> Self {
        if slice.len() < 3 {
            panic!("Slice is too short to create a new color");
        }

        let alpha = slice.get(3).copied().unwrap_or(1.0);
        Self::srgba(slice[0], slice[1], slice[2], alpha)
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    /// Converts the components of this color into an s3
    pub fn to_array(&self) -> [f32; 4] {
        let (r, g, b, a) = self.srgba.into_components();
        [r, g, b, a]
    }
}

impl Deref for Color {
    type Target = Srgba<f32>;

    fn deref(&self) -> &Self::Target {
        &self.srgba
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.srgba
    }
}

impl<T: IntoColor<Srgba>> From<T> for Color {
    fn from(t: T) -> Self {
        Self {
            srgba: t.into_color(),
        }
    }
}

impl Into<Timeline<Color>> for Color {
    fn into(self) -> Timeline<Color> {
        Timeline::constant(self)
    }
}

impl Lerp for Color {
    /// Interpolates color in oklcha color space.
    fn interpolate(a: Self, b: Self, t: f32) -> Self {
        let a: Oklcha = a.srgba.into_color();
        let b: Oklcha = b.srgba.into_color();
        Self {
            srgba: a.mix(b, t).into_color(),
        }
    }
}

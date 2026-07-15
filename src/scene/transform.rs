//! The `Transform` captures various transformations to an entity.
//!
//! Entities will have a list of transformations which are all
//! composed on top of one another. This will enable more sophisticated
//! hierarchies and entity grouping.
use crate::timeline::Timeline;
use glam::prelude::*;

/// Captures different transformations which could be applied to [`Entity`].
pub enum Transform {
    Matrix(Timeline<Mat2>),
    Affine(Timeline<Affine2>),
    Translate(Timeline<Vec2>),

    /// Scale is defined in one dimension. Directional scaling is done using Matrix variant.
    Scale(Timeline<f32>),

    /// Rotation is defined in radians CCW about the world origin.
    Rotate(Timeline<f32>),
}

impl Transform {
    /// Returns an equivalent affine2 transform  
    pub fn sample(&self, t: f32) -> Affine2 {
        match self {
            Self::Matrix(timeline) => Affine2::from_mat2(timeline.sample(t)),
            Self::Affine(timeline) => timeline.sample(t),
            Self::Translate(timeline) => Affine2::from_translation(timeline.sample(t)),
            Self::Scale(timeline) => Affine2::from_scale(Vec2::splat(timeline.sample(t))),
            Self::Rotate(timeline) => Affine2::from_angle(timeline.sample(t)),
        }
    }
}

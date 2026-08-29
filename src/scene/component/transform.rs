//! A Transform is a list of transformations which are all composed on top of
//! one another.

use crate::timeline::Timeline;
use glam::prelude::*;

// ----- Step -----

/// Captures different transformations which could be applied to an entity.
#[derive(Clone)]
pub enum Step {
    Matrix(Timeline<Mat2>),
    Translate(Timeline<Vec2>),

    /// Scale is defined in one dimension. Directional scaling is done using Matrix variant.
    Scale(Timeline<f32>),

    /// Rotation is defined in radians CCW about the world origin.
    Rotate(Timeline<f32>),
}

impl Step {
    /// Returns an equivalent affine2 transform  
    pub fn sample(&self, t: f32) -> Affine2 {
        match self {
            Self::Matrix(timeline) => Affine2::from_mat2(timeline.sample(t)),
            Self::Translate(timeline) => Affine2::from_translation(timeline.sample(t)),
            Self::Scale(timeline) => Affine2::from_scale(Vec2::splat(timeline.sample(t))),
            Self::Rotate(timeline) => Affine2::from_angle(timeline.sample(t)),
        }
    }
}

// ----- Transform -----

/// A transform is a sequence of `Step`s and an optional parent. It is lightweight and
/// fast to pass around.
#[derive(Clone, Default)]
pub struct Transform {
    steps: Vec<Step>,
}

impl Transform {
    /// Returns the local transform at a given time.
    pub fn local(&self, t: f32) -> Affine2 {
        let mut cur = Affine2::default();

        for step in self.steps.iter() {
            cur = step.sample(t) * cur;
        }

        cur
    }

    /// Adds a single step of transform to this object.
    pub fn push(&mut self, step: Step) {
        self.steps.push(step)
    }

    /// Resets this to identity transform.
    pub fn reset(&mut self) {
        self.steps.clear();
    }
}

impl Transformable for Transform {
    fn transform(&mut self) -> &mut Self {
        self
    }
}

// ----- Transformable -----

pub trait Transformable: Sized {
    fn transform(&mut self) -> &mut Transform;

    /// Applies a matrix transformation to this entity, enabling any linear transformation
    /// in 2D. This can be used for rotation, shearing, scaling (dimensionally independent),
    /// or any combination thereof.
    fn matrix(mut self, matrix: impl Into<Timeline<Mat2>>) -> Self {
        self.transform().push(Step::Matrix(matrix.into()));
        self
    }

    /// Applies a translation to this entity.
    fn translate(mut self, trans: impl Into<Timeline<Vec2>>) -> Self {
        self.transform().push(Step::Translate(trans.into()));
        self
    }

    /// Applies a one dimensional scale to this entity in world space. This should be done
    /// before other transformations, as the scale will affect all previous translations.
    fn scale(mut self, scalar: impl Into<Timeline<f32>>) -> Self {
        self.transform().push(Step::Scale(scalar.into()));
        self
    }

    /// Applies a CCW rotation for an angle in radians around the world origin.
    fn rotate(mut self, radians: impl Into<Timeline<f32>>) -> Self {
        self.transform().push(Step::Rotate(radians.into()));
        self
    }
}

//! The `Transform` captures various transformations to an entity.
//!
//! Entities will have a list of transformations which are all
//! composed on top of one another. This will enable more sophisticated
//! hierarchies and entity grouping.
use crate::timeline::Timeline;
use glam::prelude::*;

/// Captures different transformations which could be applied to an entity.
pub enum Step {
    Matrix(Timeline<Mat2>),
    Affine(Timeline<Affine2>),
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
            Self::Affine(timeline) => timeline.sample(t),
            Self::Translate(timeline) => Affine2::from_translation(timeline.sample(t)),
            Self::Scale(timeline) => Affine2::from_scale(Vec2::splat(timeline.sample(t))),
            Self::Rotate(timeline) => Affine2::from_angle(timeline.sample(t)),
        }
    }
}

/// A transform is a sequence of `Step`s.
#[derive(Default)]
pub struct Transform(Vec<Step>);

impl Transform {
    /// Consolidates all steps of this transform into a single affine transformation.
    pub fn sample(&self, t: f32) -> Affine2 {
        let mut cur = Affine2::default();
        for step in self.0.iter() {
            cur = step.sample(t) * cur;
        }
        cur
    }
}

impl Transformable for Transform {
    fn transform(&mut self) -> &mut Self {
        self
    }
}

pub trait Transformable: Sized {
    fn transform<'a>(&'a mut self) -> &'a mut Transform;

    /// Lists out the steps of this transform.
    fn steps(&mut self) -> &mut Vec<Step> {
        &mut self.transform().0
    }

    /// Applies a matrix transformation to this entity, enabling any linear transformation
    /// in 2D. This can be used for rotation, shearing, scaling (dimensionally independent),
    /// or any combination thereof.
    fn matrix(mut self, matrix: impl Into<Timeline<Mat2>>) -> Self {
        self.steps().push(Step::Matrix(matrix.into()));
        self
    }

    /// Applies an affine transform (matrix + translation) to this entity. Equivalent to calling
    /// [`matrix`](EntityBuilder::matrix) and then [`translate`](EntityBuilder::translate) on this
    /// object, But is more convenient when you already have an Affine timeline.
    fn affine(mut self, transform: impl Into<Timeline<Affine2>>) -> Self {
        self.steps().push(Step::Affine(transform.into()));
        self
    }

    /// Applies a translation to this entity.
    fn translate(mut self, trans: impl Into<Timeline<Vec2>>) -> Self {
        self.steps().push(Step::Translate(trans.into()));
        self
    }

    /// Applies a one dimensional scale to this entity in world space. This should be done
    /// before other transformations, as the scale will affect all previous translations.
    fn scale(mut self, scalar: impl Into<Timeline<f32>>) -> Self {
        self.steps().push(Step::Scale(scalar.into()));
        self
    }

    /// Applies a CCW rotation for an angle in radians around the world origin.
    fn rotate(mut self, radians: impl Into<Timeline<f32>>) -> Self {
        self.steps().push(Step::Rotate(radians.into()));
        self
    }
}

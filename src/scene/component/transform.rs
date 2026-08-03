//! The `Transform` captures various transformations to an entity.
//!
//! Entities will have a list of transformations which are all
//! composed on top of one another. This will enable more sophisticated
//! hierarchies and entity grouping.
use crate::timeline::Timeline;
use glam::prelude::*;
use std::sync::{Arc, Mutex, Weak};

// ----- Step -----

/// Captures different transformations which could be applied to an entity.
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
    inner: Arc<Mutex<Inner>>,
}

impl Transform {
    /// Consolidates this transform and all it's parents steps into a single
    /// affine transform.
    pub fn sample(&self, t: f32) -> Affine2 {
        self.inner.lock().unwrap().sample(t)
    }

    /// Adds a single step of transform to this object.
    pub fn push(&mut self, step: Step) {
        self.inner.lock().unwrap().steps.push(step)
    }

    /// Resets this to identity transform.
    pub fn reset(&mut self) {
        self.inner.lock().unwrap().steps.clear();
    }

    /// Translates this to `position`.
    pub fn to(&mut self, position: impl Into<Timeline<Vec2>>) {
        // Remove all translations.
        let steps = &mut self.inner.lock().unwrap().steps;
        steps.retain(|x| !matches!(x, Step::Translate(_)));

        // Add our new translation.
        steps.push(Step::Translate(position.into()));
    }

    /// Determines if this transform is an ancestor of `child`.
    pub fn is_ancestor(&self, child: &Transform) -> bool {
        let mut cursor = child.inner.clone();

        loop {
            // See if we are equal to the cursor.
            if Arc::ptr_eq(&self.inner, &cursor) {
                return true;
            }

            // Go to the child's next parent.
            let parent = cursor
                .lock()
                .unwrap()
                .parent
                .as_ref()
                .and_then(Weak::upgrade);

            // If the child doesn't have a parent, we must not be a parent.
            match parent {
                Some(x) => cursor = x,
                None => return false,
            }
        }
    }

    /// Sets the parent for this transform unless `self` is an ancestor of parent
    ///
    /// Returns Ok(()) if the parent was successfully updated, and Err(()) if the `self`
    /// is an ancestor of `parent`.
    pub fn parent(&mut self, parent: Option<Transform>) -> Result<(), ()> {
        // Remove the parent if we provide none.
        let Some(transform) = parent else {
            self.inner.lock().unwrap().parent = None;
            return Ok(());
        };

        // Check if we can safely add the parent.
        if !self.is_ancestor(&transform) {
            self.inner.lock().unwrap().parent = Some(Arc::downgrade(&transform.inner));
            Ok(())
        } else {
            Err(())
        }
    }

    /// Returns true if this transform has a parent.
    pub fn has_parent(&self) -> bool {
        self.inner
            .lock()
            .unwrap()
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some()
    }
}

impl Transformable for Transform {
    fn transform(&mut self) -> &mut Self {
        self
    }
}

// ----- Inner -----

#[derive(Default)]
pub(crate) struct Inner {
    pub steps: Vec<Step>,
    pub parent: Option<Weak<Mutex<Inner>>>,
}

impl Inner {
    /// Consolidates all steps of this transform into a single affine transformation.
    pub fn sample(&self, t: f32) -> Affine2 {
        let mut cur = Affine2::default();

        // Apply the local transform first.
        for step in self.steps.iter() {
            cur = step.sample(t) * cur;
        }

        // Then apply the parent transform.
        if let Some(ref parent) = self.parent {
            if let Some(ref arc) = parent.upgrade() {
                cur = arc.lock().unwrap().sample(t) * cur;
            }
        }

        cur
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

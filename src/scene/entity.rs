//! An `Entity` is simple an id representing rendering data.

use crate::render::renderer::{Geometry, Primitive};
use crate::scene::{Scene, Transform};
use crate::timeline::Timeline;
use glam::prelude::*;

#[allow(unused)]
pub struct Entity(usize);

/// This struct essentially is a timeline version of renderable. It
/// usually makes sense to keep this data packed because in rendering
/// we are accessing all fields.
pub(crate) struct EntityData {
    pub transforms: Vec<Transform>,
    pub geometry: Option<Geometry>,
    pub fill: Option<Timeline<Vec4>>,
}

// The tricky thing to decide for this API is what the best way to handle
// transformation is. We will build into an affine, but we want to support
// more convenient shorthands. The approach currently used is that we'll
// store subfields, then aggregate.
pub struct EntityBuilder<'a> {
    store: &'a mut Vec<EntityData>,
    transforms: Vec<Transform>,
    geometry: Option<Geometry>,
    fill: Option<Timeline<Vec4>>,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(store: &'a mut Vec<EntityData>, geometry: Option<Geometry>) -> Self {
        EntityBuilder {
            store,
            transforms: Vec::new(),
            geometry,
            fill: None,
        }
    }

    /// Applies a matrix transformation to this entity, enabling any linear transformation
    /// in 2D. This can be used for rotation, shearing, scaling (dimensionally independent),
    /// or any combination thereof.
    pub fn matrix(mut self, matrix: impl Into<Timeline<Mat2>>) -> Self {
        self.transforms.push(Transform::Matrix(matrix.into()));
        self
    }

    /// Applies an affine transform (matrix + translation) to this entity. Equivalent to calling
    /// [`matrix`](EntityBuilder::matrix) and then [`translate`](EntityBuilder::translate) on this
    /// object, But is more convenient when you already have an Affine timeline.
    pub fn affine(mut self, transform: impl Into<Timeline<Affine2>>) -> Self {
        self.transforms.push(Transform::Affine(transform.into()));
        self
    }

    /// Applies a translation to this entity.
    pub fn translate(mut self, trans: impl Into<Timeline<Vec2>>) -> Self {
        self.transforms.push(Transform::Translate(trans.into()));
        self
    }

    /// Applies a one dimensional scale to this entity in world space. This should be done
    /// before other transformations, as the scale will affect all previous translations.
    pub fn scale(mut self, scalar: impl Into<Timeline<f32>>) -> Self {
        self.transforms.push(Transform::Scale(scalar.into()));
        self
    }

    /// Applies a CCW rotation for an angle in radians around the world origin.
    pub fn rotate(mut self, radians: impl Into<Timeline<f32>>) -> Self {
        self.transforms.push(Transform::Rotate(radians.into()));
        self
    }

    // Sets the fill for this entity.
    pub fn fill(mut self, fill: impl Into<Timeline<Vec4>>) -> Self {
        self.fill = Some(fill.into());
        self
    }

    pub fn build(self) -> Entity {
        self.store.push(EntityData {
            transforms: self.transforms,
            geometry: self.geometry,
            fill: self.fill,
        });
        Entity(self.store.len() - 1)
    }
}

impl Scene {
    /// Returns an [`EntityBuilder`] with circle geometry attached. The default
    /// circle is at the world origin with a radius of one.
    pub fn circle(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(
            &mut self.store,
            Some(Geometry::Primitive(Primitive::Circle)),
        )
    }
}

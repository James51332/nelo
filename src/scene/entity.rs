//! An `Entity` is simple an id representing rendering data.

use crate::render::Primitive;
use crate::render::renderer::Geometry;
use crate::scene::{Scene, Transform};
use crate::timeline::Timeline;
use glam::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct EntityId(usize);

/// This struct essentially is a timeline version of renderable. It
/// usually makes sense to keep this data packed because in rendering
/// we are accessing all fields.
pub(crate) struct EntityData {
    pub transforms: Vec<Transform>,
    pub geometry: Geometry,
    pub fill: Option<Timeline<Vec4>>,
}

impl EntityData {
    fn new(geometry: Geometry) -> Self {
        Self {
            transforms: Vec::new(),
            geometry,
            fill: None,
        }
    }
}

/// A reference to change an entity.
pub struct EntityRef<'a>(&'a mut EntityData, EntityId);

impl<'a> EntityRef<'a> {
    /// Applies a matrix transformation to this entity, enabling any linear transformation
    /// in 2D. This can be used for rotation, shearing, scaling (dimensionally independent),
    /// or any combination thereof.
    pub fn matrix(self, matrix: impl Into<Timeline<Mat2>>) -> Self {
        self.0.transforms.push(Transform::Matrix(matrix.into()));
        self
    }

    /// Applies an affine transform (matrix + translation) to this entity. Equivalent to calling
    /// [`matrix`](EntityBuilder::matrix) and then [`translate`](EntityBuilder::translate) on this
    /// object, But is more convenient when you already have an Affine timeline.
    pub fn affine(self, transform: impl Into<Timeline<Affine2>>) -> Self {
        self.0.transforms.push(Transform::Affine(transform.into()));
        self
    }

    /// Applies a translation to this entity.
    pub fn translate(self, trans: impl Into<Timeline<Vec2>>) -> Self {
        self.0.transforms.push(Transform::Translate(trans.into()));
        self
    }

    /// Applies a one dimensional scale to this entity in world space. This should be done
    /// before other transformations, as the scale will affect all previous translations.
    pub fn scale(self, scalar: impl Into<Timeline<f32>>) -> Self {
        self.0.transforms.push(Transform::Scale(scalar.into()));
        self
    }

    /// Applies a CCW rotation for an angle in radians around the world origin.
    pub fn rotate(self, radians: impl Into<Timeline<f32>>) -> Self {
        self.0.transforms.push(Transform::Rotate(radians.into()));
        self
    }

    // Sets the fill for this entity.
    pub fn fill(self, fill: impl Into<Timeline<Vec4>>) -> Self {
        self.0.fill = Some(fill.into());
        self
    }

    pub fn id(self) -> EntityId {
        self.1
    }
}

pub(crate) struct Store {
    entity_data: HashMap<EntityId, EntityData>,
    next_key: usize,
}

impl Store {
    pub fn new() -> Self {
        Self {
            entity_data: HashMap::new(),
            next_key: 0,
        }
    }

    pub fn create(&mut self, geometry: Geometry) -> EntityRef<'_> {
        let id = EntityId(self.next_key);
        self.next_key = self.next_key + 1;
        EntityRef(
            self.entity_data
                .entry(id)
                .or_insert(EntityData::new(geometry)),
            id,
        )
    }

    pub fn delete(&mut self, id: EntityId) {
        self.entity_data.remove(&id);
    }

    pub fn get(&mut self, id: EntityId) -> Option<EntityRef<'_>> {
        self.entity_data.get_mut(&id).map(|x| EntityRef(x, id))
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, EntityId, EntityData> {
        self.entity_data.iter()
    }
}

impl Scene {
    /// Returns an [`EntityRef`] with circle geometry attached. The default
    /// circle is at the world origin with a radius of one.
    pub fn circle(&mut self) -> EntityRef<'_> {
        self.store.create(Geometry::Primitive(Primitive::Circle))
    }

    // Removes an entity from the scene, or a no-op if the entity doesn't exist.
    pub fn delete(&mut self, entity: EntityId) {
        self.store.delete(entity);
    }

    /// Returns an Some with a handle to the entity if it exists, or none otherwise.
    pub fn get(&mut self, entity: EntityId) -> Option<EntityRef<'_>> {
        self.store.get(entity)
    }
}

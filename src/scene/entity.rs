//! An `Entity` is simple an id representing rendering data.

use crate::scene::{Fill, Registry, Transform, Transformable};
use crate::timeline::Timeline;
use glam::prelude::*;
use std::any::Any;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(usize);

impl EntityId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

/// A reference to an entities transform.
pub struct EntityRef<'a> {
    registry: &'a mut Registry,
    id: EntityId,
}

impl<'a> EntityRef<'a> {
    pub(crate) fn new(registry: &'a mut Registry, id: EntityId) -> Self {
        Self { registry, id }
    }

    pub(crate) fn attach<T: Any>(self, data: T) -> Self {
        self.registry.attach(self.id, data);
        self
    }

    // Sets the fill for this entity.
    pub fn fill(self, fill: impl Into<Timeline<Vec4>>) -> Self {
        self.attach(Fill(fill.into()))
    }

    /// Drops this reference and returns the id of this entity, converting
    /// this mutable reference `EntityRef` to an immutable one: `EntityId`.
    pub fn id(self) -> EntityId {
        self.id
    }
}

impl Transformable for EntityRef<'_> {
    fn transform(&mut self) -> &mut Transform {
        self.registry.get_or_default(self.id)
    }
}

//! An `Entity` is simple an id representing rendering data.

use crate::render::renderer::Geometry;
use crate::scene::{Transform, Transformable};
use crate::timeline::Timeline;
use glam::prelude::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct EntityId(usize);

impl EntityId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

/// This struct essentially is a timeline version of renderable. It
/// usually makes sense to keep this data packed because in rendering
/// we are accessing all fields.
pub(crate) struct EntityData {
    pub transforms: Vec<Transform>,
    pub geometry: Geometry,
    pub fill: Option<Timeline<Vec4>>,
}

impl EntityData {
    pub fn new(geometry: Geometry) -> Self {
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
    pub(crate) fn new(data: &'a mut EntityData, id: EntityId) -> Self {
        Self(data, id)
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

impl Transformable for EntityRef<'_> {
    fn transforms(&mut self) -> &'_ mut Vec<Transform> {
        &mut self.0.transforms
    }
}

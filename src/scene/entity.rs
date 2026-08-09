//! An `Entity` is simple an id representing rendering data.

use crate::scene::{Fill, Scene, Stroke, Transform, Transformable, Visibility};
use crate::timeline::{Timeline, TimelineAlong};
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
    pub(crate) scene: &'a mut Scene,
    pub(crate) id: EntityId,
}

impl<'a> EntityRef<'a> {
    pub(crate) fn new(scene: &'a mut Scene, id: EntityId) -> Self {
        Self { scene, id }
    }

    pub fn attach<T: Any>(self, data: T) -> Self {
        self.scene.registry.attach(self.id, data);
        self
    }

    pub fn has<T: Any>(&self) -> bool {
        self.scene.registry.has::<T>(self.id)
    }

    pub fn get<T: Any>(&mut self) -> Option<&mut T> {
        self.scene.registry.get_mut(self.id)
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.scene.registry.remove(self.id)
    }

    pub fn fill(self, color: impl Into<Timeline<Vec4>>) -> Self {
        let fill = self.scene.registry.get_or_default::<Fill>(self.id);
        fill.color = color.into();
        self
    }

    pub fn no_fill(self) -> Self {
        self.scene.registry.remove::<Fill>(self.id);
        self
    }

    pub fn stroke_weight(self, weight: impl Into<TimelineAlong<f32>>) -> Self {
        let stroke = self.scene.registry.get_or_default::<Stroke>(self.id);
        stroke.weight = weight.into().0;
        self
    }

    pub fn stroke(self, color: impl Into<TimelineAlong<Vec4>>) -> Self {
        let stroke = self.scene.registry.get_or_default::<Stroke>(self.id);
        stroke.color = color.into().0;
        self
    }

    pub fn no_stroke(self) -> Self {
        self.scene.registry.remove::<Stroke>(self.id);
        self
    }

    pub fn visibility(self, amount: impl Into<Timeline<f32>>) -> Self {
        let vis = self.scene.registry.get_or_default::<Visibility>(self.id);
        vis.amount = amount.into();
        self
    }

    pub fn z_index(self, z_index: impl Into<Timeline<f32>>) -> Self {
        let vis = self.scene.registry.get_or_default::<Visibility>(self.id);
        vis.z_index = z_index.into();
        self
    }

    /// Drops this reference and returns the id of this entity, converting
    /// this mutable reference `EntityRef` to an immutable one: `EntityId`.
    pub fn id(self) -> EntityId {
        self.id
    }
}

impl Transformable for EntityRef<'_> {
    fn transform(&mut self) -> &mut Transform {
        self.scene.registry.get_or_default(self.id)
    }
}

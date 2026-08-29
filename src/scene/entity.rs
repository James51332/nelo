//! An `Entity` is simple an id representing rendering data.

use crate::scene::{Color, Fill, Scene, Stroke, Transform, Transformable, Visibility};
use crate::timeline::{Timeline, TimelineAlong};
use std::any::Any;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(usize);

impl EntityId {
    pub(super) fn new(id: usize) -> Self {
        Self(id)
    }
}

/// A reference to an entity in the scene. The reference is guaranteed to be valid.
/// It holds the entity id and a mutable reference to the scene.
pub struct EntityRef<'a> {
    pub(super) scene: &'a mut Scene,
    pub(super) id: EntityId,
}

impl<'a> EntityRef<'a> {
    pub(super) fn new(scene: &'a mut Scene, id: EntityId) -> Self {
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

    /// Returns the attached data of type `T`, attaching `T::default()` first if
    /// this entity doesn't have one. Never removes or replaces existing data.
    pub fn get_or_default<T: Any + Default>(&mut self) -> &mut T {
        self.scene.registry.get_or_default(self.id)
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        self.scene.registry.remove(self.id)
    }

    pub fn fill(self, color: impl Into<Timeline<Color>>) -> Self {
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
        stroke.weight = weight.into().inner();
        self
    }

    pub fn stroke(self, color: impl Into<TimelineAlong<Color>>) -> Self {
        let stroke = self.scene.registry.get_or_default::<Stroke>(self.id);
        stroke.color = color.into().inner();
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

//! A Scene is an interface between data and rendering.

pub mod camera;
pub mod color;
pub mod component;
mod demo;
pub mod entity;
pub mod fonts;
pub mod group;
pub mod hierarchy;
pub mod playback;
mod registry;

pub use camera::Camera;
pub use color::Color;
pub use component::{
    Arrow, Circle, Fill, Glyph, Label, Spline, Step, Stroke, Transform, Transformable, Visibility,
};
pub use entity::{EntityId, EntityRef};
pub use fonts::Font;
pub use group::GroupRef;
pub use hierarchy::Hierarchy;
pub use playback::Playback;

use ab_glyph::FontArc;
use registry::Registry;
use std::collections::{BTreeSet, HashMap};

/// A Scene is the way that data is stored. All render data is attached to
/// entities, and renderers operate on entities which meet their criteria.
///
/// Usually, you don't have to think about this. `EntityRef` is a mutable
/// reference to an entity. It attaches components as needed. Since `EntityRef`
/// holds a mutable reference to the scene, you can use the `.id()` method to
/// get the `EntityId`, which is an immutable reference to the entity.
pub struct Scene {
    /// All entities which are active within the scene.
    active: BTreeSet<EntityId>,

    /// The entity id which is used next. Increment by one to given default rendering order.
    next_id: usize,

    /// Stores all components within the scene.
    registry: Registry,

    /// Stores the parent/child relationships within the scene.
    hierarchy: Hierarchy,

    camera: Camera,

    fonts: HashMap<Font, FontArc>,
}

impl Scene {
    /// Creates a scene with a default 16:9 aspect ratio.
    pub fn new() -> Self {
        Self::with_aspect(16.0 / 9.0)
    }

    /// Creates a scene with a customized aspect ratio. Defaults to 16:9
    /// if given aspect is negative or zero.
    pub fn with_aspect(aspect: f32) -> Self {
        let aspect = if aspect > 0.0 { aspect } else { 16.0 / 9.0 };
        Self {
            registry: Registry::default(),
            hierarchy: Hierarchy::default(),
            active: BTreeSet::default(),
            next_id: 0,
            camera: Camera::new(aspect),
            fonts: Font::map(),
        }
    }

    // ----- Entity Management -----

    /// Creates an empty entity with a `Transform` component.
    pub fn create(&mut self) -> EntityRef<'_> {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        self.active.insert(id);
        EntityRef::new(self, id).attach(Transform::default())
    }

    /// Returns an Some with a handle to the entity if it exists, or none otherwise.
    pub fn get(&mut self, entity: EntityId) -> Option<EntityRef<'_>> {
        if self.active.contains(&entity) {
            Some(EntityRef::new(self, entity))
        } else {
            None
        }
    }

    // Returns an iterator over all entities in this scene.
    pub fn entities(&self) -> Vec<EntityId> {
        self.active.iter().copied().collect()
    }

    /// Deletes an entity from the scene, or a no-op if the entity doesn't exist.
    pub fn delete(&mut self, entity: EntityId) {
        // Remove the entity from the active list.
        self.active.remove(&entity);

        // Remove the parents and children.
        self.hierarchy.remove(entity);

        // Remove all attached components.
        self.registry.delete(entity);
    }
}

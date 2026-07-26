//! A Scene is an interface between data and rendering.

pub mod camera;
pub mod component;
pub mod curve;
pub mod entity;
pub mod group;
pub mod path;
mod registry;
pub mod transform;

pub use camera::Camera;
pub use component::{Circle, Fill};
pub use curve::{Along, Curve, Spline, TimelineAlong, TimelineSpline};
pub use entity::{EntityId, EntityRef};
pub(crate) use registry::{Query, Registry};
pub use transform::{Transform, Transformable};

use crate::timeline::{Easing, Timeline};
use glam::prelude::*;
use std::any::Any;

/// A Scene is the way that data is stored. All render data is attached to
/// entities, and renderers operate on entities which meet their criteria.
///
/// Usually, you don't have to think about this. `EntityRef` is a mutable
/// reference to an entity. It attaches components as needed. Since `EntityRef`
/// holds a mutable reference to the scene, you can use the `.id()` method to
/// get the `EntityId`, which is an immutable reference to the entity.
pub struct Scene {
    registry: Registry,
    active: Vec<EntityId>,
    next_id: usize,
    camera: Camera,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            active: Vec::new(),
            next_id: 0,
            camera: Camera::new(),
        }
    }

    /// Creates an empty entity with a `Transform` component.
    pub fn create(&mut self) -> EntityRef<'_> {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        self.active.push(id);
        EntityRef::new(self, id).attach(Transform::default())
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Returns all attached data of a certain type sorted by EntityId.
    pub fn view<T: Any>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.registry.view()
    }

    /// Returns all entities an attached data for entities with components of type
    /// `A` and `B` attached.
    pub fn view_pair<A: Any, B: Any>(&self) -> Vec<(EntityId, &A, &B)> {
        self.registry.view_pair()
    }

    /// Returns all entities an attached data for entities with components of type
    /// `A`, `B`, and `C` attached.
    pub fn view_triple<A: Any, B: Any, C: Any>(&self) -> Vec<(EntityId, &A, &B, &C)> {
        self.registry.view_triple()
    }

    /// Returns a Vector of entities and their attached components which have
    /// up to five types specified by the generic tuple `T`.
    pub fn view_tuple<T: Query>(&self) -> T::Item<'_> {
        self.registry.view_tuple::<T>()
    }

    /// Returns an Some with a handle to the entity if it exists, or none otherwise.
    pub fn get(&mut self, entity: EntityId) -> Option<EntityRef<'_>> {
        match self.active.binary_search(&entity) {
            Ok(_) => Some(EntityRef::new(self, entity)),
            _ => None,
        }
    }

    /// Deletes an entity from the scene, or a no-op if the entity doesn't exist.
    pub fn delete(&mut self, entity: EntityId) {
        // Remove the entity from the active list.
        match self.active.binary_search(&entity) {
            Ok(i) => self.active.remove(i),
            _ => return,
        };

        // Remove all attached components.
        self.registry.delete(entity);
    }

    /// Returns a small demo scene.
    pub fn demo() -> Self {
        const PERIOD: f32 = 3.0;
        let mut scene = Self::new();

        // Set the background color.
        scene.camera_mut().background(Vec4::new(0.4, 0.3, 0.5, 1.0));

        // Central pulsing circle.
        scene.circle().scale(
            Timeline::triangle(2.0 * PERIOD)
                .then(Easing::SineInOut)
                .add(0.25),
        );

        // Some circles which go back and for from spirtal to a line.
        scene
            .group()
            .create(16, |_, s| s.circle().scale(0.1))
            .arrange(path::line(Vec2::X * 2.0, Vec2::X * 5.0))
            .for_each(|i, e| e.rotate(Timeline::triangle(6.0).ease().multiply(0.2 * i as f32)));

        // Wavy path.
        scene
            .curve(|t: f32, x: f32| Vec2::new(x, -4.0 - 0.6 * (x - 4.0 * t).sin()))
            .weight(0.05)
            .start_alpha(-10.0)
            .end_alpha(10.0);

        scene
    }
}

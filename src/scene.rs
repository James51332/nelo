//! A Scene is an interface between data and rendering.

pub mod camera;
pub mod component;
pub mod entity;
pub mod group;
pub mod path;
mod registry;
pub mod text;
pub mod transform;

pub use camera::Camera;
pub use component::{Circle, Fill, Spline, Stroke};
pub use entity::{EntityId, EntityRef};
pub use group::GroupRef;
pub(crate) use registry::{Query, Registry};
pub use text::Glyph;
pub use transform::{Transform, Transformable};

use crate::timeline::Timeline;
use ab_glyph::{FontArc, FontRef};
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
    font: FontArc,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            active: Vec::new(),
            next_id: 0,
            camera: Camera::new(),
            font: FontArc::new(
                FontRef::try_from_slice(include_bytes!("fonts/cmu.serif-roman.ttf")).unwrap(),
            ),
        }
    }

    /// Creates an empty entity with a `Transform` component.
    pub fn create(&mut self) -> EntityRef<'_> {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        self.active.push(id);
        EntityRef::new(self, id).attach(Transform::default())
    }

    pub fn sample_camera(&self, size: (u32, u32), t: f32) -> (Vec4, Affine2) {
        self.camera.sample(size, t)
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn font(&self) -> &FontArc {
        &self.font
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
    pub fn demo() -> Scene {
        let mut scene = Scene::new();

        // Set the background color.
        scene.camera().background(Vec4::new(0.4, 0.3, 0.5, 1.0));

        // Timeline to sample to repeat animations.
        let repeat = Timeline::triangle(6.0).ease();

        // Some circles which go back and forth from spiral to a line.
        let line = path::line(Vec2::X * 2.5, Vec2::X * 5.0);
        scene
            .group()
            .create(15, |_, s| s.circle().scale(0.1))
            .arrange(line)
            .for_each(|i, e| e.rotate(repeat.clone().add(0.2).multiply(i as f32)));

        // Let's create a shape using a spline.
        scene
            .text("Hello, world!")
            .scale(0.75)
            .rotate(repeat.clone().add(-0.5).multiply(1.5));

        // Wavy path.
        scene.spline_with_range(
            |t: f32, x: f32| Vec2::new(x, -4.0 - 0.6 * (x - 4.0 * t).sin()),
            -10.0,
            10.0,
        );

        scene
    }
}

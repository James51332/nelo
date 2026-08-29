//! A Scene is an interface between data and rendering.

pub mod camera;
pub mod color;
pub mod component;
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

use crate::timeline::{Path, Timeline};
use ab_glyph::FontArc;
use glam::prelude::*;
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

    // ----- Camera -----

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn sample_camera(&self, size: (u32, u32), t: f32) -> (Color, Affine2) {
        self.camera.sample(size, t)
    }

    pub fn sample_height(&self, time: f32) -> f32 {
        self.camera.height.sample(time)
    }

    pub fn aspect(&self) -> f32 {
        self.camera.aspect()
    }

    // ----- Fonts -----

    pub fn font(&self, font: Font) -> &FontArc {
        &self.fonts.get(&font).expect("Font not found in font map")
    }

    pub fn default_font(&self) -> &FontArc {
        self.font(Font::default())
    }

    // ----- Demo -----

    /// Returns a small demo scene.
    pub fn demo() -> Scene {
        let mut scene = Scene::new();

        // Set the background color.
        scene.camera().background(Color::srgb(0.4, 0.3, 0.5));

        // Timeline to sample to repeat animations.
        let repeat = Timeline::triangle(6.0).ease();

        // Some circles which go back and forth from spiral to a line.
        let line = Path::line(Vec2::X * 2.0, Vec2::X * 4.0);
        scene
            .group()
            .create(12, |_, s| s.dot().scale(0.08))
            .arrange(line)
            .for_each(|i, e| e.rotate(repeat.clone().add(0.2).multiply(i as f32)));

        // Create some elements in a group.
        scene
            .group()
            .create_once(|s| s.triangle())
            .create_once(|s| s.square())
            .create_once(|s| s.circle())
            .for_each(|_, e| e.scale(0.5))
            .row(1.25);

        // Render some text.
        scene.text("Hello, Nelo!").translate(Vec2::Y * 4.0);

        // Wavy path.
        scene.spline_with_range(
            |t: f32, x: f32| Vec2::new(x, -4.0 - 0.6 * (x - 4.0 * t).sin()),
            -10.0,
            10.0,
        );

        scene
    }
}

///! An `Entity` is simple an id.
use crate::render::renderer::{Geometry, Primitive};
use crate::scene::Scene;
use crate::timeline::Timeline;
use glam::prelude::*;

#[allow(unused)]
pub struct Entity(usize);

/// This struct essentially is a timeline version of renderable. It
/// usually makes sense to keep this data packed because in rendering
/// we are accessing all fields.
pub(crate) struct EntityData {
    // TODO More Robust transform approach
    pub transform: Timeline<Vec2>,
    pub geometry: Option<Geometry>,
    pub fill: Option<Timeline<Vec4>>,
}

// The tricky thing to decide for this API is what the best way to handle
// transformation is. We will build into an affine, but we want to support
// more convenient shorthands. The approach currently used is that we'll
// store subfields, then aggregate.
pub struct EntityBuilder<'a> {
    store: &'a mut Vec<EntityData>,
    transform: Timeline<Vec2>,
    geometry: Option<Geometry>,
    fill: Option<Timeline<Vec4>>,
}

impl<'a> EntityBuilder<'a> {
    pub(crate) fn new(store: &'a mut Vec<EntityData>, geometry: Option<Geometry>) -> Self {
        EntityBuilder {
            store,
            transform: Timeline::constant(Vec2::new(0.0, 0.0)),
            geometry,
            fill: None,
        }
    }

    pub fn translate(mut self, transform: impl Into<Timeline<Vec2>>) -> Self {
        self.transform = transform.into();
        self
    }

    pub fn fill(mut self, fill: impl Into<Timeline<Vec4>>) -> Self {
        self.fill = Some(fill.into());
        self
    }

    pub fn build(self) -> Entity {
        self.store.push(EntityData {
            transform: self.transform,
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

//! A [`Scene`] is an interface between data and rendering.
//!
//! The fundamental unit of data is the [`Entity`]. We use a standard
//! ECS model for nelo.

pub mod entity;
pub mod path;
pub mod store;
pub mod transform;

pub(crate) use entity::EntityData;
pub use entity::{EntityId, EntityRef};
pub use transform::{Transform, Transformable};

use crate::render::{Geometry, Primitive, Renderable};
use crate::timeline::{Easing, Timeline};
use glam::prelude::*;
use store::Store;

pub struct Scene {
    store: Store,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            store: Store::new(),
        }
    }

    /// Samples all of the scene into renderable data.
    pub fn sample(&self, t: f32) -> Vec<Renderable> {
        self.store
            .iter()
            .map(|(_id, c)| {
                // Combine all of the transforms on this entity. We'll have to adjust this logic
                // when we support groups and hierarchy, but this is sufficient for now.
                let mut combined = Affine2::default();
                for transform in c.transforms.iter() {
                    combined = transform.sample(t) * combined;
                }

                // Build a renderable.
                Renderable {
                    transform: combined,
                    geometry: c.geometry,
                    fill: match &c.fill {
                        Some(timeline) => timeline.sample(t),
                        None => Vec4::ONE,
                    },
                }
            })
            .collect()
    }

    /// Returns an `EntityRef` with circle geometry attached. The default
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

    /// A small animated scene with a ring of orbiting circles around a pulsing center.
    pub fn demo() -> Self {
        use std::f32::consts::TAU;

        const PERIOD: f32 = 3.0;
        let mut scene = Self::new();

        // Central pulsing circle.
        scene
            .circle()
            .scale(|t: f32| 1.25 + 0.75 * ((t - 2.0) * TAU / (PERIOD * 2.0)).sin())
            .fill(Vec4::new(0.9, 0.9, 1.0, 1.0));

        // Define the path over time. Change between square and circle on repeat.
        let path = Timeline::keyframes(path::square())
            .ease_at(PERIOD, path::circle(), Easing::CubicInOut)
            .ease_at(PERIOD * 2.0, path::square(), Easing::CubicInOut)
            .build()
            .compose(|t| t % (2.0 * PERIOD));

        // Orbiting square.
        const N: usize = 12;
        for i in 0..N {
            let phase = i as f32 / N as f32;
            let color = Vec4::new(0.5 + 0.5 * phase, 0.6, 1.0 - 0.5 * phase, 1.0);

            scene
                .circle()
                .scale(0.5)
                .translate(
                    path.clone()
                        .map(move |shape| {
                            Timeline::sawtooth(PERIOD)
                                .then(Easing::CubicInOut)
                                .add(phase + 0.125)
                                .then(shape)
                        })
                        .flatten()
                        .multiply(3.5),
                )
                .fill(color);
        }

        scene
    }
}

//! A [`Scene`] is an interface between data and rendering.
//!
//! The fundamental unit of data is the [`Entity`]. We use a standard
//! ECS model for nelo.

pub mod component;
pub mod entity;
pub mod path;
mod registry;
pub mod transform;

pub use component::{Circle, Fill};
pub use entity::{EntityId, EntityRef};
pub(crate) use registry::{Query, Registry};
pub use transform::{Transform, Transformable};

use crate::timeline::{Easing, Timeline};
use glam::prelude::*;
use std::any::Any;

pub struct Scene {
    registry: Registry,
    next_id: usize,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            next_id: 0,
        }
    }

    /// Creates an empty entity with a transform component
    pub fn create(&mut self) -> EntityRef<'_> {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        EntityRef::new(&mut self.registry, id).attach::<Vec<Transform>>(Vec::new())
    }

    /// Returns all attached data of a certain type sorted by EntityId.
    pub fn view<T: Any>(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.registry.view()
    }

    pub fn view_pair<A: Any, B: Any>(&self) -> Vec<(EntityId, &A, &B)> {
        self.registry.view_pair()
    }

    pub fn view_triple<A: Any, B: Any, C: Any>(&self) -> Vec<(EntityId, &A, &B, &C)> {
        self.registry.view_triple()
    }

    // Returns a Vector of entities and their attached components which have
    // up to five types specified by the generic tuple `T`.
    pub fn view_tuple<T: Query>(&self) -> T::Item<'_> {
        self.registry.view_tuple::<T>()
    }

    // Returns an `EntityRef` with circle geometry attached. The default
    // circle is at the world origin with a radius of one.
    pub fn circle(&mut self) -> EntityRef<'_> {
        self.create().attach(Circle).fill(Vec4::ONE)
    }

    // Removes an entity from the scene, or a no-op if the entity doesn't exist.
    // pub fn delete(&mut self, entity: EntityId) {
    //     self.store.delete(entity);
    // }

    // Returns an Some with a handle to the entity if it exists, or none otherwise.
    // pub fn get(&mut self, entity: EntityId) -> Option<EntityRef<'_>> {
    //     self.regi.get(entity)
    // }

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

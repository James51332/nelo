//! A [`Scene`] is an interface between data and rendering.
//!
//! The fundamental unit of data is the [`Entity`]. We use a standard
//! ECS model for nelo.

pub mod entity;
pub mod transform;

pub use transform::Transform;

use crate::render::Renderable;
use entity::EntityData;
use glam::prelude::*;

pub struct Scene {
    store: Vec<EntityData>,
}

impl Scene {
    pub fn new() -> Self {
        Self { store: Vec::new() }
    }

    /// Samples all of the scene into renderable data. Only returns
    /// data with geoemetry, since that is a requirement for [`Renderable`]
    pub fn sample(&self, t: f32) -> Vec<Renderable> {
        self.store
            .iter()
            .filter_map(|c| {
                if let Some(geometry) = c.geometry {
                    // Combine all of the transforms on this entity. We'll have to adjust this logic
                    // when we support groups and hierarchy, but this is sufficient for now.
                    let mut combined = Affine2::default();
                    for transform in c.transforms.iter() {
                        combined = transform.sample(t) * combined;
                    }

                    // Build a renderable.
                    Some(Renderable {
                        transform: combined,
                        geometry,
                        fill: match &c.fill {
                            Some(timeline) => timeline.sample(t),
                            None => Vec4::ONE,
                        },
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// A small animated scene: a ring of orbiting circles around a pulsing center.
    pub fn demo() -> Self {
        use std::f32::consts::TAU;

        let mut scene = Self::new();

        // Central pulsing circle.
        scene
            .circle()
            .scale(|t: f32| 1.25 + 0.5 * t.sin())
            .fill(Vec4::new(0.9, 0.9, 1.0, 1.0))
            .build();

        // Orbiting ring.
        const N: usize = 12;
        for i in 0..N {
            let hue = i as f32 / N as f32;
            let color = Vec4::new(0.5 + 0.5 * hue, 0.6, 1.0 - 0.5 * hue, 1.0);
            let phase = i as f32 / N as f32 * TAU;

            scene
                .circle()
                .scale(0.75)
                .translate(Vec2::new(3.5, 0.0))
                .rotate(phase)
                .rotate(move |t: f32| t * 0.6)
                .fill(color)
                .build();
        }

        scene
    }
}

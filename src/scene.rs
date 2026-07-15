//! A [`Scene`] is an interface between data and rendering.
//!
//! The fundamental unit of data is the [`Entity`]. We use a standard
//! ECS model for nelo.

pub mod entity;

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
                    Some(Renderable {
                        transform: Affine2::from_translation(c.transform.sample(t)),
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
        scene.circle().fill(Vec4::new(0.9, 0.9, 1.0, 1.0)).build();

        // Orbiting ring.
        const N: usize = 12;
        for i in 0..N {
            scene
                .circle()
                .translate(move |t: f32| {
                    let phase = i as f32 / N as f32 * TAU;
                    let angle = phase + t * 0.6;
                    let x = 3.5 * angle.cos();
                    let y = 3.5 * angle.sin();
                    Vec2::new(x as f32, y as f32)
                })
                .fill(move |_: f32| {
                    let hue = i as f32 / N as f32;
                    Vec4::new(0.5 + 0.5 * hue, 0.6, 1.0 - 0.5 * hue, 1.0)
                })
                .build();
        }

        scene
    }
}

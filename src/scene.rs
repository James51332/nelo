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
}

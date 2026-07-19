//! A collection of components which are used by the renderers.

use crate::scene::{EntityRef, Scene};
use crate::timeline::Timeline;
use glam::prelude::*;

/// A circle has no attached data. It has a radius of 1 but can
/// be scaled using a transform.
pub struct Circle;

/// A fill is a color over time.
pub struct Fill(pub Timeline<Vec4>);

impl Fill {
    pub fn sample(&self, t: f32) -> Vec4 {
        self.0.sample(t)
    }
}

impl Scene {
    /// Returns an `EntityRef` with circle geometry attached. The default
    /// circle is at the world origin with a radius of one and white fill.
    pub fn circle(&mut self) -> EntityRef<'_> {
        self.create().attach(Circle).fill(Vec4::ONE)
    }
}

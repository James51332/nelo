//! Each scene has exactly one camera. It defines a height and a transform.

use crate::scene::{Transform, Transformable};
use crate::timeline::Timeline;
use glam::prelude::Affine2;

pub struct Camera {
    height: Timeline<f32>,
    transform: Transform,
}

impl Camera {
    pub fn height(&mut self, height: impl Into<Timeline<f32>>) {
        self.height = height.into();
    }

    /// Creates a new camera with a default height of 10.0
    pub fn new() -> Self {
        Self {
            height: Timeline::constant(10.0),
            transform: Transform::default(),
        }
    }

    pub fn sample(&self, t: f32) -> (f32, Affine2) {
        (self.height.sample(t), self.transform.sample(t))
    }
}

impl Transformable for Camera {
    fn transform(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

//! Each scene has exactly one camera. It defines a height and a transform.

use crate::render::Color;
use crate::scene::{Transform, Transformable};
use crate::timeline::Timeline;
use glam::prelude::*;

pub struct Camera {
    pub height: Timeline<f32>,
    pub transform: Transform,
    pub background: Timeline<Color>,
}

impl Camera {
    /// Creates a new camera with a default height of 10.0 and the default
    /// background color.
    pub fn new() -> Self {
        Self {
            height: Timeline::constant(10.0),
            transform: Transform::default(),
            background: Color::srgb(0.02, 0.02, 0.04).into(),
        }
    }

    /// Returns the cameras background color, view projection at time `t`.
    pub fn sample(&self, size: (u32, u32), t: f32) -> (Color, Affine2) {
        let (width, height) = (size.0 as f32, size.1 as f32);
        let aspect = if width == 0.0 {
            1.0
        } else {
            width / if height == 0.0 { 1.0 } else { height }
        };
        let scene_height = self.height.sample(t);
        let scale = Vec2::new(2.0 / (scene_height * aspect), 2.0 / scene_height);
        let proj = Affine2::from_scale(scale);
        let view = self.transform.sample(t).inverse();

        (self.background.sample(t), proj * view)
    }

    pub fn height(&mut self, height: impl Into<Timeline<f32>>) -> &mut Self {
        self.height = height.into();
        self
    }

    pub fn background(&mut self, bg: impl Into<Timeline<Color>>) -> &mut Self {
        self.background = bg.into();
        self
    }
}

impl Transformable for &mut Camera {
    fn transform(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

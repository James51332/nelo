//! A playback is a wrapper around a scene which protects the scene from further modification.

use crate::scene::Scene;

pub struct Playback {
    scene: Scene,
    length: Option<f32>,
}

impl Playback {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene,
            length: None,
        }
    }

    pub fn length(&self) -> Option<f32> {
        self.length
    }

    pub fn with_length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }
}

impl From<Scene> for Playback {
    fn from(scene: Scene) -> Self {
        Self::new(scene)
    }
}

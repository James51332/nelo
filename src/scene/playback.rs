//! A trait which immutably wraps a scene with an optional name and length.

use crate::scene::Scene;

pub struct Playback {
    scene: Scene,
    length: Option<f32>,
    name: Option<String>,
}

impl Playback {
    pub fn new(scene: Scene, length: Option<f32>, name: Option<String>) -> Self {
        Self {
            scene,
            length,
            name,
        }
    }

    /// Returns the current scene for rendering.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// The length of play.
    pub fn length(&self) -> Option<f32> {
        self.length
    }

    /// Consumes and returns a new playback with length.
    pub fn with_length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    /// The name of this playback.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Consumes and returns a new playnack with given name.
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

impl From<Scene> for Playback {
    fn from(scene: Scene) -> Self {
        Self::new(scene, None, None)
    }
}

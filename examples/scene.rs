use glam::prelude::*;
use nelo::scene::Scene;
use nelo::timeline::Timeline;

fn main() {
    let mut scene = Scene::new();

    let circle = scene
        .circle()
        .transform(Timeline::dynamic(|t: f32| {
            Vec2::new((2.5 * t).cos(), (2.5 * t).sin())
        }))
        .build();
}

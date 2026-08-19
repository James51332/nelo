//! Renderers the demo scene at t=3.0 sec to a PNG file.

use nelo::prelude::*;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SCENE_TIME: f32 = 3.0;

fn main() -> Result<(), String> {
    let exporter = ImageExport {
        width: WIDTH,
        height: HEIGHT,
        time: SCENE_TIME,
        file_name: "nelo_scene",
        file_ext: "png",
        gpu: None,
    };

    exporter.export(Scene::demo())
}

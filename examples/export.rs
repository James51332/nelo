use nelo::prelude::*;

fn main() -> Result<(), String> {
    let scene = Scene::demo();
    let config = ExportConfig::default();
    export::video(scene, &config)
}

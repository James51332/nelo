use nelo::prelude::*;

fn main() -> Result<(), String> {
    let scene = Scene::demo();
    let config = ExportConfig::default();
    nelo::export(scene, &config)
}

use nelo::prelude::*;

fn main() -> Result<(), String> {
    VideoExport::default().export(Scene::demo())
}

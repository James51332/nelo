use nelo::prelude::*;

fn main() -> Result<(), String> {
    let playback: Playback = Story::demo().into();
    VideoExport {
        end_time: playback.length().unwrap_or(10.0),
        ..VideoExport::default()
    }
    .export(playback)
}

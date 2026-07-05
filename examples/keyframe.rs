use nelo::keyframe::Easing;
use nelo::timeline::Timeline;

fn main() {
    let keyframes = Timeline::keyframes(0.0)
        .at(1.0, 4.0)
        .ease_at(5.0, 0.0, Easing::CubicInOut)
        .build();

    for i in 0..6 {
        let t = i as f32;
        println!(
            "Keyframed timeline at t={t} has value {}",
            keyframes.sample(t)
        );
    }
}

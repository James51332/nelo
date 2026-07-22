# nelo

A stateless, timeline-driven animation engine for explorable visual animations.

## philosophy

> Moments in a scene are independent of one another. Everything is a function of time.*

At the heart of nelo is the `Timeline`, a generic type which can be queried at any given time.

Timelines can be built from a few sources:
* Constants
* Closures
* Keyframes
* Input (UI, Keyboard, Mouse)

Timelines can also be combined in a few ways:
* Addition - Layer one timeline on another (w/ optional offset)
* Multiplication - Apply first then second transform (e.g. matrix multiplication)
* Composition - Use one timelines output as the time input for another
* Sequencing - Join timelines together in series
* Sampling - Sample one timeline within a closure

On it's own, this API forces robust animations, but semantic sugar elevates the experience
to the level of any iterative simulation tool:
* Convenient Animations
* Grouping & Positioning
* Object Transformations

## example

Here's a small example scene:

```rust
let mut scene = Scene::new();

// Central pulsing circle.
scene
    .circle()
    .scale(|t: f32| 1.25 + 0.5 * t.sin())
    .fill(Vec4::new(0.9, 0.9, 1.0, 1.0));

// Orbiting square.
const N: usize = 12;
for i in 0..N {
    let phase = i as f32 / N as f32;
    let color = Vec4::new(0.5 + 0.5 * phase, 0.6, 1.0 - 0.5 * phase, 1.0);

    scene
        .circle()
        .scale(0.5)
        .translate(
            Timeline::rate(0.25)
                .then(|t| t % 1.0)
                .then(Easing::QuadInOut)
                .add(phase + 0.125)
                .then(path::square())
                .multiply(3.5),
        )
        .fill(color);
}
```

## dependencies

Exporting uses `ffmpeg`, so make sure it's installed and avaiable in your `$PATH`.


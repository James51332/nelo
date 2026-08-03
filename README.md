# nelo

[![Crates.io](https://img.shields.io/crates/v/nelo.svg)](https://crates.io/crates/nelo)
![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)

A stateless, timeline-driven animation engine for explorable visual animations.

## philosophy

> Moments in a scene are independent of one another. Everything is a function of time.

At the heart of nelo is the `Timeline`, a generic type which can be queried at any given time.

Timelines can be built from a few sources:
* Constants
* Closures
* Keyframes

They can also be combined to built much more complicated behavior using a handful of convenient
methods to add, multiply, compose, repeat, or 

Timelines take full advantage of Rust's powerful trait system to make the API a real joy.
You never have to pay for timelines where you don't want them, but you can drop them in 
to animate any parameter on any entity.

## example

Here's the code in 
[`src/scene.rs`](src/scene.rs)
to create the demo:

```rust
pub fn demo() -> Scene {
    let mut scene = Scene::new();

    // Set the background color.
    scene.camera().background(Vec4::new(0.4, 0.3, 0.5, 1.0));

    // Timeline to sample to repeat animations.
    let repeat = Timeline::triangle(6.0).ease();

    // Some circles which go back and forth from spiral to a line.
    let line = Path::line(Vec2::X * 2.0, Vec2::X * 4.0);
    scene
        .group()
        .create(12, |_, s| s.circle().scale(0.08))
        .arrange(line)
        .for_each(|i, e| e.rotate(repeat.clone().add(0.2).multiply(i as f32)));

    // Create some elements in a group.
    scene
        .group()
        .create_once(|s| s.triangle())
        .create_once(|s| s.square())
        .create_once(|s| s.circle())
        .for_each(|_, e| e.scale(0.5))
        .row(1.25);

    // Render some text.
    scene.text("Hello, Nelo!").translate(Vec2::new(0.0, 4.0));

    // Wavy path.
    scene.spline_with_range(
        |t: f32, x: f32| Vec2::new(x, -4.0 - 0.6 * (x - 4.0 * t).sin()),
        -10.0,
        10.0,
    );

    scene
}
```

And here's the result rendered at `t = 3.0`:

<p align="center">
  <img width="400" src="https://codeberg.org/dadabo/nelo/raw/branch/main/demo.png" alt="Demo scene"/>
</p>

## dependencies

Exporting uses `ffmpeg`, so make sure it's installed and avaiable in your `$PATH`.


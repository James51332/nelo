//! A path is a Along<Vec2> in the interval [0, 1].

use std::f32::consts::TAU;

use crate::timeline::{Along, Lerp, Timeline};
use glam::prelude::*;

// ----- Path -----

/// A path is a parameter which varies w/ respect to alpha. It can convert to a
/// `Timeline<Vec2>` using `Path::timeline()` or `Timeline::from()`. We also
/// add a few convenience methods so that we can interface with paths.
pub type Path = Along<Vec2>;

impl Timeline<Vec2> {
    /// Alias for .along() on Timeline<Vec2>
    pub fn path(self) -> Path {
        self.along()
    }
}

impl Path {
    /// Returns a path around the unit circle with length one second.
    /// Loops if sampled out of bounds.
    pub fn circle() -> Self {
        Timeline::dynamic(move |t: f32| {
            let theta = t * std::f32::consts::TAU;
            Vec2::new(theta.cos(), theta.sin())
        })
        .with_length(1.0)
        .path()
    }

    /// Traces a path CCW around a square with vertices at (+/- 1, +/- 1) in one second.
    /// Loops if sampled out of bounds.
    pub fn square() -> Self {
        Timeline::dynamic(move |t: f32| {
            let t = t.rem_euclid(1.0);

            if t < 0.125 {
                Vec2::interpolate(Vec2::X, Vec2::ONE, t * 8.0)
            } else if t <= 0.375 {
                Vec2::interpolate(Vec2::ONE, Vec2::new(-1.0, 1.0), (t - 0.125) * 4.0)
            } else if t <= 0.625 {
                Vec2::interpolate(Vec2::new(-1.0, 1.0), Vec2::NEG_ONE, (t - 0.375) * 4.0)
            } else if t <= 0.875 {
                Vec2::interpolate(Vec2::NEG_ONE, Vec2::new(1.0, -1.0), (t - 0.625) * 4.0)
            } else {
                Vec2::interpolate(Vec2::new(1.0, -1.0), Vec2::X, (t - 0.875) * 8.0)
            }
        })
        .with_length(1.0)
        .path()
    }

    /// Returns a triangle path.
    pub fn triangle() -> Self {
        // We spend equal time on each side even though we aren't equilateral.
        Timeline::keyframes(Vec2::new(0.5, 0.0))
            .at(1.0 / 6.0, Vec2::new(0.0, 1.0))
            .at(3.0 / 6.0, Vec2::new(-1.0, -1.0))
            .at(5.0 / 6.0, Vec2::new(1.0, -1.0))
            .at(6.0 / 6.0, Vec2::new(0.5, 0.0))
            .build()
            .repeat()
            .path()
    }

    /// Returns a star path with the specified number of `points` and optional
    /// `inner_radius`. Uses standard five-point star radius if `inner_radius`
    /// is `None`.
    pub fn star(points: u32, inner_radius: Option<f32>) -> Self {
        const STAR_INNER_RADIUS: f32 = 0.38196601125;
        let inner = inner_radius.unwrap_or(STAR_INNER_RADIUS);
        let vertices = (points as f32 * 2.0).max(1.0);

        let mut builder = Timeline::keyframes(Vec2::X);
        for i in 1..(vertices as u32) {
            let radius = if i % 2 == 1 { inner } else { 1.0 };
            let i = i as f32;
            let angle = i / vertices * TAU;
            builder = builder.at(i / vertices, radius * Vec2::from_angle(angle));
        }
        builder = builder.at(1.0, Vec2::X);

        builder.build().repeat().path()
    }

    /// Returns a path from a to b over the course of one second. Continues with constant
    /// velocity if sampled out of bounds. Call `repeat` to loop.
    pub fn line(a: Vec2, b: Vec2) -> Self {
        Timeline::dynamic(move |t| Vec2::interpolate(a, b, t))
            .with_length(1.0)
            .path()
    }

    pub fn rotate(self, angle: impl Into<Along<f32>>) -> Self {
        let angle = angle.into();
        Timeline::dynamic(move |t| Vec2::from_angle(angle.sample(t)).rotate(self.sample(t))).path()
    }

    pub fn bezier(start: Vec2) -> PathBuilder {
        PathBuilder::new(start)
    }
}

// ----- PathBuilder -----

#[derive(Debug, Clone, Copy)]
enum Bezier {
    LineTo(Vec2),
    QuadTo(Vec2, Vec2),
    CubicTo(Vec2, Vec2, Vec2),
}

impl Bezier {
    fn end(self) -> Vec2 {
        match self {
            Self::LineTo(end) => end,
            Self::QuadTo(_, end) => end,
            Self::CubicTo(_, _, end) => end,
        }
    }
}

/// Utility to generate bezier paths.
pub struct PathBuilder {
    start: Vec2,
    segments: Vec<Bezier>,
}

impl PathBuilder {
    fn new(start: Vec2) -> Self {
        Self {
            start,
            segments: Vec::new(),
        }
    }

    pub fn line_to(mut self, end: Vec2) -> Self {
        self.segments.push(Bezier::LineTo(end));
        self
    }

    pub fn quad_to(mut self, c0: Vec2, end: Vec2) -> Self {
        self.segments.push(Bezier::QuadTo(c0, end));
        self
    }

    pub fn cubic_to(mut self, c0: Vec2, c1: Vec2, end: Vec2) -> Self {
        self.segments.push(Bezier::CubicTo(c0, c1, end));
        self
    }

    pub fn build(self) -> Path {
        let start = self.start;
        let segments = self.segments;
        let n = segments.len();
        let step = 1.0 / n as f32;
        Timeline::dynamic(move |t: f32| {
            // Bounds check guarantees we intepolate valid segments.
            if t <= 0.0 {
                return start;
            } else if t >= 1.0 {
                return segments.last().map_or(start, |s| s.end());
            }

            // Determine which segment to use.
            let cur_step = t / step;
            let step_index = cur_step.floor();

            // Get the path to interpolate.
            let i = step_index as usize;
            let start = if i == 0 { start } else { segments[i - 1].end() };
            let remaining = segments[i];

            // Compute the point for given by changing t to local time.
            let t = cur_step - step_index;
            let tc = 1.0 - t;
            match remaining {
                Bezier::LineTo(end) => tc * start + t * end,
                Bezier::QuadTo(c0, end) => tc * tc * start + 2.0 * tc * t * c0 + t * t * end,
                Bezier::CubicTo(c0, c1, end) => {
                    tc * tc * tc * start
                        + 3.0 * tc * tc * t * c0
                        + 3.0 * tc * t * t * c1
                        + t * t * t * end
                }
            }
        })
        .with_length(1.0)
        .path()
    }
}

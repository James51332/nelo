//! A Path is a parametric curve for an alpha value in the interval [0, 1].

use crate::timeline::{Lerp, Timeline};
use glam::prelude::*;

/// Returns a path around the unit circle with length one second.
/// Loops if sampled out of bounds.
pub fn circle() -> Timeline<Vec2> {
    Timeline::dynamic(move |t: f32| {
        let theta = t * std::f32::consts::TAU;
        Vec2::new(theta.cos(), theta.sin())
    })
    .with_length(1.0)
}

/// Traces a path CCW around a square with vertices at (+/- 1, +/- 1) in one second.
/// Loops if sampled out of bounds.
pub fn square() -> Timeline<Vec2> {
    Timeline::dynamic(move |t: f32| {
        let t = t.rem_euclid(1.0);

        if t < 0.125 {
            Vec2::interpolate(&Vec2::X, &Vec2::ONE, t * 8.0)
        } else if t <= 0.375 {
            Vec2::interpolate(&Vec2::ONE, &Vec2::new(-1.0, 1.0), (t - 0.125) * 4.0)
        } else if t <= 0.625 {
            Vec2::interpolate(&Vec2::new(-1.0, 1.0), &Vec2::NEG_ONE, (t - 0.375) * 4.0)
        } else if t <= 0.875 {
            Vec2::interpolate(&Vec2::NEG_ONE, &Vec2::new(1.0, -1.0), (t - 0.625) * 4.0)
        } else {
            Vec2::interpolate(&Vec2::new(1.0, -1.0), &Vec2::X, (t - 0.875) * 8.0)
        }
    })
}

/// Returns a path from a to b over the course of one second. Continues with constant
/// velocity if sampled out of bounds.
pub fn line(a: Vec2, b: Vec2) -> Timeline<Vec2> {
    Timeline::dynamic(move |t| Vec2::interpolate(&a, &b, t)).with_length(1.0)
}

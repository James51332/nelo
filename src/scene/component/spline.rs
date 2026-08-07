//! Spline component is used for most rendering.

use crate::scene::{EntityRef, Scene, Stroke};
use crate::timeline::{Along, Timeline, TimelineSpline};
use glam::Vec2;

// ----- Spline -----

pub struct Spline {
    pub spline_path: Timeline<Along<Vec2>>,
    pub start_alpha: Timeline<f32>,
    pub end_alpha: Timeline<f32>,
}

impl Scene {
    pub fn spline<T>(&mut self, spline_path: T) -> EntityRef<'_>
    where
        T: Into<TimelineSpline>,
    {
        self.spline_with_range(spline_path, 0.0, 1.0)
    }

    pub fn spline_with_range<T, U, V>(&mut self, spline_path: T, start: U, end: V) -> EntityRef<'_>
    where
        T: Into<TimelineSpline>,
        U: Into<Timeline<f32>>,
        V: Into<Timeline<f32>>,
    {
        self.create()
            .attach(Spline {
                spline_path: spline_path.into().0.0,
                start_alpha: start.into(),
                end_alpha: end.into(),
            })
            .attach(Stroke::default())
    }
}

// ----- Arrow -----

/// Wraps a spline, but adds a triangle to the end.
pub struct Arrow {
    pub spline: Spline,
}

impl Scene {
    pub fn arrow<T>(&mut self, spline_path: T) -> EntityRef<'_>
    where
        T: Into<TimelineSpline>,
    {
        self.arrow_with_range(spline_path, 0.0, 1.0)
    }

    pub fn arrow_with_range<T, U, V>(&mut self, spline_path: T, start: U, end: V) -> EntityRef<'_>
    where
        T: Into<TimelineSpline>,
        U: Into<Timeline<f32>>,
        V: Into<Timeline<f32>>,
    {
        self.create()
            .attach(Arrow {
                spline: Spline {
                    spline_path: spline_path.into().0.0,
                    start_alpha: start.into(),
                    end_alpha: end.into(),
                },
            })
            .attach(Stroke::default())
    }
}

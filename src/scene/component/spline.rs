//! Spline component is used for most rendering.

use crate::scene::{EntityRef, Scene, Stroke};
use crate::timeline::{Along, Timeline, TimelineSpline};
use glam::Vec2;

// ----- Spline -----

pub struct Spline {
    pub spline_path: Timeline<Along<Vec2>>,
    pub start_alpha: Timeline<f32>,
    pub end_alpha: Timeline<f32>,
    pub close: Timeline<bool>,
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
                spline_path: spline_path.into().inner(),
                start_alpha: start.into(),
                end_alpha: end.into(),
                close: false.into(),
            })
            .attach(Stroke::default())
    }

    /// Creates a spline for a function of x.
    pub fn plot<T>(&mut self, function: T, x_min: f32, x_max: f32) -> EntityRef<'_>
    where
        T: Into<Along<f32>>,
    {
        let function = function.into();
        let spline = move |_t, x| Vec2::new(x, function.sample(x));
        self.spline_with_range(spline, x_min, x_max)
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
                    spline_path: spline_path.into().inner(),
                    start_alpha: start.into(),
                    end_alpha: end.into(),
                    close: false.into(),
                },
            })
            .attach(Stroke::default())
    }
}

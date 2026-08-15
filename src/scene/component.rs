//! A collection of components which are used by the renderers.

pub mod axes;
pub mod group;
pub mod latex;
pub mod spline;
pub mod text;
pub mod transform;

pub use group::GroupRef;
pub use spline::{Arrow, Spline};
pub use transform::{Transform, Transformable};

use crate::render::Color;
use crate::scene::{EntityRef, Scene};
use crate::timeline::{Along, Path, Timeline};
use std::f32::consts::PI;

// ----- Circle -----

/// A circle has no attached data. It has a radius of 1 but can
/// be scaled using a transform.
pub struct Circle;

impl Scene {
    /// Returns an `EntityRef` with a solid dot attached.
    pub fn dot(&mut self) -> EntityRef<'_> {
        self.create().attach(Circle).attach(Fill::solid())
    }

    /// Returns an `EntityRef` with circle geometry attached. The default
    /// circle is at the world origin with a radius of one and white fill.
    pub fn circle(&mut self) -> EntityRef<'_> {
        self.create()
            .attach(Circle)
            .attach(Fill::default())
            .attach(Stroke::default())
    }

    /// Returns an `EntityRef` with a square attached.
    pub fn square(&mut self) -> EntityRef<'_> {
        self.spline(Path::square()).attach(Fill::default())
    }

    /// Returns an `EntityRef` with a square attached.
    pub fn triangle(&mut self) -> EntityRef<'_> {
        self.spline(Path::triangle()).attach(Fill::default())
    }

    /// Returns an `EntityRef` with a star attached.
    pub fn star(&mut self) -> EntityRef<'_> {
        self.spline(Path::star(5, None).rotate(PI / 2.0))
            .attach(Fill::default())
    }
}

// ----- Fill -----

/// A fill is a color over time.
pub struct Fill {
    pub color: Timeline<Color>,
}

impl Fill {
    fn solid() -> Self {
        Self {
            color: Color::WHITE.into(),
        }
    }
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            color: Color::WHITE.with_alpha(0.5).into(),
        }
    }
}

// ----- Stroke -----

pub struct Stroke {
    pub color: Timeline<Along<Color>>,
    pub weight: Timeline<Along<f32>>,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            weight: Timeline::constant(0.025).along().into(),
            color: Timeline::constant(Color::WHITE).along().into(),
        }
    }
}

// ----- Visibility -----

/// Struct that renderers can opt into. Commonly animated by story
/// system to introduce an object.
pub struct Visibility {
    /// Ranges from zero to one based on how visible the object is. Often the
    /// alpha is proportional to the cube of this value, even though this time
    /// line is applied, so that fill comes after stroke.
    pub amount: Timeline<f32>,

    /// Z_index is the depth of an object. Higher z_index means higher view
    /// priority.
    pub z_index: Timeline<f32>,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            amount: 1.0.into(),
            z_index: 0.0.into(),
        }
    }
}

// ----- Glyph -----

/// A collection of contours
pub struct Glyph {
    pub contours: Vec<Spline>,
}

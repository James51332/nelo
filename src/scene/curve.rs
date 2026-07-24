//! Types and traits for defining values which vary both w.r.t time and curve position.

use crate::scene::{EntityId, EntityRef, Registry, Scene, Transform, Transformable};
use crate::timeline::{Lerp, Timeline};
use glam::prelude::*;

/// A curve is a spline and some associated data. We use `Along<T>` to define data that
/// is on the path. A spline is equivalent to `Along<Vec2>`.
pub struct Curve {
    pub spline: Timeline<Spline>,
    pub weight: Timeline<Along<f32>>,
    pub start_alpha: Timeline<f32>,
    pub end_alpha: Timeline<f32>,
}

// ----- Spline -----

pub type Spline = Along<Vec2>;

impl Timeline<Vec2> {
    /// Converts this timeline into a spline with time mapped to the spline parameter alpha.
    pub fn spline(self) -> Spline {
        self.along()
    }
}

// ----- Along<T> -----

/// Along defines a value "along" a parametric curve. It's sampled over alpha, which
/// usually varies from zero to one.
#[derive(Clone)]
pub struct Along<T: 'static>(Timeline<T>);

/// Alongs don't expose the same API at timelines, so we should build a timeline and
/// then call .along().
impl<T> Along<T> {
    /// Sample this instance with parameter value alpha.
    pub fn sample(&self, alpha: f32) -> T
    where
        T: Clone,
    {
        self.0.sample(alpha)
    }
}

impl<T> Timeline<T> {
    /// Converts this timeline into an `Along`. This maps t to alpha.
    pub fn along(self) -> Along<T> {
        Along(self)
    }
}

impl<T: Clone> From<Timeline<T>> for Along<T> {
    fn from(t: Timeline<T>) -> Self {
        t.along()
    }
}

impl<T: Clone> From<Along<T>> for Timeline<Along<T>> {
    fn from(t: Along<T>) -> Self {
        Timeline::constant(t)
    }
}

impl<T: Clone + Lerp> Lerp for Along<T> {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(Timeline::interpolate(&a.0, &b.0, t))
    }
}

impl<T: Clone, F: Fn(f32) -> T + Clone + 'static> From<F> for Along<T> {
    /// Alongs can also be generated from parameters. This isn't used by our public API since
    /// we always want to enable the value to change over time, so we use `TimelineAlong<T>`.
    /// This adds one more layer of indirection.
    fn from(f: F) -> Self {
        Self(Timeline::dynamic(f))
    }
}

// ----- TimelineSpline -----

/// Helper type used for building curves from parametrics.
#[derive(Clone)]
pub struct TimelineSpline(pub(crate) TimelineAlong<Vec2>);

impl From<Timeline<Vec2>> for TimelineSpline {
    /// Converts a `Timeline<Vec2>` to a constant spline.
    ///
    /// The big difference from TimelineAlong is that timeline along assumes
    /// that a timeline defines the time dependent value (e.g. color varying
    /// with time). TimelineSpline merits the one exception.
    fn from(s: Timeline<Vec2>) -> Self {
        Self(s.spline().into())
    }
}

impl From<Timeline<Spline>> for TimelineSpline {
    fn from(s: Timeline<Spline>) -> Self {
        Self(s.into())
    }
}

impl From<Timeline<Timeline<Vec2>>> for TimelineSpline {
    fn from(s: Timeline<Timeline<Vec2>>) -> Self {
        Self(s.map(|p| p.spline()).into())
    }
}

impl From<Spline> for TimelineSpline {
    fn from(s: Spline) -> Self {
        Self(s.into())
    }
}

impl Lerp for TimelineSpline {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(TimelineAlong::interpolate(&a.0, &b.0, t))
    }
}

impl<F: Fn(f32, f32) -> Vec2 + Clone + 'static> From<F> for TimelineSpline {
    fn from(f: F) -> Self {
        Self(f.into())
    }
}

// ----- TimelineAlong -----

#[derive(Clone)]
pub struct TimelineAlong<T: 'static>(pub(crate) Timeline<Along<T>>);

macro_rules! timeline_along_from {
    ($($t:ty),*) => {
        $(impl From<$t> for TimelineAlong<$t> {
            fn from(t: $t) -> Self {
                Self(Timeline::constant(t).along().into())
            }
        })*
    };
}

timeline_along_from!(f32, Vec2, Vec3, Vec4, Mat2, Affine2);

impl<T: Clone> From<Timeline<T>> for TimelineAlong<T> {
    /// Converts a `Timeline<T>` to a time-varying TimelineAlong whose value
    /// is uniform along the curve.
    fn from(t: Timeline<T>) -> Self {
        Self(t.along().into())
    }
}

impl<T: Clone> From<Timeline<Along<T>>> for TimelineAlong<T> {
    fn from(s: Timeline<Along<T>>) -> Self {
        Self(s)
    }
}

impl<T: Clone> From<Timeline<Timeline<T>>> for TimelineAlong<T> {
    fn from(s: Timeline<Timeline<T>>) -> Self {
        Self(s.map(|p| p.along()))
    }
}

impl<T: Clone> From<Along<T>> for TimelineAlong<T> {
    fn from(s: Along<T>) -> Self {
        Self(s.into())
    }
}

impl<T: Clone + Lerp> Lerp for TimelineAlong<T> {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Self(Timeline::interpolate(&a.0, &b.0, t))
    }
}

impl<T: Clone + 'static, F: Fn(f32, f32) -> T + Clone + 'static> From<F> for TimelineAlong<T> {
    /// Takes a closure over time and alpha and converts it to a timeline.
    fn from(f: F) -> Self {
        Self(Timeline::dynamic(move |t| {
            let inner = f.clone();
            Along(Timeline::dynamic(move |a| inner(t, a)))
        }))
    }
}

// ----- Curve -----

pub struct CurveRef<'a> {
    registry: &'a mut Registry,
    id: EntityId,
}

impl CurveRef<'_> {
    fn curve(&mut self) -> &mut Curve {
        self.registry.get_mut(self.id).unwrap()
    }

    pub fn weight(mut self, weight: impl Into<TimelineAlong<f32>>) -> Self {
        self.curve().weight = weight.into().0;
        self
    }

    pub fn start_alpha(mut self, alpha: impl Into<Timeline<f32>>) -> Self {
        self.curve().start_alpha = alpha.into();
        self
    }

    pub fn end_alpha(mut self, alpha: impl Into<Timeline<f32>>) -> Self {
        self.curve().end_alpha = alpha.into();
        self
    }

    pub fn id(self) -> EntityId {
        self.id
    }
}

impl Transformable for CurveRef<'_> {
    fn transform(&mut self) -> &mut Transform {
        self.registry.get_or_default(self.id)
    }
}

impl<'a> EntityRef<'a> {
    /// Returns a curve reference for this entity.
    pub fn as_curve(self) -> Option<CurveRef<'a>> {
        if self.has::<Curve>() {
            Some(CurveRef {
                registry: &mut self.scene.registry,
                id: self.id,
            })
        } else {
            None
        }
    }
}

impl Scene {
    pub fn curve(&mut self, spline: impl Into<TimelineSpline>) -> CurveRef<'_> {
        self.create()
            .attach(Curve {
                spline: spline.into().0.0,
                weight: Timeline::constant(0.2).along().into(),
                start_alpha: Timeline::constant(0.0),
                end_alpha: Timeline::constant(1.0),
            })
            .fill(Vec4::ONE)
            .as_curve()
            .unwrap()
    }
}

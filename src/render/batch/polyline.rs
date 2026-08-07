//! A sequence of lines. Allows user to specify the vertex type.

use crate::render::batch::StrokePoint;
use crate::render::{Batch, MeshVertex};
use crate::timeline::Along;
use glam::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;

// ----- Subdivide -----

const MIN_SEGMENTS: u32 = 50;
const MAX_SUBDIVISIONS: u32 = 10;

/// Use alpha as key for polyline gen. Keys need Ord, so we wrap f32 and add it.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Key(f32);
impl Eq for Key {}
impl Ord for Key {
    fn cmp(&self, other: &Key) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

type Map = BTreeMap<Key, Vec2>;

// ----- Polyline -----

/// A series of points and alpha values. Used to convert
#[derive(Debug, Clone)]
pub struct Polyline {
    points: Vec<(f32, Vec2)>,
}

impl Polyline {
    pub fn points(&self) -> &Vec<(f32, Vec2)> {
        &self.points
    }

    /// Takes a spline and the range of alpha values for which it is rendered and
    /// returns a polyline of (alpha, point) pairs.
    pub fn flatten(spline: &Along<Vec2>, start: f32, end: f32, tolerance: f32) -> Self {
        // Start by inserting the minimum number of segments.
        let mut map = Map::new();
        let step = (end - start) / MIN_SEGMENTS as f32;
        for i in 0..=MIN_SEGMENTS {
            let alpha = start + (i as f32) * step;
            map.insert(Key(alpha), spline.sample(alpha));
        }

        // Then apply our subdivions to each segment.
        let tolerance_squared = tolerance * tolerance;
        for i in 0..MIN_SEGMENTS {
            let min = start + i as f32 * step;
            let max = start + (i + 1) as f32 * step;
            Self::subdivide_segment(
                spline,
                &mut map,
                min,
                max,
                MAX_SUBDIVISIONS,
                tolerance_squared,
            );
        }

        // Collect our values back into polyline.
        Self {
            points: map.into_iter().map(|(k, v)| (k.0, v)).collect(),
        }
    }

    fn subdivide_segment(
        spline: &Along<Vec2>,
        map: &mut Map,
        start: f32,
        end: f32,
        max_subdivisions: u32,
        tolerance_squared: f32,
    ) {
        if max_subdivisions == 0 {
            return;
        }

        let start_point = map.get(&Key(start));
        let end_point = map.get(&Key(end));
        if let (Some(start_point), Some(end_point)) = (start_point, end_point) {
            // Sample the distance and find the distance from the chord.
            let mid_alpha = (start + end) * 0.5;
            let sampled = spline.sample(mid_alpha);

            // Compute the distance from the chord.
            let delta = sampled - start_point;
            let dir = (end_point - start_point).normalize_or_zero();
            let divergence = (delta - dir * dir.dot(delta)).length_squared();

            // If we are too far, repeat with less depth.
            if divergence >= tolerance_squared {
                map.insert(Key(mid_alpha), sampled);

                Self::subdivide_segment(
                    spline,
                    map,
                    start,
                    mid_alpha,
                    max_subdivisions - 1,
                    tolerance_squared,
                );
                Self::subdivide_segment(
                    spline,
                    map,
                    mid_alpha,
                    end,
                    max_subdivisions - 1,
                    tolerance_squared,
                );
            }
        }
    }
}

// ----- Batch -----

impl Batch {
    /// Method to subdivide and render filled spline.
    pub fn fill<T>(&mut self, polyline: Polyline, color: T)
    where
        T: Fn(f32) -> Vec4,
    {
        // Helper to convert polyline points into MeshVertex.
        let convert = |(a, pos)| MeshVertex::new(pos, Vec2::ZERO, color(a));

        // Create a builder from the first point.
        let mut iter = polyline.points.into_iter();
        let Some(start) = iter.next() else {
            log::info!("Skipping empty polyline!");
            return;
        };

        let mut builder = self.fill_builder(1.0);
        builder.begin_subpath(convert(start));

        // Iterate over the rest to build the spline.
        iter.for_each(|x| builder.line_to(convert(x)));
        builder.end_subpath(true);
        let result = builder.finish();
        if let Err(e) = result {
            log::info!("Failed to fill polyline: {e}");
        };
    }

    /// Method to subdivide and render stroked spline.
    pub fn stroke<T>(&mut self, polyline: Polyline, color_weight: T)
    where
        T: Fn(f32) -> (Vec4, f32),
    {
        // Helper to convert polyline points into MeshVertex.
        let convert = |(a, pos)| {
            let (color, weight) = color_weight(a);
            StrokePoint::new(pos, color, weight)
        };

        // Create a builder from the first point.
        let mut iter = polyline.points.into_iter();
        let Some(start) = iter.next() else {
            log::info!("Skipping empty polyline!");
            return;
        };
        let mut builder = self.stroke_builder(convert(start), 1.0);

        // Iterate over the rest to build the spline.
        iter.for_each(|x| builder.line_to(convert(x)));
        let result = builder.finish(false);
        if let Err(e) = result {
            log::info!("Failed to stroke polyline: {e}");
        };
    }
}

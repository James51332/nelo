//! Utility for generating geometry.

use crate::scene::Spline;
use glam::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;

const MIN_SEGMENTS: u32 = 50;
const MAX_SUBDIVISIONS: u32 = 10;
const DIVERGE_TOLERANCE: f32 = 0.001;
const DIVERGE_TOLERANCE_SQUARED: f32 = DIVERGE_TOLERANCE * DIVERGE_TOLERANCE;

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

pub fn generate_polyline(spline: &Spline, start: f32, end: f32) -> Vec<(f32, Vec2)> {
    // Start by inserting the minimum number of segments.
    let mut map = Map::new();
    let step = (end - start) / MIN_SEGMENTS as f32;
    for i in 0..=MIN_SEGMENTS {
        let alpha = start + (i as f32) * step;
        map.insert(Key(alpha), spline.sample(alpha));
    }

    // Then apply our subdivions to each segment.
    for i in 0..MIN_SEGMENTS {
        let min = start + i as f32 * step;
        let max = start + (i + 1) as f32 * step;
        subdivide_segment(spline, &mut map, min, max, MAX_SUBDIVISIONS);
    }

    // Collect our values back into polyline.
    map.into_iter().map(|(k, v)| (k.0, v)).collect()
}

fn subdivide_segment(spline: &Spline, map: &mut Map, start: f32, end: f32, max_subdivisions: u32) {
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
        if divergence >= DIVERGE_TOLERANCE_SQUARED {
            map.insert(Key(mid_alpha), sampled);
            subdivide_segment(spline, map, start, mid_alpha, max_subdivisions - 1);
            subdivide_segment(spline, map, mid_alpha, end, max_subdivisions - 1);
        }
    }
}

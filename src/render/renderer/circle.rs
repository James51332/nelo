//! Helper methods to render circles

use crate::render::{Batch, StrokePoint};
use crate::scene::{Circle, Fill, Scene, Stroke, Transform, Visibility};
use glam::prelude::*;
use std::f32::consts::TAU;

/// Renders all circles from the scene into the batch.
pub fn circles(batch: &mut Batch, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all elements with the required components.
    let items = scene.view_pair::<Transform, Circle>();
    items.into_iter().for_each(|(id, transform, _)| {
        let affine = transform.sample(t);

        // Visibility changes opacity for background and stroke length.
        let visibility = scene
            .component::<Visibility>(id)
            .map_or(1.0, |v| v.amount.sample(t).clamp(0.0, 1.0));

        if let Some(fill) = scene.component::<Fill>(id) {
            let mut color = fill.color.sample(t);
            // Apply a quad easing. This just looks better.
            color.w *= visibility * visibility * visibility;
            batch.add_circle(affine, color);
        }

        if let Some(stroke) = scene.component::<Stroke>(id) {
            // Utility to convert alpha to a StrokePoint.
            let color = stroke.color.sample(t);
            let weight = stroke.weight.sample(t);
            let convert = move |a: f32| {
                let theta = TAU * a;
                let pos = affine.matrix2 * Vec2::new(theta.cos(), theta.sin()) + affine.translation;
                StrokePoint::new(pos, color.sample(a), weight.sample(a))
            };

            // Compute how many points we have.
            const POINTS_PER_UNIT: f32 = 75.0;
            let scale = affine.matrix2.determinant().abs().sqrt();
            let points = POINTS_PER_UNIT * scale;
            let step = 1.0 / points;

            // The scale of stroke builder is world space, which is normal here.
            let mut builder = batch.stroke_builder(convert(0.0), 1.0);
            for i in 1..((points * visibility) as usize) {
                let a = step * i as f32;
                builder.line_to(convert(a));
            }

            // Only close the curve if visibility is high.
            let res = builder.finish(visibility >= 0.99);
            if let Err(e) = res {
                log::info!("Error adding stroke to circle: {e}");
            }
        }
    });
}

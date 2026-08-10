//! Helper methods to render circles

use crate::render::batch::RenderCommand;
use crate::render::{Batch, Polyline};
use crate::scene::{Circle, Fill, Scene, Stroke, Transform, Visibility};
use crate::timeline::Path;

/// Renders all circles from the scene into the batch.
pub fn circles(batch: &mut Batch, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all elements with the required components.
    let items = scene.view_pair::<Transform, Circle>();
    items.into_iter().for_each(|(id, transform, _)| {
        let affine = transform.sample(t);

        // Visibility changes opacity for background and stroke length.
        let visibility = scene.component::<Visibility>(id);
        let vis_amount = visibility.map_or(1.0, |v| v.amount.sample(t).clamp(0.0, 1.0));
        let z_index = visibility.map_or(0.0, |v| v.z_index.sample(t));
        if vis_amount < 0.005 {
            return;
        }

        if let Some(fill) = scene.component::<Fill>(id) {
            let mut color = fill.color.sample(t);
            // Apply a quad easing. This just looks better.
            color.alpha *= vis_amount * vis_amount * vis_amount;
            let command = RenderCommand::Circle {
                transform: affine.clone(),
                color,
            };

            batch.add_command(command, id, z_index);
        }

        if let Some(stroke) = scene.component::<Stroke>(id) {
            // Generate the path and flatten it.
            let path = Path::circle().map(move |v| affine.matrix2 * v + affine.translation);
            let polyline = Polyline::flatten(&path, 0.0, vis_amount, batch.tolerance());

            // Generate the render command and submit.
            let color = stroke.color.sample(t);
            let weight = stroke.weight.sample(t);
            let map = |a| (color.sample(a), weight.sample(a));
            let command = polyline.to_stroke(map, vis_amount >= 0.995);
            batch.add_command(command, id, z_index);
        }
    });
}

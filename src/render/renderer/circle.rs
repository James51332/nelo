//! Helper methods to render circles

use crate::render::batch::RenderCommand;
use crate::render::{Encoder, Polyline};
use crate::scene::{Circle, Fill, Scene, Stroke, Transform, Visibility};
use crate::timeline::Path;

/// Renders all circles from the scene into the batch.
pub(crate) fn circles(encoder: &mut Encoder, scene: &Scene, t: f32, _size: (u32, u32)) {
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

            encoder.add_command(id, command, z_index);
        }

        if let Some(stroke) = scene.component::<Stroke>(id) {
            // Generate the path and flatten it.
            let path = Path::circle().map(move |v| affine.matrix2 * v + affine.translation);
            let polyline = Polyline::flatten(&path, 0.0, vis_amount, scene.sample_height(t));

            // Generate the render command and submit.
            let close = vis_amount >= 0.995;
            let commands = RenderCommand::polyline(polyline, close, t, None, Some(stroke));
            for command in commands.into_iter() {
                encoder.add_command(id, command, z_index);
            }
        }
    });
}

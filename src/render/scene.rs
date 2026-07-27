//! Renders a scene at a given time. It must own its scene.

use crate::render::{Batch, BatchSet, CameraBuffer, Gpu, SplinePoint, tesselate};
use crate::scene::{Circle, Fill, Scene, Spline, Stroke, Transform, path};
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

type Renderers = Vec<Box<dyn Fn(&mut BatchSet, &Scene, f32, (u32, u32))>>;

pub struct SceneRenderer {
    scene: Scene,
    camera_buffer: CameraBuffer,

    // Batches are reusable geometry pipelines.
    batches: BatchSet,

    // Renderers forward the scene into batches.
    renderers: Renderers,
}

impl SceneRenderer {
    pub fn new(gpu: &Gpu, scene: Scene) -> Self {
        let camera_buffer = CameraBuffer::new(&gpu);
        let batches = BatchSet::new(&gpu, camera_buffer.layout());
        let renderers: Renderers = vec![
            Box::new(filled_circles),
            Box::new(stroked_circles),
            Box::new(stroked_curves),
        ];

        Self {
            scene,
            camera_buffer,
            batches,
            renderers,
        }
    }

    // Renders the scene to the assigned frame and presents the frame if possible.
    // Uses all renderers and supplies them the data according to their geometry
    // filter.
    pub fn render(&mut self, gpu: &Gpu, view: &TextureView, t: f32) {
        let size = (view.texture().width(), view.texture().height());

        // Populate out batches.
        for renderer in self.renderers.iter() {
            renderer(&mut self.batches, &self.scene, t, size);
        }

        // Copy the data to the gpu.
        self.batches.circles.prepare(&gpu);
        self.batches.splines.prepare(&gpu);

        // Upload the camera data into the buffer.
        let (background, view_proj) = self.scene.sample_camera(size, t);
        self.camera_buffer.upload(gpu, &view_proj, t);

        // Get our command encoder and build our render pass.
        let encoder_desc = CommandEncoderDescriptor {
            label: Some("nelo scene renderer encoder"),
        };
        let mut encoder = gpu.device().create_command_encoder(&encoder_desc);

        // Build our render pass from each of the renderers.
        {
            let render_pass_desc = RenderPassDescriptor {
                label: Some("nelo pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: background.x as f64,
                            g: background.y as f64,
                            b: background.z as f64,
                            a: background.w as f64,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            };
            let mut pass = encoder.begin_render_pass(&render_pass_desc);

            // Set the camera to bind_group zero.
            pass.set_bind_group(0, self.camera_buffer.bind_group(), &[]);

            // Submit the batches.
            self.batches.circles.submit(gpu, &mut pass);
            self.batches.splines.submit(gpu, &mut pass);
        }

        // Submit the draw commands to the GPU.
        gpu.queue().submit(std::iter::once(encoder.finish()));
    }
}

// Simple closures extract data to render from the scene.
fn filled_circles(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all elements with the required components.
    let items = scene.view_triple::<Transform, Circle, Fill>();

    // Submit a circle for each one.
    items.iter().for_each(|(_, transform, _, fill)| {
        batches
            .circles
            .push(transform.sample(t), fill.color.sample(t));
    });
}

fn stroked_circles(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Find the circles with transform and stroke.
    let items = scene.view_triple::<Transform, Circle, Stroke>();

    items.iter().for_each(|(_, transform, _, stroke)| {
        let affine = transform.sample(t);
        let spline = path::circle().map(move |x| affine.matrix2 * x + affine.translation);
        let polyline = tesselate::generate_polyline(&spline.along(), 0.0, 1.0);
        let points: Vec<SplinePoint> = polyline
            .into_iter()
            .map(|(a, pos)| {
                SplinePoint::new(
                    pos,
                    stroke.color.sample(t).sample(a),
                    stroke.weight.sample(t).sample(a),
                )
            })
            .collect();

        batches.splines.push(&points);
    });
}

fn stroked_curves(batches: &mut BatchSet, scene: &Scene, t: f32, _size: (u32, u32)) {
    // Get a view of all curves with a stroke.
    let items = scene.view_triple::<Transform, Spline, Stroke>();

    items.iter().for_each(|(_, transform, spline, stroke)| {
        // Subdivide the curve into a polyline.
        let affine = transform.sample(t);

        // Apply the transformation.
        let spline_path = spline
            .spline_path
            .sample(t)
            .timeline()
            .map(move |x| affine.matrix2 * x + affine.translation)
            .along();

        let polyline = tesselate::generate_polyline(
            &spline_path,
            spline.start_alpha.sample(t),
            spline.end_alpha.sample(t),
        );

        // Convert the (alpha, pos) values into renderable points.
        let spline_points = polyline
            .into_iter()
            .map(|(a, pos)| {
                SplinePoint::new(
                    pos,
                    stroke.color.sample(t).sample(a),
                    stroke.weight.sample(t).sample(a),
                )
            })
            .collect();

        // Submit the curve to the batch.
        batches.splines.push(&spline_points);
    });
}

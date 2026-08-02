//! Renders a scene at a given time.

pub mod circle;
pub mod glyph;
pub mod spline;

use crate::render::{Batch, BatchSet, CameraBuffer, Gpu};
use crate::scene::Scene;
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

pub type ComponentRenderer = Box<dyn Fn(&mut BatchSet, &Scene, f32, (u32, u32))>;

pub struct Renderer {
    scene: Scene,
    camera_buffer: CameraBuffer,

    // Batches are reusable geometry pipelines.
    batches: BatchSet,

    // Component renderers forward the scene into batches.
    renderers: Vec<ComponentRenderer>,
}

impl Renderer {
    pub fn new(gpu: &Gpu, scene: Scene) -> Self {
        let camera_buffer = CameraBuffer::new(&gpu);
        let batches = BatchSet::new(&gpu, camera_buffer.layout());
        let renderers: Vec<ComponentRenderer> = vec![
            Box::new(circle::filled_circles),
            Box::new(circle::stroked_circles),
            Box::new(spline::filled_splines),
            Box::new(spline::stroked_splines),
            glyph::get_filled_renderer(scene.font().clone()),
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
        self.batches.models.prepare(&gpu);
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
            self.batches.models.submit(gpu, &mut pass);
            self.batches.splines.submit(gpu, &mut pass);
        }

        // Submit the draw commands to the GPU.
        gpu.queue().submit(std::iter::once(encoder.finish()));
    }
}

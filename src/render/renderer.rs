//! Renders a scene at a given time.

pub mod circle;
pub mod glyph;
pub mod spline;

use crate::render::{Batch, CameraBuffer, Gpu};
use crate::scene::Scene;
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

pub type ComponentRenderer = Box<dyn Fn(&mut Batch, &Scene, f32, (u32, u32))>;

pub struct Renderer {
    scene: Scene,
    camera_buffer: CameraBuffer,

    // Geometry primitives
    batch: Batch,

    // Component renderers forward the scene into the batch.
    renderers: Vec<ComponentRenderer>,
}

impl Renderer {
    pub fn new(gpu: &Gpu, scene: Scene) -> Self {
        let camera_buffer = CameraBuffer::new(&gpu);
        let batch = Batch::new(&gpu, camera_buffer.layout());
        let renderers: Vec<ComponentRenderer> = vec![
            Box::new(circle::circles),
            Box::new(spline::splines),
            Box::new(spline::arrows),
            Box::new(glyph::filled_glyphs),
        ];

        Self {
            scene,
            camera_buffer,
            batch,
            renderers,
        }
    }

    // Renders the scene to the assigned frame and presents the frame if possible.
    // Uses all renderers and supplies them the data according to their geometry
    // filter.
    pub fn render(&mut self, gpu: &Gpu, view: &TextureView, t: f32) {
        let size = (view.texture().width(), view.texture().height());

        // Populate out batch.
        for renderer in self.renderers.iter() {
            renderer(&mut self.batch, &self.scene, t, size);
        }

        // Copy the data to the gpu.
        self.batch.prepare(&gpu);

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

            // Submit the batch.
            self.batch.submit(gpu, &mut pass);
        }

        // Submit the draw commands to the GPU.
        gpu.queue().submit(std::iter::once(encoder.finish()));
    }
}

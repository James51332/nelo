//! Scene renderer renders a scene at a given time. It must own its scene.

use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

use crate::render::{CameraBuffer, CircleRenderer, CurveRenderer, Gpu, Renderer};
use crate::scene::Scene;

pub struct SceneRenderer {
    scene: Scene,
    camera_buffer: CameraBuffer,
    renderers: Vec<Box<dyn Renderer>>,
}

impl SceneRenderer {
    pub fn new(gpu: &Gpu, scene: Scene) -> Self {
        let camera_buffer = CameraBuffer::new(&gpu);
        let renderers: Vec<Box<dyn Renderer>> = vec![
            Box::new(CircleRenderer::new(gpu, &camera_buffer.layout())),
            Box::new(CurveRenderer::new(gpu, &camera_buffer.layout())),
        ];

        Self {
            scene,
            camera_buffer,
            renderers,
        }
    }

    pub fn camera_buffer(&self) -> &CameraBuffer {
        &self.camera_buffer
    }

    pub fn add(&mut self, renderer: impl Renderer + 'static) {
        self.renderers.push(Box::new(renderer));
    }

    // Renders the scene to the assigned frame and presents the frame if possible.
    // Uses all renderers and supplies them the data according to their geometry
    // filter.
    pub fn render(&mut self, gpu: &Gpu, view: &TextureView, t: f32) {
        let size = (view.texture().width(), view.texture().height());
        for renderer in self.renderers.iter_mut() {
            renderer.prepare(&gpu, size, &self.scene, t);
        }

        // Upload the camera data into the buffer.
        let (background, view_proj) = self.scene.camera().sample(size, t);
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

            for renderer in self.renderers.iter() {
                pass.set_bind_group(0, self.camera_buffer.bind_group(), &[]);
                renderer.submit(&mut pass);
            }
        }

        // Submit the draw commands to the GPU.
        gpu.queue().submit(std::iter::once(encoder.finish()));
    }
}

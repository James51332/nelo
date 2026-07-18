//! Scene renderer renders a scene at a given time. It must own its scene.

use crate::render::{Camera, Gpu, Renderer};
use crate::scene::Scene;

pub struct SceneRenderer {
    camera: Camera,
    scene: Scene,
    renderers: Vec<Box<dyn Renderer>>,
}

impl SceneRenderer {
    pub fn new(camera: Camera, scene: Scene) -> Self {
        Self {
            scene,
            camera,
            renderers: Vec::new(),
        }
    }

    pub fn add(&mut self, renderer: impl Renderer + 'static) {
        self.renderers.push(Box::new(renderer));
    }

    // Renders the scene to the assigned frame and presents the frame if possible.
    // Uses all renderers and supplies them the data according to their geometry
    // filter.
    pub fn render(&mut self, gpu: &Gpu, view: &wgpu::TextureView, t: f32) {
        for renderer in self.renderers.iter_mut() {
            renderer.prepare(&gpu, &self.scene, t);
        }

        // Update the camera data.
        let size = (view.texture().width(), view.texture().height());
        self.camera.upload(gpu, t, size);

        // Get our command encoder and build our render pass.
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nelo scene renderer encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nelo pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.04,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // Camera is always at binding 0.
            pass.set_bind_group(0, self.camera.bind_group(), &[]);

            for renderer in self.renderers.iter() {
                renderer.submit(&mut pass);
            }
        }

        // Submit the draw commands to the GPU.
        gpu.queue().submit(std::iter::once(encoder.finish()));
    }
}

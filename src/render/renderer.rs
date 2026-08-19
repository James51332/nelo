//! Renders a scene at a given time.

pub mod circle;
pub mod glyph;
pub mod spline;

use crate::render::{Batch, CameraBuffer, Encoder};
use crate::scene::Scene;
use wgpu::{
    Color, CommandEncoderDescriptor, Device, Extent3d, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

pub type ComponentRenderer = Box<dyn Fn(&mut Encoder, &Scene, f32, (u32, u32))>;

// ----- Renderer -----

pub struct Renderer {
    device: Device,
    queue: Queue,
    format: TextureFormat,

    scene: Scene,
    camera_buffer: CameraBuffer,
    msaa_texture: Texture,

    // Primitive render and methods that load scene into it.
    batch: Batch,
    renderers: Vec<ComponentRenderer>,
}

impl Renderer {
    pub fn new(
        device: Device,
        queue: Queue,
        format: TextureFormat,
        playback: impl Into<Playback>,
    ) -> Self {
        let scene = playback.into().scene;
        let camera_buffer = CameraBuffer::new(&device);
        let msaa_texture = Self::create_msaa_texture(&device, format, (800, 600));
        let batch = Batch::new(&device, format, camera_buffer.layout());
        let renderers: Vec<ComponentRenderer> = vec![
            Box::new(circle::circles),
            Box::new(spline::splines),
            Box::new(spline::arrows),
            Box::new(glyph::glyphs),
        ];

        Self {
            device,
            queue,
            format,

            scene,
            camera_buffer,
            msaa_texture,

            batch,
            renderers,
        }
    }

    // Renders the scene to the assigned frame and presents the frame if possible.
    // Uses all renderers and supplies them the data according to their geometry
    // filter.
    pub fn render(&mut self, view: &TextureView, t: f32) {
        let size = (view.texture().width(), view.texture().height());

        // Get a valid view over our MSAA texture.
        let msaa_size = (self.msaa_texture.width(), self.msaa_texture.height());
        if size != msaa_size {
            self.msaa_texture = Self::create_msaa_texture(&self.device, self.format, size);
        }
        let msaa_view = self
            .msaa_texture
            .create_view(&TextureViewDescriptor::default());

        // Populate our batch with appropriate render commands.
        {
            let mut encoder = self.batch.encoder();
            for renderer in self.renderers.iter() {
                renderer(&mut encoder, &self.scene, t, size);
            }
        }

        // Prepare the batches with the render data.
        self.batch.prepare(&self.queue);

        // Upload the camera data into the buffer.
        let (background, view_proj) = self.scene.sample_camera(size, t);
        self.camera_buffer.upload(&self.queue, &view_proj, t);

        // Get our command encoder and build our render pass.
        let encoder_desc = CommandEncoderDescriptor {
            label: Some("nelo scene renderer encoder"),
        };
        let mut encoder = self.device.create_command_encoder(&encoder_desc);

        // Build our render pass from each of the renderers.
        {
            let [r, g, b, a] = background.to_array();
            let render_pass_desc = RenderPassDescriptor {
                label: Some("nelo pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&view),
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
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
            self.batch.submit(&mut pass);
        }

        // Submit the draw commands to the GPU.
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn create_msaa_texture(device: &Device, format: TextureFormat, size: (u32, u32)) -> Texture {
        let (width, height) = size;
        let texture_desc = TextureDescriptor {
            label: Some("nelo scene renderer msaa"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        device.create_texture(&texture_desc)
    }
}

// ----- Playback -----

pub struct Playback {
    scene: Scene,
    length: Option<f32>,
}

impl Playback {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene,
            length: None,
        }
    }

    pub fn length(&self) -> Option<f32> {
        self.length
    }

    pub fn with_length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }
}

impl From<Scene> for Playback {
    fn from(scene: Scene) -> Self {
        Self::new(scene)
    }
}

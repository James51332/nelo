//! Core app features for viewer

use crate::render::Renderer;
use crate::scene::Playback;
use crate::viewer::UiRenderer;
use egui::{CentralPanel, Frame, Id, Panel, Slider, SliderClamping, TextureId, load::SizedTexture};
use egui::{Image, Rect, Vec2};
use egui_wgpu::ScreenDescriptor;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, CurrentSurfaceTexture, Device, DeviceDescriptor,
    Extent3d, Instance, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
    TextureViewDescriptor,
};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::Window,
};

// ----- App -----

pub struct App {
    // Core
    device: Device,
    queue: Queue,
    window: Arc<Window>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,

    // Renderers
    renderer: Renderer,
    ui: UiRenderer,

    // Intermediate Texture
    target: Texture,
    view: TextureView,
    id: TextureId,

    // Locked canvas aspect, and the size in physical pixels the scene should render at, as
    // measured from the last ui pass.
    aspect: f32,
    canvas: (u32, u32),

    // Playback
    last_frame: Instant,
    time: f32,
    playing: bool,
}

impl App {
    pub async fn new(window: Arc<Window>, playback: Playback) -> Self {
        // Create our wgpu instance.
        let instance = Instance::default();

        // Get our adapter to retrieve device and queue.
        let adapter_opts = RequestAdapterOptions::default();
        let adapter = instance
            .request_adapter(&adapter_opts)
            .await
            .expect("Failed to get GPU adapter");

        let device_opts = DeviceDescriptor::default();
        let (device, queue) = adapter
            .request_device(&device_opts)
            .await
            .expect("Failed to get GPU device");

        // Create the render surface.
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create render surface");

        // Render egui in linear rgb and scene in srgb.
        let formats = surface.get_capabilities(&adapter).formats;
        let format = formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb() && f.add_srgb_suffix() != *f)
            .or_else(|| formats.first().copied())
            .expect("Adapter does not support the render surface");
        let scene_format = format.add_srgb_suffix();

        // Configure the surface.
        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Srgb,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::default(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create the scene and ui renderers.
        let renderer = Renderer::new(device.clone(), queue.clone(), scene_format, playback);
        let mut ui = UiRenderer::new(&device, format, window.clone());

        // Create canvas texture. Scene is srgb, but viewed as linear in egui.
        let (width, height) = (1280, 720);
        let texture_desc = TextureDescriptor {
            label: Some("nelo scene render texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: scene_format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        };
        let target = device.create_texture(&texture_desc);

        // The scene renders through the sRGB view, so the gpu encodes its linear colors on write.
        let view = target.create_view(&TextureViewDescriptor::default());
        let ui_view = target.create_view(&TextureViewDescriptor {
            label: Some("nelo scene ui view"),
            format: Some(format),
            ..Default::default()
        });

        let id = ui.register(&device, &ui_view);

        // Get the current time.
        let last_frame = Instant::now();

        Self {
            device,
            queue,
            window,
            surface,
            config,
            target,
            view,
            id,
            renderer,
            ui,
            aspect: width as f32 / height as f32,
            canvas: (width, height),
            last_frame,
            time: 0.0,
            playing: true,
        }
    }

    fn draw(&mut self) {
        // Try to get a swapchain texture.
        let texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) => t,
            CurrentSurfaceTexture::Suboptimal(t) => {
                self.surface.configure(&self.device, &self.config);
                t
            }
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Validation
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Lost => {
                return;
            }
        };

        // Get the time.
        let now = Instant::now();
        if self.playing {
            let elapsed = (now - self.last_frame).as_secs_f32();
            self.time += elapsed;
        }
        self.last_frame = now;

        // Create an encoder.
        let encoder_desc = CommandEncoderDescriptor::default();
        let mut encoder = self.device.create_command_encoder(&encoder_desc);

        // Make sure that our target is sized properly.
        if self.canvas != (self.target.width(), self.target.height()) {
            self.resize_canvas();
        }

        // Render the current frame into the scene texture.
        self.renderer.render(&self.view, self.time);

        // Create a view onto the swapchain image, which the ui renders to.
        let surface_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        // The scene texture is drawn at its own resolution and scaled down to fit.
        let scene = SizedTexture::new(
            self.id,
            (self.target.width() as f32, self.target.height() as f32),
        );

        // Build the ui at the window root.
        let aspect = self.aspect;
        self.ui.run(|ui| {
            Panel::top(Id::new("menu")).show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.checkbox(&mut self.playing, "Playing");
                    ui.add(
                        Slider::new(&mut self.time, 0.0..=30.0)
                            .clamping(SliderClamping::Never)
                            .step_by(0.001),
                    );
                });
            });

            CentralPanel::default()
                .frame(Frame::default().inner_margin(0.0))
                .show(ui, |ui| {
                    // Fit the locked aspect inside the available space. Sizing off `aspect`
                    // rather than the target's current size keeps the target out of the
                    // calculation that decides how big the target should be.
                    let avail = ui.available_rect_before_wrap();
                    let fitted = avail.width().min(avail.height() * aspect);
                    let size = Vec2::new(fitted, fitted / aspect);
                    let rect = Rect::from_center_size(avail.center(), size);
                    Image::from_texture(scene).paint_at(ui, rect);

                    // Calculate available canvas space.
                    let ppp = ui.ctx().pixels_per_point();
                    let width = (rect.width() * ppp).round().max(1.0);
                    self.canvas = (width as u32, (width / aspect).round().max(1.0) as u32);
                });
        });

        // Paint the ui onto the swapchain image.
        let screen = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        self.ui.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &surface_view,
            screen,
        );

        self.queue.submit(Some(encoder.finish()));

        // Present the image.
        self.queue.present(texture);
    }

    fn resize_canvas(&mut self) {
        // Egui sends like 20000 for first frame. Clamp to protect.
        let limit = self.device.limits().max_texture_dimension_2d;
        self.canvas = (self.canvas.0.clamp(1, limit), self.canvas.1.clamp(1, limit));

        let texture_desc = TextureDescriptor {
            label: Some("nelo scene render texture"),
            size: Extent3d {
                width: self.canvas.0,
                height: self.canvas.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.config.format.add_srgb_suffix(),
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[self.config.format],
        };

        self.target = self.device.create_texture(&texture_desc);
        self.view = self.target.create_view(&TextureViewDescriptor::default());

        let ui_view = self.target.create_view(&TextureViewDescriptor {
            label: Some("nelo scene ui view"),
            format: Some(self.config.format),
            ..Default::default()
        });
        self.ui.update(&self.device, &ui_view, self.id);
    }

    pub fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        // Let ui handle input first.
        self.ui.handle_input(&event);

        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(size) => {
                self.config.width = size.width;
                self.config.height = size.height;
                self.surface.configure(&self.device, &self.config);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            // Escape to quit
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            _ => (),
        };
    }
}

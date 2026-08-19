//! Viewer for nelo scenes

mod ui;

use crate::render::{Playback, Renderer};
use egui_wgpu::ScreenDescriptor;
use std::sync::Arc;
use std::time::Instant;
use ui::UiRenderer;
use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, CurrentSurfaceTexture, Device, DeviceDescriptor,
    Instance, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

// ----- State -----

pub struct State {
    device: Device,
    queue: Queue,
    window: Arc<Window>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    renderer: Renderer,
    ui: UiRenderer,

    last_frame: Instant,
    time: f32,
    playing: bool,
}

impl State {
    async fn new(window: Arc<Window>, playback: Playback) -> Self {
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

        // Get the rendering format.
        let formats = surface.get_capabilities(&adapter).formats;
        let format = formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(TextureFormat::Rgba8UnormSrgb);

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
        let renderer = Renderer::new(device.clone(), queue.clone(), format, playback);
        let ui = UiRenderer::new(&device, format, window.clone());

        // Get the current time.
        let last_frame = Instant::now();

        Self {
            device,
            queue,
            window,
            surface,
            config,
            renderer,
            ui,
            last_frame,
            time: 0.0,
            playing: true,
        }
    }

    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
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
            _ => (),
        };
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

        // Create a view.
        let view_desc = TextureViewDescriptor::default();
        let view = texture.texture.create_view(&view_desc);

        // Get the time and render.
        let now = Instant::now();
        if self.playing {
            let elapsed = (now - self.last_frame).as_secs_f32();
            self.time += elapsed;
        }
        self.last_frame = now;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        // Render the current frame.
        self.renderer.render(&view, self.time);

        // Render the ui.
        self.ui.begin_frame();

        egui::Window::new("Settings")
            .resizable([true, true])
            .show(self.ui.context(), |ui| {
                ui.label("Time");
                ui.checkbox(&mut self.playing, "Playing");

                if !self.playing {
                    ui.add(egui::Slider::new(&mut self.time, 0.0..=25.0).step_by(0.1));
                }
            });

        self.ui.end_frame_and_draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: self.window.scale_factor() as f32,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        // Present the image.
        self.queue.present(texture);
    }
}

// ----- Viewer -----

pub enum Viewer {
    Pending(Playback),
    Starting,
    Running { window: Arc<Window>, state: State },
}

impl Viewer {
    pub fn new(playback: impl Into<Playback>) -> Self {
        Self::Pending(playback.into())
    }
}

impl ApplicationHandler for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Pending(playback) = std::mem::replace(self, Self::Starting) {
            // First create the window.
            let (width, height) = (1280, 720);
            let attrs = Window::default_attributes()
                .with_title("Nelo Viewer")
                .with_inner_size(LogicalSize { width, height });

            let window = Arc::new(
                event_loop
                    .create_window(attrs)
                    .expect("Failed to create window"),
            );

            // Then create the state.
            let state = pollster::block_on(State::new(window.clone(), playback));

            // Update to running state.
            *self = Self::Running { window, state };
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Self::Running { window: _, state } = self else {
            return;
        };

        state.event(event_loop, event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Self::Running { window, state: _ } = self else {
            return;
        };

        window.request_redraw();
    }
}

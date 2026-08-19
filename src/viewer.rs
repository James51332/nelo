//! Viewer for nelo scenes

use std::sync::Arc;
use std::time::Instant;
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, Device, DeviceDescriptor, Instance, PresentMode,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
    TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{render::Renderer, story::Story};

// ----- State -----

pub struct State {
    device: Device,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    renderer: Renderer,
    start: Instant,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
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
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::default(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create the renderer.
        let renderer = Renderer::new(device.clone(), queue.clone(), format, Story::demo());

        // Get the current time.
        let start = Instant::now();

        Self {
            device,
            surface,
            config,
            renderer,
            start,
        }
    }

    fn event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
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
        let time = self.start.elapsed().as_secs_f32();
        self.renderer.render(&view, time);

        // Present the image.
        texture.present();
    }
}

// ----- Viewer -----

#[derive(Default)]
pub struct Viewer {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
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
            self.window = Some(window.clone());

            // Then create the state.
            let state = pollster::block_on(State::new(window));
            self.state = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            state.event(event_loop, event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

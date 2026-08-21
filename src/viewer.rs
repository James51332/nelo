//! Viewer for nelo scenes

mod app;
mod canvas;
mod ui;

pub use canvas::Canvas;
pub use ui::UiRenderer;
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, Device, DeviceDescriptor, Instance, PresentMode,
    Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages,
    TextureViewDescriptor,
};

use crate::scene::Playback;
use app::App;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

// ----- Viewer -----

pub enum Viewer {
    Pending(Playback),
    Starting,
    Running {
        window: Arc<Window>,
        device: Device,
        queue: Queue,
        surface: Surface<'static>,
        config: SurfaceConfiguration,
        app: App,
    },
}

impl Viewer {
    pub fn new(playback: impl Into<Playback>) -> Self {
        Self::Pending(playback.into())
    }

    pub fn draw(&mut self) {
        let Self::Running {
            window: _,
            device,
            queue,
            surface,
            config,
            app,
        } = self
        else {
            return;
        };

        // Try to get a swapchain texture.
        let texture = match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(t) => t,
            CurrentSurfaceTexture::Suboptimal(t) => {
                surface.configure(device, config);
                t
            }
            CurrentSurfaceTexture::Outdated => {
                surface.configure(device, config);
                return;
            }
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Validation
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Lost => {
                return;
            }
        };

        // Create a view onto the swapchain image, which the ui renders to.
        let surface_view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        app.render(&surface_view);

        // Present the image.
        queue.present(texture);
    }

    pub async fn start(&mut self, event_loop: &ActiveEventLoop) {
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
                .find(|f| f.is_srgb())
                .or_else(|| formats.first().copied())
                .expect("Adapter does not support the render surface");
            let ui_format = format.remove_srgb_suffix();

            // Configure the surface.
            let size = window.inner_size();
            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: ui_format,
                color_space: wgpu::SurfaceColorSpace::Srgb,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::default(),
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &config);

            // Then create the state.
            let app = App::new(
                window.clone(),
                device.clone(),
                queue.clone(),
                format,
                ui_format,
                playback,
            );

            // Update to running state.
            *self = Self::Running {
                window,
                device,
                queue,
                surface,
                config,
                app,
            };
        }
    }
}

impl ApplicationHandler for Viewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self, Self::Pending(..)) {
            pollster::block_on(self.start(event_loop));
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Self::Running {
            window: _,
            device,
            queue: _,
            surface,
            config,
            app,
        } = self
        else {
            return;
        };

        app.handle_event(&event);

        match event {
            WindowEvent::RedrawRequested => self.draw(),
            WindowEvent::Resized(size) => {
                config.width = size.width;
                config.height = size.height;
                surface.configure(device, &config);
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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Self::Running { window, .. } = self {
            window.request_redraw();
        }
    }
}

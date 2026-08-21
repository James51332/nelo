//! Core app features for viewer

use crate::render::Renderer;
use crate::scene::Playback;
use crate::viewer::{Canvas, UiRenderer};
use egui::{CentralPanel, Frame, Id, Panel, Slider, SliderClamping, TextureId, load::SizedTexture};
use egui::{Image, Rect, Vec2};
use egui_wgpu::ScreenDescriptor;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{CommandEncoderDescriptor, Device, Queue, TextureFormat, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

// ----- App -----

pub struct App {
    // Core
    device: Device,
    queue: Queue,
    window: Arc<Window>,

    // Renderers
    renderer: Renderer,
    ui: UiRenderer,

    // Intermediate Texture
    available_height: u32,
    canvas: Canvas,
    id: TextureId,

    // Playback
    last_frame: Instant,
    time: f32,
    playing: bool,
}

impl App {
    pub fn new(
        window: Arc<Window>,
        device: Device,
        queue: Queue,
        format: TextureFormat,
        ui_format: TextureFormat,
        playback: Playback,
    ) -> Self {
        // Create canvas texture. Scene is srgb, but viewed as linear in egui.
        let (width, available_height) = (1920, 1080);
        let mut canvas = Canvas::new(&device, format, ui_format, width, available_height);
        canvas.set_aspect(&device, playback.scene().aspect());

        // Create the scene renderer.
        let renderer = Renderer::new(device.clone(), queue.clone(), format, playback);

        // Create the ui renderer and register canvas texture.
        let mut ui = UiRenderer::new(&device, ui_format, window.clone());
        let id = ui.register(&device, &canvas.ui_view());

        // Get the current time.
        let last_frame = Instant::now();

        Self {
            device,
            queue,
            window,
            available_height,
            canvas,
            id,
            renderer,
            ui,
            last_frame,
            time: 0.0,
            playing: true,
        }
    }

    pub fn render(&mut self, surface_view: &TextureView) {
        // Get the time.
        let now = Instant::now();
        if self.playing {
            let elapsed = (now - self.last_frame).as_secs_f32();
            self.time += elapsed;
        }
        self.last_frame = now;

        // Make sure that our canvas size is accurate.
        if self.canvas.height() != self.available_height {
            self.canvas.resize(&self.device, self.available_height);
            self.ui.update(&self.device, self.canvas.ui_view(), self.id);
        }

        // Create an encoder.
        let encoder_desc = CommandEncoderDescriptor::default();
        let mut encoder = self.device.create_command_encoder(&encoder_desc);

        // Render the current frame into the scene texture.
        self.renderer.render(&self.canvas.view(), self.time);

        // The scene texture is drawn at its own resolution and scaled down to fit.
        let scene = SizedTexture::new(
            self.id,
            (self.canvas.width() as f32, self.canvas.height() as f32),
        );

        // Build the ui at the window root.
        self.ui.run(|ui| {
            Panel::top(Id::new("menu"))
                .frame(Frame::default().inner_margin(10.0))
                .show(ui, |ui| {
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
                    let aspect = self.canvas.aspect();
                    let avail = ui.available_rect_before_wrap();
                    let fitted = avail.width().min(avail.height() * aspect);
                    let size = Vec2::new(fitted, fitted / aspect);
                    let rect = Rect::from_center_size(avail.center(), size);
                    Image::from_texture(scene).paint_at(ui, rect);

                    // Calculate available canvas space.
                    let ppp = ui.ctx().pixels_per_point();
                    self.available_height = (rect.height() * ppp).round().max(1.0) as u32;
                });
        });

        // Paint the ui onto the swapchain image.
        let PhysicalSize { width, height } = self.window.inner_size();
        let screen = ScreenDescriptor {
            size_in_pixels: [width, height],
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
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.ui.handle_input(event);
    }
}

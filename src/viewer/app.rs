//! Core app features for viewer

use crate::export::{Export, VideoExport};
use crate::render::Renderer;
use crate::viewer::{Canvas, Catalog, UiRenderer};
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

    // Rendering
    renderer: Renderer,
    ui: UiRenderer,

    // Intermediate Texture
    available_height: u32,
    canvas: Canvas,
    id: TextureId,

    // Playback
    last_frame: Instant,
    time: f32,
    length: Option<f32>,
    playing: bool,

    // Sources,
    catalog: Box<dyn Catalog>,
    active: String,
    stale: bool,
}

impl App {
    pub fn new(
        window: Arc<Window>,
        device: Device,
        queue: Queue,
        format: TextureFormat,
        ui_format: TextureFormat,
        catalog: Box<dyn Catalog>,
        active: String,
    ) -> Self {
        // We need to generate a playback at least once.
        let playback = catalog.playback(&active).expect("Playback not found");
        let length = playback.length();

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
            length,
            playing: true,
            catalog,
            active,
            stale: false,
        }
    }

    pub fn render(&mut self, surface_view: &TextureView) {
        // See if we need to update the playback.
        if self.catalog.stale() || self.stale {
            if let Some(playback) = self.catalog.playback(&self.active) {
                // Update the canvas aspect ratio.
                self.canvas
                    .set_aspect(&self.device, playback.scene().aspect());

                // Submit to the renderer.
                self.length = playback.length();
                self.renderer.set_playback(playback);
            } else {
                log::warn!("Playback ({}) not found!", self.active);
            }

            self.stale = false;
        }

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
                            Slider::new(&mut self.time, 0.0..=self.length.unwrap_or(30.0))
                                .clamping(SliderClamping::Never)
                                .step_by(0.001),
                        );

                        // Show the dropdown.
                        let mut next = self.active.clone();
                        egui::ComboBox::from_label("Playback")
                            .selected_text(self.active.clone())
                            .show_ui(ui, |ui| {
                                for key in self.catalog.names() {
                                    ui.selectable_value(&mut next, key.clone(), key);
                                }
                            });

                        if next != self.active {
                            self.active = next;
                            self.stale = true;
                        }

                        // Show the export button.
                        if ui.button("Export").clicked() {
                            let playback = self
                                .catalog
                                .playback(&self.active)
                                .expect("Playback not found");

                            let video = VideoExport {
                                width: 3840,
                                height: 2160,
                                frame_rate: 30,
                                file_name: self.active.clone(),
                                file_ext: "mp4".into(),
                                gpu: Some((self.device.clone(), self.queue.clone())),
                                start_time: 0.0,
                                end_time: playback.length().unwrap_or(self.time),
                            };

                            std::thread::spawn(move || video.export(playback));
                        }
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

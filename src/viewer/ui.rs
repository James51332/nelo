//! Helper for rendering ui with egui.
//!
//! Adapted from MIT Licensed template by kaphula:
//! https://github.com/kaphula/winit-egui-wgpu-template/blob/master/src/egui_tools.rs

use std::sync::Arc;

use egui::{Context, FullOutput, TextureId, Ui, ViewportId};
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor, wgpu};
use egui_winit::State;
use wgpu::{
    Color, FilterMode, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct UiRenderer {
    window: Arc<Window>,
    state: State,
    renderer: Renderer,
    output: Option<FullOutput>,
}

impl UiRenderer {
    pub fn new(device: &Device, format: TextureFormat, window: Arc<Window>) -> UiRenderer {
        // Create the state.
        let context = Context::default();
        let egui_state = State::new(
            context,
            ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        // Create the renderer.
        let render_opts = RendererOptions {
            msaa_samples: 0,
            depth_stencil_format: None,
            dithering: true,
            predictable_texture_filtering: false,
        };
        let egui_renderer = Renderer::new(device, format, render_opts);

        UiRenderer {
            window,
            state: egui_state,
            renderer: egui_renderer,
            output: None,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        let _ = self.state.on_window_event(&self.window, event);
    }

    fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    /// Build a root ui via egui.
    pub fn run(&mut self, build: impl FnMut(&mut Ui)) {
        self.context()
            .set_pixels_per_point(self.window.scale_factor() as f32);
        let raw_input = self.state.take_egui_input(&self.window);
        self.output = Some(self.context().run_ui(raw_input, build));
    }

    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        let Some(mut full_output) = self.output.take() else {
            panic!("run must be called before draw can be called!");
        };

        self.state
            .handle_platform_output(&self.window, full_output.platform_output);

        let tris = self
            .context()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

        for (id, deltas) in &full_output.textures_delta.set {
            for delta in deltas {
                self.renderer.update_texture(device, queue, *id, delta);
            }
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                depth_slice: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            multiview_mask: None,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);

        // Free textures only after painting, since the pass above may still reference them.
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        // TexturesDelta panics on drop if it still holds unapplied deltas, so mark ours handled.
        full_output.textures_delta.clear();
    }

    pub fn update(&mut self, device: &Device, view: &TextureView, id: TextureId) {
        self.renderer
            .update_egui_texture_from_wgpu_texture(device, view, FilterMode::Linear, id);
    }

    /// Registers a wgpu texture with egui so it can be drawn as an [`egui::Image`].
    pub fn register(&mut self, device: &Device, view: &TextureView) -> TextureId {
        self.renderer
            .register_native_texture(device, view, FilterMode::Linear)
    }
}

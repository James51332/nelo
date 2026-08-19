//! Helper for rendering ui with egui.
//!
//! Adapted from MIT Licensed template by kaphula:
//! https://github.com/kaphula/winit-egui-wgpu-template/blob/master/src/egui_tools.rs

use std::sync::Arc;

use egui::{Context, ViewportId};
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor, wgpu};
use egui_winit::State;
use wgpu::{RenderPassColorAttachment, RenderPassDescriptor};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct UiRenderer {
    window: Arc<Window>,
    state: State,
    renderer: Renderer,
    frame_started: bool,
}

impl UiRenderer {
    pub fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

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
            frame_started: false,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        let _ = self.state.on_window_event(&self.window, event);
    }

    pub fn ppp(&mut self, v: f32) {
        self.context().set_pixels_per_point(v);
    }

    pub fn begin_frame(&mut self) {
        let raw_input = self.state.take_egui_input(&self.window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("begin_frame must be called before end_frame_and_draw can be called!");
        }

        self.ppp(screen_descriptor.pixels_per_point);

        let mut full_output = self.state.egui_ctx().end_pass();

        self.state
            .handle_platform_output(&self.window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        // Upload new and changed textures before painting. A texture can have several deltas in
        // one frame (a full upload followed by partial patches), so apply them in order.
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
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
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

        // `TexturesDelta` panics on drop if it still holds unapplied deltas, so mark ours handled.
        full_output.textures_delta.clear();

        self.frame_started = false;
    }
}

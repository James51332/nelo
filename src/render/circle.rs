//! Circle renderer: instanced, signed-distance-field filled circles.

use crate::render::{Gpu, Renderer};
use crate::scene::{Circle, Fill, Scene, Transform};
use glam::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CircleInstance {
    fill: Vec4,        // 16 bytes
    matrix2: Mat2,     // 16 bytes
    translation: Vec2, // 8 bytes
    _pad: Vec2,        // 8 bytes (48 total, 16-aligned)
}

/// Maximum circles per frame (fixed-capacity instance buffer).
const MAX_CIRCLES: u64 = 100_000;

pub struct CircleRenderer {
    pipeline: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
    count: u32,
}

impl CircleRenderer {
    pub fn new(
        gpu: &Gpu,
        camera_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nelo circle shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/circle.wgsl").into()),
            });

        let layout = gpu
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("nelo circle pipeline layout"),
                bind_group_layouts: &[Some(camera_layout)],
                immediate_size: 0,
            });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x4, // color       (offset 0)
                1 => Float32x2, // matrix col1 (offset 16)
                2 => Float32x2, // matrix col2 (offset 24)
                3 => Float32x2, // translation (offset 32)
            ],
        };

        let pipeline = gpu
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("nelo circle pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[instance_layout],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        let instances = gpu.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("nelo circle instances"),
            size: MAX_CIRCLES * std::mem::size_of::<CircleInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            instances,
            count: 0,
        }
    }
}

impl Renderer for CircleRenderer {
    fn prepare(&mut self, gpu: &Gpu, scene: &Scene, t: f32) {
        let items = scene.view_triple::<Transform, Circle, Fill>();
        let capped = items.len().min(MAX_CIRCLES as usize);
        if capped < items.len() {
            log::warn!(
                "CircleRenderer: {} circles exceeds capacity {MAX_CIRCLES}, dropping {}",
                items.len(),
                items.len() - capped
            );
        }

        let data: Vec<CircleInstance> = items[..capped]
            .iter()
            .map(|(_, transform, _, fill)| {
                let affine = transform.sample(t);
                CircleInstance {
                    fill: fill.sample(t),
                    matrix2: affine.matrix2,
                    translation: affine.translation,
                    _pad: Vec2::ZERO,
                }
            })
            .collect();

        gpu.queue()
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&data));
        self.count = capped as u32;
    }

    fn submit<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

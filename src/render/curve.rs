//! Renderes for `Curve` entities.

use crate::render::{Gpu, Renderer};
use crate::scene::{Curve, Fill, Scene, Spline, Transform};
use glam::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CurveSegment {
    point1: Vec2,
    point2: Vec2,
    color1: Vec4,
    color2: Vec4,
    width1: f32,
    width2: f32,
    _pad: Vec2,
}

const MAX_SEGMENTS: u64 = 100_000;

pub struct CurveRenderer {
    pipeline: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
    count: u32,
}

impl CurveRenderer {
    pub fn new(
        gpu: &Gpu,
        camera_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = gpu
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nelo curve shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/curve.wgsl").into()),
            });

        let layout = gpu
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("nelo circle pipeline layout"),
                bind_group_layouts: &[Some(camera_layout)],
                immediate_size: 0,
            });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CurveSegment>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // point1 (offset 0)
                1 => Float32x2, // point2 (offset 8)
                2 => Float32x4, // color1 (offset 16)
                3 => Float32x4, // color2 (offset 32)
                4 => Float32,   // width1 (offset 48)
                5 => Float32,   // width2 (offset 52)
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
            size: MAX_SEGMENTS * std::mem::size_of::<CurveSegment>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            instances,
            count: 0,
        }
    }

    fn generate_polyline(&self, spline: &Spline, start: f32, end: f32) -> Vec<(f32, Vec2)> {
        // TODO: Use a more intelligent recursive subdivision.
        const NUM_SAMPLES: u32 = 100;

        let mut out = Vec::new();
        for i in 0..NUM_SAMPLES {
            let alpha = start + (end - start) * i as f32 / (NUM_SAMPLES - 1) as f32;
            out.push((alpha, spline.sample(alpha)));
        }

        out
    }
}

impl Renderer for CurveRenderer {
    fn prepare(&mut self, gpu: &Gpu, scene: &Scene, t: f32) {
        let items = scene.view_triple::<Transform, Curve, Fill>();
        let capped = items.len().min(MAX_SEGMENTS as usize);
        if capped < items.len() {
            log::warn!(
                "CurveRenderer: {} segments exceeds capacity {MAX_SEGMENTS}, dropping {}",
                items.len(),
                items.len() - capped
            );
        }

        let data: Vec<CurveSegment> = items[..capped]
            .iter()
            .map(|(_, transform, curve, fill)| {
                let affine = transform.sample(t);
                self.generate_polyline(
                    &curve.spline.sample(t),
                    curve.start_alpha.sample(t),
                    curve.end_alpha.sample(t),
                )
                .windows(2)
                .map(|points| CurveSegment {
                    point1: affine.matrix2 * points[0].1 + affine.translation,
                    point2: affine.matrix2 * points[1].1 + affine.translation,
                    color1: fill.sample(t),
                    color2: fill.sample(t),
                    width1: curve.weight.sample(t).sample(points[0].0),
                    width2: curve.weight.sample(t).sample(points[1].0),
                    _pad: Vec2::ZERO,
                })
                .collect::<Vec<CurveSegment>>()
            })
            .flatten()
            .collect();

        gpu.queue()
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&data));
        self.count = data.len() as u32;
    }

    fn submit(&self, pass: &mut wgpu::RenderPass) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

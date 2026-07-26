//! Renderes for `Curve` entities.

use crate::render::tesselate;
use crate::render::{Gpu, Renderer};
use crate::scene::{Curve, Fill, Scene, Transform};
use bytemuck::{Pod, Zeroable};
use glam::prelude::*;
use std::mem::size_of;
use wgpu::{
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, RenderPass, RenderPipeline,
    VertexBufferLayout, VertexStepMode, vertex_attr_array,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
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
    pipeline: RenderPipeline,
    instances: Buffer,
    count: u32,
}

impl CurveRenderer {
    pub fn new(gpu: &Gpu, camera_layout: &BindGroupLayout) -> Self {
        // Create the render pipeline.
        let shader = include_str!("../shaders/curve.wgsl");
        let vertex_layout = VertexBufferLayout {
            array_stride: size_of::<CurveSegment>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x2, // point1 (offset 0)
                1 => Float32x2, // point2 (offset 8)
                2 => Float32x4, // color1 (offset 16)
                3 => Float32x4, // color2 (offset 32)
                4 => Float32,   // width1 (offset 48)
                5 => Float32,   // width2 (offset 52)
            ],
        };
        let bind_group_layouts = &[Some(camera_layout)];
        let pipeline = gpu.create_pipeline(shader, vertex_layout, bind_group_layouts);

        // Create our instance buffer.
        let buffer_desc = BufferDescriptor {
            label: Some("nelo circle instances"),
            size: size_of::<CurveSegment>() as u64 * MAX_SEGMENTS,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };
        let instances = gpu.device().create_buffer(&buffer_desc);

        Self {
            pipeline,
            instances,
            count: 0,
        }
    }
}

impl Renderer for CurveRenderer {
    fn prepare(&mut self, gpu: &Gpu, _size: (u32, u32), scene: &Scene, t: f32) {
        let items = scene.view_triple::<Transform, Curve, Fill>();
        let data: Vec<CurveSegment> = items
            .iter()
            .map(|(_, transform, curve, fill)| {
                let affine = transform.sample(t);
                tesselate::generate_polyline(
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

        let capped = data.len().min(MAX_SEGMENTS as usize);
        if capped < data.len() {
            log::warn!(
                "CurveRenderer: {} segments exceeds capacity {MAX_SEGMENTS}, dropping {}",
                data.len(),
                data.len() - capped
            );
        }

        gpu.queue()
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&data[..capped]));
        self.count = capped as u32;
    }

    fn submit(&self, pass: &mut RenderPass) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

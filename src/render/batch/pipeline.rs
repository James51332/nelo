//! Helper module to create GPU pipelines.

use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Device, FragmentState, FrontFace,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexBufferLayout, VertexState,
};

/// Helper method to create a pipeline.
pub(crate) fn create(
    device: &Device,
    format: TextureFormat,
    shader: &str,
    vertex_layout: VertexBufferLayout,
    bind_group_layouts: &[Option<&BindGroupLayout>],
) -> RenderPipeline {
    // Create the shader from the source.
    let shader_desc = ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(shader.into()),
    };
    let shader = device.create_shader_module(shader_desc);

    // In general, we'll have a camera bind group. Therefore, we attach the layout.
    let layout_desc = PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        immediate_size: 0,
    };
    let layout = device.create_pipeline_layout(&layout_desc);

    // Setup our pipeline descriptor.
    let pipeline_desc = RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: VertexState {
            module: &shader,
            entry_point: None,
            buffers: &[vertex_layout],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: None,
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    };

    device.create_render_pipeline(&pipeline_desc)
}

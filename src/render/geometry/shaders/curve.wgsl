// Camera uniform at group 0, uploaded once per frame by the driver. Layout must
// match `CameraUniform` in `camera.rs` (std140: mat4 then f32, tail-padded).
struct Camera {
    view_proj: mat4x4<f32>,
    time: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct Instance {
    @location(0) point1: vec2<f32>,
    @location(1) point2: vec2<f32>,
    @location(2) color1: vec4<f32>,
    @location(3) color2: vec4<f32>,
    @location(4) width1: f32,
    @location(5) width2: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_x: f32,
};

const EDGE_PAD: f32 = 0.1;

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: Instance) -> VertexOutput {
    // Two triangles forming a quad in space. x: [-0.5, 0.5] (width), y: [0, 1] (segment)
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-0.5, 0.0),
        vec2<f32>(0.5, 0.0),
        vec2<f32>(0.5, 1.0),
        vec2<f32>(-0.5, 0.0),
        vec2<f32>(0.5, 1.0),
        vec2<f32>(-0.5, 1.0),
    );

    let corner = corners[vid];
    let dir = normalize(inst.point2 - inst.point1);
    let normal = vec2<f32>(-dir.y, dir.x);
    let width = mix(inst.width1, inst.width2, corner.y);
    let center = mix(inst.point1, inst.point2, corner.y);
    let world = center + normal * (corner.x * (1 + EDGE_PAD) * width);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.color = mix(inst.color1, inst.color2, corner.y);
    out.local_x = corner.x;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = abs(in.local_x) - 0.5;
    let aa = fwidth(d);
    let alpha = 1.0 - smoothstep(-aa, aa, d);
    if alpha <= 0.0 {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

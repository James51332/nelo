// Instanced, SDF-filled circles — the resolution-independent path.
//
// One instance per circle: {center, radius, color}. The vertex shader expands a
// unit quad around each center and the fragment shader carves a circle out of it
// with a signed distance field, so the edge stays crisp at any zoom without
// re-tessellating. Mirrors `CircleInstance` and `Camera` on the Rust side; the
// @location indices must match the vertex buffer layout in `circle.rs`.

// Camera uniform at group 0, uploaded once per frame by the driver. Layout must
// match `CameraUniform` in `camera.rs` (std140: mat4 then f32, tail-padded).
struct Camera {
    view_proj: mat4x4<f32>,
    time: f32,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

// Per-instance vertex input; one of these per circle. @location indices match
// the `vertex_attr_array!` in `circle.rs`.
struct Instance {
    @location(0) color: vec4<f32>,
    @location(1) col1: vec2<f32>,
    @location(2) col2: vec2<f32>,
    @location(3) translation: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
};

// Grow the quad this fraction past the circle's radius. The fragment SDF fades
// the edge out at |local| == 1 + fwidth(d), a hair beyond the true radius, so
// without slack the geometry clips the outer half of the anti-aliasing. `local`
// is scaled by the same factor, keeping the edge itself at length 1.
const EDGE_PAD: f32 = 0.1;

@vertex
fn vs_main(@builtin(vertex_index) vid: u32, inst: Instance) -> VertexOutput {
    // Two triangles forming a quad in local space, padded to [-(1+pad), 1+pad].
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    let local = corners[vid] * (1.0 + EDGE_PAD);
    let world = mat2x2<f32>(inst.col1, inst.col2) * local + inst.translation;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.local = local;
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // SDF of a unit circle in local space; the quad is scaled by radius so the
    // edge falls at |local| == 1.
    let d = length(in.local) - 1.0;
    let aa = fwidth(d);
    let alpha = 1.0 - smoothstep(-aa, aa, d);
    if alpha <= 0.0 {
        discard;
    }
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

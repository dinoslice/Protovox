struct Camera {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
}

struct Transform {
    x: f32,
    y: f32,
    z: f32,
    pitch: f32,
    yaw: f32,
    roll: f32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    buffer_offset: f32,
}

var<push_constant> transform: Transform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let world_pos: vec3<f32> = vec3(transform.x, transform.y, transform.z);
    let world_scale: vec3<f32> = vec3(transform.scale_x, transform.scale_y, transform.scale_z);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4(input.position * world_scale + world_pos, 1.0);
    out.tex_coords = vec3(input.tex_coords, transform.buffer_offset);
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, input.tex_coords.xy, 2);
}
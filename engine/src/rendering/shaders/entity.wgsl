struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
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
}

var<push_constant> transform: Transform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let world_pos: vec3<f32> = vec3(transform.x, transform.y, transform.z);
    let world_scale: vec3<f32> = vec3(transform.scale_x, transform.scale_y, transform.scale_z);

    var out: VertexOutput;
    out.clip_position = vec4(input.position * world_scale + world_pos, 0.0);
    out.tex_coords = input.tex_coords;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(input.tex_coords, 0.0, 0.0);
}
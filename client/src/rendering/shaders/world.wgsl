struct Camera {
    view_proj: mat4x4<f32>,
}

@group(1) @binding(0)
var<uniform> camera: Camera;

var<push_constant> chunk_loc: vec3<i32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct FaceData {
    @location(2) data: u32,
}

// stores the output of the vertex shader
struct VertexOutput {
    // use this value as clip coordinates, represents the position of the vertex in clip space, after transformation
    // in frame-buffer space -> if window is 800x600, within [0,800), [0,600)
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) face_data: u32,
}

const FACE_BOTTOM: u32 = 0;
const FACE_TOP: u32 = 1;
const FACE_FRONT: u32 = 2;
const FACE_BACK: u32 = 3;
const FACE_LEFT: u32 = 4;
const FACE_RIGHT: u32 = 5;

const CHUNK_SIZE: vec3<u32> = vec3(32, 64, 32);

@vertex
fn vs_main(
    model: VertexInput,
    face: FaceData,
) -> VertexOutput {

    // unpack data
    let chunk_pos = vec3(
        f32(face.data >> 0 & 31),
        f32(face.data >> 5 & 63),
        f32(face.data >> 11 & 31),
    );
    let face_type = face.data >> 16 & 0x7;

    // correct face orientation
    var pos: vec3<f32> = model.position;

    if (face_type == FACE_BOTTOM) {
        pos = pos.zyx;
    } else if (face_type == FACE_TOP) {
        pos.y += 1.0;
    } else if (face_type == FACE_FRONT) {
        pos = pos.zxy;
        pos.z += 1.0;
    } else if (face_type == FACE_BACK) {
        pos = pos.xzy;
    } else if (face_type == FACE_LEFT) {
        pos = pos.yxz;
    } else if (face_type == FACE_RIGHT) {
        pos = pos.yzx;
        pos.x += 1.0;
    }

    var chunk_origin = chunk_loc * vec3<i32>(CHUNK_SIZE);

    // return result
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * vec4<f32>(pos + chunk_pos + vec3<f32>(chunk_origin), 1.0);
    out.face_data = face.data;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d_array<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { // store result in first color target
    let face_type = in.face_data >> 16 & 0x7;
    let texture_id = in.face_data >> (16 + 3) & 0xFF;

    // TODO: refactor this?
    var rotated_coords: vec2<f32>;

    switch (face_type) {
        case FACE_RIGHT, FACE_BACK: { // 180 deg
            rotated_coords = vec2(1.0 - in.tex_coords.x, 1.0 - in.tex_coords.y);
        }
        case FACE_LEFT, FACE_FRONT: { // 270 deg
            rotated_coords = vec2(in.tex_coords.y, 1.0 - in.tex_coords.x);
        }
        default: {
            rotated_coords = in.tex_coords;
        }
    }

    // return the color sampled at the texture
    return textureSample(t_diffuse, s_diffuse, rotated_coords, texture_id);
}
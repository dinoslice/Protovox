// Vertex shader
struct Camera {
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct FaceData {
    @location(2) face: vec2<u32>,
    @location(3) position: vec3<f32>,
}

//struct InstanceInput {
//    @location(2) model_matrix_0: vec4<f32>,
//    @location(3) model_matrix_1: vec4<f32>,
//    @location(4) model_matrix_2: vec4<f32>,
//    @location(5) model_matrix_3: vec4<f32>,
//};

// stores the output of the vertex shader
struct VertexOutput {
    // use this value as clip coordinates, represents the position of the vertex in clip space, after transformation
    // in frame-buffer space -> if window is 800x600, within [0,800), [0,600)
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

const FACE_BOTTOM: u32 = 0;
const FACE_TOP: u32 = 1;
const FACE_FRONT: u32 = 2;
const FACE_BACK: u32 = 3;
const FACE_LEFT: u32 = 4;
const FACE_RIGHT: u32 = 5;

@vertex
fn vs_main(
    model: VertexInput,
//    instance: InstanceInput,
    face: FaceData,
) -> VertexOutput {
//    let model_matrix = mat4x4<f32>(
//        instance.model_matrix_0,
//        instance.model_matrix_1,
//        instance.model_matrix_2,
//        instance.model_matrix_3,
//    );

    var pos: vec3<f32> = model.position;

    let f = face.face.x;

    if (face.face.x == FACE_BOTTOM) {
        pos = pos.zyx;
    } else if (face.face.x == FACE_TOP) {
        pos.y += 1.0;
    } else if (face.face.x == FACE_FRONT) {
        pos = pos.zxy;
        pos.z += 1.0;
    } else if (face.face.x == FACE_BACK) {
        pos = pos.xzy;
    } else if (face.face.x == FACE_LEFT) {
        pos = pos.yxz;
    } else if (face.face.x == FACE_RIGHT) {
        pos = pos.yzx;
        pos.x += 1.0;
    }

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    if (face.position.z != 0.0) {
        out.tex_coords = vec2<f32>(0.0, 0.0);
    }

    out.clip_position = camera.view_proj * vec4<f32>(pos + face.position, 1.0);
    return out;
}


@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { // store result in first color target
    // return the color sampled at the texture
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
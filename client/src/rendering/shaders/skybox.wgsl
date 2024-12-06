@group(0) @binding(0)
var skybox: texture_cube<f32>;
@group(0) @binding(1)
var skybox_sampler: sampler;

struct Camera {
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) clip_pos: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
      var pos: vec2<f32>;

      if(vertex_index == 0) {
        pos = vec2<f32>(-1, 3);
      } else if(vertex_index == 1) {
        pos = vec2<f32>(-1, -1);
      } else {
        pos = vec2<f32>(3, -1);
      }

      var vsOut: VertexOutput;
      vsOut.position = vec4<f32>(pos, 1, 1);
      vsOut.clip_pos = vsOut.position;
      return vsOut;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
      let ndc = in.clip_pos.xyz / in.clip_pos.w;

          // Transform NDC to view-space direction using inverse projection
          let view_dir = (camera.inv_proj * vec4<f32>(ndc, 1.0)).xyz;

          // Transform view-space direction to world-space direction using inverse view matrix
          let world_dir = (camera.inv_view * vec4<f32>(view_dir, 0.0)).xyz;

          // Normalize the world-space direction for sampling the skybox
          return textureSample(skybox, skybox_sampler, normalize(world_dir));
}
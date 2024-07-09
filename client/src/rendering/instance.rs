use glm::{Mat4, Quat, Vec3};
use na::Unit;

pub struct Instance {
    pub position: Vec3,
    pub rotation: Unit<Quat>,
}

impl Instance {
    pub fn as_raw(&self) -> [[f32; 4]; 4] {
        (Mat4::new_translation(&self.position) *  self.rotation.to_homogeneous()).into()
    }

    pub const fn buffer_desc() -> wgpu::VertexBufferLayout<'static> {
        // wgsl doesn't have vertex format for mat4, so store as 4x vec4 which will be reassembled in shader
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance, // switch to the next item in the buffer after each instance
            attributes: &ATTRIBUTES,
        }
    }
}
use glm::{Quat, Vec3};
use std::ops::{Add, Mul};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[macro_export]
macro_rules! vertex_buffer_desc {
    ($base:expr) => {
        {
            use crate::rendering::math::vertex::*;

            const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
                wgpu::vertex_attr_array![$base => Float32x3, $base + 1 => Float32x2];

            wgpu::VertexBufferLayout {
                array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            }
        }
    };
}

impl Mul<&Vec3> for Vertex {
    type Output = Vertex;

    fn mul(self, transform: &Vec3) -> Self::Output {
        Self { position: [self.position[0] * transform.x, self.position[1] * transform.y, self.position[2] * transform.z], tex_coords: self.tex_coords }
    }
}

impl Mul<&Quat> for Vertex {
    type Output = Vertex;

    fn mul(self, transform: &Quat) -> Self::Output {
        let v = Vec3::new(self.position[0], self.position[1], self.position[2]);
        let v = glm::quat_rotate_vec3(transform, &v);

        Self { position: [v.x, v.y, v.z], tex_coords: self.tex_coords }
    }
}

impl Add<&Vec3> for Vertex {
    type Output = Vertex;

    fn add(self, transform: &Vec3) -> Self::Output {
        Self { position: [self.position[0] + transform.x, self.position[1] + transform.y, self.position[2] + transform.z], tex_coords: self.tex_coords }
    }
}

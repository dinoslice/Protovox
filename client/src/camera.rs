use glm::{Mat4, Vec3};
use na::{Perspective3, UnitQuaternion};
use shipyard::Component;
use crate::components::Transform;

#[derive(Component)]
pub struct Camera {
    pub offset: Vec3,
    pub perspective: Perspective3<f32>,
}

impl Camera {
    pub fn view_matrix(&self, position: Vec3, pitch: f32, yaw: f32) -> Mat4 {
        let direction = UnitQuaternion::from_euler_angles(-pitch, yaw, 0.0) * Vec3::z_axis();

        let target = position + direction.zyx();

        glm::look_at_rh(&position, &target, &Vec3::y_axis())
    }

    pub fn as_uniform(&self, transform: &Transform) -> [[f32; 4]; 4] {
        const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        (OPENGL_TO_WGPU_MATRIX * self.perspective.as_matrix() * self.view_matrix(transform.position + self.offset, transform.pitch, transform.yaw)).into()
    }
}
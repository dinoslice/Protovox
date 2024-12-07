use glm::{Mat4, Vec3};
use na::{Perspective3, UnitQuaternion};
use shipyard::Component;

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
}
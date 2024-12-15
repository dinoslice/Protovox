use bytemuck::{Pod, Zeroable};
use glm::{Mat4, Vec3};
use na::UnitQuaternion;
use crate::camera::Camera;
use crate::components::Transform;

pub fn view_matrix(position: Vec3, pitch: f32, yaw: f32) -> Mat4 {
    let direction = UnitQuaternion::from_euler_angles(-pitch, yaw, 0.0) * Vec3::z_axis();

    let target = position + direction.zyx();

    glm::look_at_rh(&position, &target, &Vec3::y_axis())
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct ShaderCam {
    pub view: Mat4,
    pub proj: Mat4,
    pub inv_view: Mat4,
    pub inv_proj: Mat4,
    pub view_proj: Mat4,
    pub inv_view_proj: Mat4,
}

impl ShaderCam {
    pub fn from_camera_and_transform(camera: &Camera, transform: &Transform) -> Option<Self> {
        Self::new(
            *camera.perspective.as_matrix(),
            view_matrix(transform.position + camera.offset, transform.pitch, transform.yaw)
        )
    }

    pub fn new(view: Mat4, proj: Mat4) -> Option<Self> {
        let inv_view = view.try_inverse()?;
        let inv_proj = proj.try_inverse()?;
        let view_proj = view * proj;
        let inv_view_proj = view_proj.try_inverse()?;

        Some(Self { view, proj, inv_view, inv_proj, view_proj, inv_view_proj })
    }
}
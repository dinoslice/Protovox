use glm::Vec3;
use na::Perspective3;
use shipyard::Component;

#[derive(Component)]
pub struct Camera {
    pub offset: Vec3,
    pub perspective: Perspective3<f32>,
}
use glm::Vec3;
use shipyard::Component;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct LocalPlayer;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct Player;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct Entity;

#[derive(Clone, Component, Debug, Default)]
pub struct Transform {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Component, Debug, Default)]
pub struct Velocity(pub Vec3);

#[derive(Clone, Component, Debug, Default)]
pub struct PlayerSpeed(pub f32);

#[derive(Clone, Component, Debug, Default)]
pub struct Hitbox(pub Vec3);
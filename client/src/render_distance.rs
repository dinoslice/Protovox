use glm::U16Vec3;
use serde::{Deserialize, Serialize};
use shipyard::Component;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct RenderDistance(pub U16Vec3);

impl Default for RenderDistance {
    fn default() -> Self {
        Self(U16Vec3::new(4, 2, 4))
    }
}
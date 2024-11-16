use glm::U16Vec3;
use serde::{Deserialize, Serialize};
use shipyard::Component;

#[derive(Debug, Clone, Component, Serialize, Deserialize, Eq, PartialEq)]
pub struct RenderDistance(pub U16Vec3);

impl Default for RenderDistance {
    fn default() -> Self {
        Self(U16Vec3::new(4, 2, 4))
    }
}

impl RenderDistance {
    pub fn render_size(&self) -> U16Vec3 {
        self.0.map(|n| 2 * n + 1)
    }
}
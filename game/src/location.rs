use glm::Vec3;
use serde::{Deserialize, Serialize};
use crate::chunk;
use crate::chunk::location::ChunkLocation;
use shipyard::Component;

#[repr(transparent)]
#[derive(Debug, Default, Clone, Component, Serialize, Deserialize)]
pub struct WorldLocation(pub Vec3);

impl From<&ChunkLocation> for WorldLocation {
    fn from(loc: &ChunkLocation) -> Self {
        Self(loc.0.component_mul(&chunk::CHUNK_SIZE.cast()).cast())
    }
}
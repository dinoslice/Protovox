use glm::Vec3;
use crate::chunk;
use crate::chunk::location::ChunkLocation;

pub struct WorldLocation(pub Vec3);

impl From<&ChunkLocation> for WorldLocation {
    fn from(loc: &ChunkLocation) -> Self {
        Self(loc.0.component_mul(&chunk::CHUNK_SIZE.cast()).cast())
    }
}
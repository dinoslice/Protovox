use glm::{IVec3, Vec3};
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

impl From<&BlockLocation> for WorldLocation {
    fn from(loc: &BlockLocation) -> Self {
        Self(loc.0.cast())
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Component, Serialize, Deserialize)]
pub struct BlockLocation(pub IVec3);

impl From<&ChunkLocation> for BlockLocation {
    fn from(loc: &ChunkLocation) -> Self {
        Self(loc.0.component_mul(&chunk::CHUNK_SIZE.cast()))
    }
}

impl From<&WorldLocation> for BlockLocation {
    fn from(loc: &WorldLocation) -> Self {
        Self(loc.0.map(|n| n.floor() as _))
    }
}

impl From<ChunkLocation> for BlockLocation {
    fn from(loc: ChunkLocation) -> Self {
        (&loc).into()
    }
}

impl From<WorldLocation> for BlockLocation {
    fn from(loc: WorldLocation) -> Self {
        (&loc).into()
    }
}
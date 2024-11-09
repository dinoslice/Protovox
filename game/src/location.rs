use glm::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use crate::chunk;
use crate::chunk::location::ChunkLocation;
use shipyard::Component;
use crate::chunk::pos::ChunkPos;

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

impl std::ops::Add<ChunkPos> for WorldLocation {
    type Output = WorldLocation;

    fn add(self, rhs: ChunkPos) -> Self::Output {
        Self(self.0 + Vec3::new(rhs.x() as _, rhs.y() as _, rhs.z() as _))
    }
}

impl BlockLocation {
    pub fn get_aabb_bounds(&self) -> (Vec3, Vec3) {
        let min = self.0.map(|n| n as _);
        let max = min.map(|n| n + 1.0);

        (min, max)
    }
}
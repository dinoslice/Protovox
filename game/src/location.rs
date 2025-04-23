use glm::{IVec3, TVec3, Vec3};
use serde::{Deserialize, Serialize};
use crate::chunk;
use crate::chunk::location::ChunkLocation;
use shipyard::Component;
use crate::chunk::pos::ChunkPos;

#[repr(transparent)]
#[derive(Debug, Default, Clone, Component, Serialize, Deserialize, PartialEq, derive_more::Add, derive_more::Sub, derive_more::From)]
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

impl From<BlockLocation> for WorldLocation {
    fn from(loc: BlockLocation) -> Self {
        (&loc).into()
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Component, Serialize, Deserialize, Eq, PartialEq, derive_more::Add, derive_more::Sub, derive_more::From)]
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

impl BlockLocation {
    pub fn from_chunk_parts(loc: &ChunkLocation, pos: &ChunkPos) -> Self {
        let mut this = Self::from(loc);
        this.0 += TVec3::<u8>::from(pos).cast();
        this
    }

    pub fn as_chunk_parts(&self) -> (ChunkLocation, ChunkPos) {
        (self.into(), self.into())
    }

    pub fn get_aabb_bounds(&self) -> (Vec3, Vec3) {
        let min = self.0.cast();
        let max = min.map(|n| n + 1.0);

        (min, max)
    }
}
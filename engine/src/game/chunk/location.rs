use glm::IVec3;
use serde::{Deserialize, Serialize};
use crate::game::chunk::CHUNK_SIZE;
use crate::game::location::{BlockLocation, WorldLocation};

#[repr(transparent)]
#[derive(Default, Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ChunkLocation(pub IVec3);

impl From<&WorldLocation> for ChunkLocation {
    fn from(loc: &WorldLocation) -> Self {
        Self(loc.0.component_div(&CHUNK_SIZE.cast()).map(|f| f.floor() as i32))
    }
}

impl From<WorldLocation> for ChunkLocation {
    fn from(loc: WorldLocation) -> Self {
        (&loc).into()
    }
}

impl From<&BlockLocation> for ChunkLocation {
    fn from(loc: &BlockLocation) -> Self {
        Self(loc.0.map_with_location(|r, _, n| n.div_euclid(CHUNK_SIZE[r] as _)))
    }
}
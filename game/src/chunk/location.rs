use glm::IVec3;
use serde::{Deserialize, Serialize};
use crate::chunk;
use crate::location::WorldLocation;

#[repr(transparent)]
#[derive(Default, Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub struct ChunkLocation(pub IVec3);

impl From<&WorldLocation> for ChunkLocation {
    fn from(loc: &WorldLocation) -> Self {
        Self(loc.0.component_div(&chunk::CHUNK_SIZE.cast()).map(|f| f.trunc() as i32))
    }
}

impl From<WorldLocation> for ChunkLocation {
    fn from(loc: WorldLocation) -> Self {
        (&loc).into()
    }
}


use glm::TVec3;
use crate::chunk;
use crate::location::WorldLocation;

#[derive(Default, Eq, PartialEq, Clone, Debug)]
pub struct ChunkLocation(pub TVec3<i32>);

impl From<&WorldLocation> for ChunkLocation {
    fn from(loc: &WorldLocation) -> Self {
        Self(loc.0.component_div(&chunk::CHUNK_SIZE.cast()).map(|f| f.trunc() as i32))
    }
}


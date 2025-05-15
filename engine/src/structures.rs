use glm::TVec3;
use serde::{Deserialize, Serialize};
use game::block::Block;
use game::chunk::pos::ChunkPos;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    data: Box<[Block]>, // TODO: maybe switch to Arc
    size: ChunkPos,
    origin: ChunkPos,
}

impl Structure {
    pub fn new(size_v: TVec3<u8>, origin: ChunkPos) -> Option<Self> {
        let Ok(size) = ChunkPos::try_from(size_v) else {
            return None; // TODO: errors
        };

        if origin.x() >= size.x() || origin.y() >= size.y() || origin.z() >= size.z() {
            return None;
        }

        let data = vec![Block::Air; size_v.product() as _].into();

        Some(Self { data, size, origin })
    }

    fn index(&self, pos: ChunkPos) -> Option<usize> {
        let pos = TVec3::<u8>::from(pos).cast::<usize>();
        let size = TVec3::<u8>::from(self.size).cast::<usize>();


        if pos.zip_map(&size, |a, b| a.min(b)) != pos {
            return None;
        }

        Some(pos.x + pos.y * size.x + pos.z * size.x * size.y)
    }

    pub fn get(&self, index: ChunkPos) -> Option<&Block> {
        let index = self.index(index)?;

        Some(self.data.get(index).expect("should be valid index"))
    }

    pub fn get_mut(&mut self, index: ChunkPos) -> Option<&mut Block> {
        let index = self.index(index)?;

        Some(self.data.get_mut(index).expect("should be valid index"))
    }

    pub fn size(&self) -> ChunkPos {
        self.size
    }

    pub fn origin(&self) -> ChunkPos {
        self.origin
    }
}
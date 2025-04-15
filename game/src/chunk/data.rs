use serde_big_array::BigArray;
use serde::{Deserialize, Serialize};
use crate::block::Block;
use crate::chunk::BLOCKS_PER_CHUNK;
use crate::chunk::location::ChunkLocation;
use crate::chunk::pos::ChunkPos;

pub type ChunkBlocks = [Block; BLOCKS_PER_CHUNK];

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkData {
    pub location: ChunkLocation,
    #[serde(with = "BigArray")]
    pub blocks: ChunkBlocks,
}

impl ChunkData {
    pub fn empty(location: ChunkLocation) -> Self {
        Self {
            location,
            blocks: [const { Block::Air }; BLOCKS_PER_CHUNK],
        }
    }

    pub fn block_mut(&mut self, pos: ChunkPos) -> &mut Block {
        &mut self.blocks[pos.0 as usize]
    }

    pub fn block_ref(&self, pos: ChunkPos) -> &Block {
        &self.blocks[pos.0 as usize]
    }
}
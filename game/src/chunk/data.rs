use crate::block::Block;
use crate::chunk::BLOCKS_PER_CHUNK;
use crate::chunk::location::ChunkLocation;
use crate::chunk::pos::ChunkPos;

pub type ChunkBlocks = [Block; BLOCKS_PER_CHUNK];

#[derive(Debug)]
pub struct ChunkData {
    pub location: ChunkLocation,
    pub blocks: ChunkBlocks,
}

impl ChunkData {
    pub fn empty(location: ChunkLocation) -> Self {
        Self {
            location,
            blocks: [Block::Air; BLOCKS_PER_CHUNK],
        }
    }

    pub fn set_block(&mut self, pos: ChunkPos, block: Block) {
        self.blocks[pos.0 as usize] = block;
    }

    pub fn get_block(&self, pos: ChunkPos) -> Block {
        self.blocks[pos.0 as usize]
    }
}
use crate::block::Block;
use crate::chunk::BLOCKS_PER_CHUNK;
use crate::chunk::pos::ChunkPos;

pub struct ChunkData {
    pub blocks: [Block; BLOCKS_PER_CHUNK],
}

impl Default for ChunkData {
    fn default() -> Self {
        Self { blocks: [Block::Air; BLOCKS_PER_CHUNK] }
    }
}

impl ChunkData {
    pub fn set_block(&mut self, pos: ChunkPos, block: Block) {
        self.blocks[pos.0 as usize] = block;
    }

    pub fn get_block(&self, pos: ChunkPos) -> Block {
        self.blocks[pos.0 as usize].clone()
    }
}
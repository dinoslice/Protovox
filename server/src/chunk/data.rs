use crate::block::Block;
use crate::chunk::pos::ChunkPos;

pub struct ChunkData {
    pub blocks: [Block; 65536],
}

impl ChunkData {
    pub fn set_block(&mut self, pos: ChunkPos, block: Block) {
        self.blocks[pos.0 as usize] = block;
    }

    pub fn get_block(&mut self, pos: ChunkPos) -> Block {
        self.blocks[pos.0 as usize].clone()
    }
}
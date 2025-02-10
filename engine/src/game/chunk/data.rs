use serde::{Deserialize, Serialize};
use resources::ResourceKey;
use crate::base_types::block::Block;
use crate::game::chunk::BLOCKS_PER_CHUNK;
use crate::game::chunk::location::ChunkLocation;
use crate::game::chunk::pos::ChunkPos;
use serde_big_array::BigArray;

pub type ChunkBlocks = [ResourceKey<Block>; BLOCKS_PER_CHUNK];

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkData {
    pub location: ChunkLocation,
    #[serde(with = "BigArray")]
    pub blocks: ChunkBlocks,
}

impl ChunkData {
    pub fn empty(location: ChunkLocation) -> Self {
        let mut blocks = Vec::new();
        for _ in 0..BLOCKS_PER_CHUNK {
            blocks.push(ResourceKey::<Block>::default());
        }

        Self {
            location,
            blocks: ChunkBlocks::try_from(blocks).unwrap(),
        }
    }

    pub fn set_block(&mut self, pos: ChunkPos, block: ResourceKey<Block>) {
        self.blocks[pos.0 as usize] = block;
    }

    pub fn get_block(&self, pos: ChunkPos) -> ResourceKey<Block> {
        self.blocks[pos.0 as usize].clone()
    }
}
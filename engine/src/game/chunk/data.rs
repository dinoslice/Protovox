use serde::{Deserialize, Deserializer, Serialize, Serializer};
use resources::ResourceKey;
use crate::base_types::block::Block;
use crate::game::chunk::BLOCKS_PER_CHUNK;
use crate::game::chunk::location::ChunkLocation;
use crate::game::chunk::pos::ChunkPos;
use serde_big_array::Array;

pub type ChunkBlocks = [ResourceKey<Block>; BLOCKS_PER_CHUNK];

fn serialize_boxed_slice<S, T>(data: &Box<[T; BLOCKS_PER_CHUNK]>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    data.as_ref().serialize(serializer) // Serialize as a normal slice
}

fn deserialize_boxed_slice<'de, D, T>(deserializer: D) -> Result<Box<[T; BLOCKS_PER_CHUNK]>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + std::fmt::Debug,
{
    let vec: Vec<T> = Vec::deserialize(deserializer)?;
    let b: Box<[T; BLOCKS_PER_CHUNK]> = Box::try_from(vec).unwrap();
    Ok(b) // Convert back to Box<[T]>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkData {
    pub location: ChunkLocation,
    #[serde(serialize_with = "serialize_boxed_slice", deserialize_with = "deserialize_boxed_slice")]
    pub blocks: Box<[ResourceKey<Block>; BLOCKS_PER_CHUNK]>,
}

impl ChunkData {
    pub fn empty(location: ChunkLocation) -> Self {
        let mut blocks = Vec::with_capacity(BLOCKS_PER_CHUNK);

        for _ in 0..BLOCKS_PER_CHUNK {
            blocks.push(ResourceKey::<Block>::default());
        }

        Self {
            location,
            blocks: Box::try_from(blocks.into_boxed_slice()).unwrap(),
        }
    }

    pub fn set_block(&mut self, pos: ChunkPos, block: ResourceKey<Block>) {
        self.blocks[pos.0 as usize] = block;
    }

    pub fn get_block(&self, pos: ChunkPos) -> &ResourceKey<Block> {
        &self.blocks[pos.0 as usize]
    }
}
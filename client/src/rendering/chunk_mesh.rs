use std::array;
use game::block::Block;
use game::block::face_type::FaceType;
use game::chunk::BLOCKS_PER_CHUNK;
use game::chunk::data::{ChunkBlocks, ChunkData};
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use crate::chunks::chunk_manager::ChunkManager;
use crate::chunks::client_chunk::ClientChunk;
use crate::rendering::face_data::FaceData;

pub struct ChunkMeshContext<'a> {
    pub sides: [Option<&'a ChunkBlocks>; 6],
    pub center: &'a ChunkBlocks,
}


impl<'a> ChunkMeshContext<'a> {
    pub fn from_manager(chunk_mgr: &'a ChunkManager, center_chunk: &'a ChunkData) -> Self {
        let center_loc = &center_chunk.location;

        let sides = array::from_fn(|i| {
            let ft = FaceType::from_repr(i as _)
                .expect("within range");

            let new_loc = ChunkLocation(center_loc.0 + ft.as_vector());

            chunk_mgr.get_chunk_ref(&new_loc)
                .map(|c| &c.data.blocks)
        });

        Self {
            sides,
            center: &center_chunk.blocks,
        }
    }
}
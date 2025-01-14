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

// TODO: generational arena?
pub struct ChunkMesh {
    pub faces: Vec<FaceData>
}

impl ChunkMesh {
    // TODO: optimize this
    pub fn from_chunk(chunk: &ChunkData) -> Self {
        let mut faces = Vec::new();

        for pos in 0..BLOCKS_PER_CHUNK {
            let pos = ChunkPos(pos as _);

            let block = chunk.get_block(pos);

            if block == Block::Air {
                continue;
            }

            for ft in FaceType::ALL {
                if let Some(adj) = pos.adjacent_to_face(ft) {
                    if chunk.get_block(adj) != Block::Air {
                        continue;
                    }
                }

                faces.push(FaceData::new(pos, ft, block.texture_id(ft).expect("not air")));
            }
        }

        Self { faces }
    }

    #[allow(dead_code)]
    pub fn from_chunk_unoptimized(chunk: &ChunkData) -> Self {
        let non_air_block_count = chunk.blocks
            .iter()
            .filter(|&&b| b != Block::Air)
            .count();

        let mut faces = Vec::with_capacity(non_air_block_count);

        for (pos, block) in chunk.blocks.iter().enumerate().filter(|(_, &b)| b != Block::Air) {
            debug_assert_eq!(chunk.blocks.len(), u16::MAX as usize + 1, "size of ChunkBlocks changed");
            let pos = pos.try_into().expect("index should fit into u16");

            for ft in FaceType::ALL {
                faces.push(FaceData::new(
                    ChunkPos(pos),
                    ft,
                    block.texture_id(ft).expect("already filtered to not be air")
                ));
            }
        }

        Self { faces }
    }
}

pub struct ChunkMeshContext<'a> {
    pub sides: [Option<&'a ChunkBlocks>; 6],
    pub center: &'a ChunkBlocks,
}


impl<'a> ChunkMeshContext<'a> {
    pub fn from_manager(chunk_mgr: &'a ChunkManager, center_chunk: &'a ClientChunk) -> Option<Self> {
        let center_loc = &center_chunk.data.location;

        let sides = array::from_fn(|i| {
            let ft = FaceType::from_repr(i as _)
                .expect("within range");

            let new_loc = ChunkLocation(center_loc.0 + ft.as_vector());

            chunk_mgr.get_chunk_ref(&new_loc)
                .map(|c| &c.data.blocks)
        });

        let center = &center_chunk.data.blocks;

        Some(Self { sides, center })
    }
}
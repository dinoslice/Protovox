use std::array;
use game::block::Block;
use game::block::face_type::FaceType;
use game::chunk::BLOCKS_PER_CHUNK;
use game::chunk::data::{ChunkBlocks, ChunkData};
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use crate::chunks::chunk_manager::ChunkManager;
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
                .map(|c| c.data.blocks_ref())
        });

        Self {
            sides,
            center: &center_chunk.blocks_ref(),
        }
    }

    pub fn faces(&self) -> Vec<FaceData> {
        let mut faces = Vec::new();

        for pos in 0..BLOCKS_PER_CHUNK {
            let pos = ChunkPos(pos as _);

            let block = self.center.get(pos.0 as usize).expect("should be in range");

            if *block == Block::Air {
                continue;
            }

            for ft in FaceType::ALL {
                fn adj_is_air(blocks: &ChunkBlocks, adj: ChunkPos) -> bool {
                    *blocks.get(adj.0 as usize).expect("in range") == Block::Air
                }

                match pos.adjacent_to_face(ft) {
                    // in range
                    Ok(adj) => {
                        if !adj_is_air(self.center, adj) {
                            continue;
                        }
                    }
                    Err(adj) => {
                        if !self.sides[ft as usize].is_none_or(|blocks| adj_is_air(blocks, adj)) {
                            continue;
                        }
                    }
                }

                faces.push(FaceData::new(pos, ft, block.texture_id(ft).expect("not air")));
            }
        }

        faces
    }
}


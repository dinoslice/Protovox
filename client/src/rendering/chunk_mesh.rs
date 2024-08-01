use game::block::Block;
use game::block::face_type::FaceType;
use game::chunk::BLOCKS_PER_CHUNK;
use game::chunk::data::ChunkData;
use game::chunk::pos::ChunkPos;
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

                faces.push(FaceData::new(pos, ft, block.texture_id(ft)));
            }
        }

        Self { faces }
    }

    pub fn from_chunk_unoptimized(chunk: &ChunkData) -> Self {
        let non_air_block_count = chunk.blocks
            .iter()
            .filter(|&&b| b != Block::Air)
            .count();

        let mut faces = Vec::with_capacity(non_air_block_count);

        for (pos, block) in chunk.blocks.iter().enumerate().filter(|(_, &b)| b != Block::Air) {
            let pos = pos.try_into().unwrap();

            for ft in FaceType::ALL {
                faces.push(FaceData::new(
                    ChunkPos(pos),
                    ft,
                    block.texture_id(ft)
                ));
            }
        }

        Self { faces }
    }
}
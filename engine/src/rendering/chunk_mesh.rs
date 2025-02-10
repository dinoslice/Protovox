use shipyard::UniqueView;
use resources::Registry;
use crate::base_types::AIR;
use crate::game::chunk::BLOCKS_PER_CHUNK;
use crate::game::chunk::data::ChunkData;
use crate::game::chunk::pos::ChunkPos;
use crate::game::face_type::FaceType;
use crate::rendering::face_data::FaceData;

// TODO: generational arena?
pub struct ChunkMesh {
    pub faces: Vec<FaceData>
}

impl ChunkMesh {
    // TODO: optimize this
    pub fn from_chunk(chunk: &ChunkData, registry: &Registry) -> Self {
        let mut faces = Vec::new();

        for pos in 0..BLOCKS_PER_CHUNK {
            let pos = ChunkPos(pos as _);

            let block = chunk.get_block(pos);

            if block == AIR.clone() {
                continue;
            }

            for ft in FaceType::ALL {
                if let Some(adj) = pos.adjacent_to_face(ft) {
                    if chunk.get_block(adj) != AIR.clone() {
                        continue;
                    }
                }

                let tex = registry.get(&block).unwrap();
                let tex = tex.get_texture(ft, registry);
                let tex = registry.get(&tex).unwrap();

                faces.push(FaceData::new(pos, ft, tex.atlas_id));
            }
        }

        Self { faces }
    }

    #[allow(dead_code)]
    pub fn from_chunk_unoptimized(chunk: &ChunkData, registry: &Registry) -> Self {
        let non_air_block_count = chunk.blocks
            .iter()
            .filter(|b| **b != AIR.clone())
            .count();

        let mut faces = Vec::with_capacity(non_air_block_count);

        for (pos, block) in chunk.blocks.iter().enumerate().filter(|(_, b)| **b != AIR.clone()) {
            assert_eq!(chunk.blocks.len(), u16::MAX as usize + 1, "size of ChunkBlocks changed");
            let pos = pos.try_into().expect("index should fit into u16");

            for ft in FaceType::ALL {
                let tex = registry.get(&block).unwrap();
                let tex = tex.get_texture(ft, registry);
                let tex = registry.get(&tex).unwrap();

                faces.push(FaceData::new(
                    ChunkPos(pos),
                    ft,
                    tex.atlas_id
                ));
            }
        }

        Self { faces }
    }
}
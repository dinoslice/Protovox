use game::block::Block;
use game::chunk::data::ChunkData;
use game::chunk::pos::ChunkPos;
use crate::rendering::face_data::{ FaceData, FaceType };

pub trait VoxelRenderable {
    fn as_faces(&self) -> Vec<FaceData>;
}

impl VoxelRenderable for ChunkData {
    fn as_faces(&self) -> Vec<FaceData> {
        let non_air_block_count = self.blocks
            .iter()
            .filter(|&&b| b != Block::Air)
            .count();

        let mut face_data = Vec::with_capacity(non_air_block_count);

        for (pos, block) in self.blocks.iter().enumerate().filter(|(_, &b)| b != Block::Air) {
            let pos = pos.try_into().unwrap();

            for ft in FaceType::ALL {
                face_data.push(FaceData::new(
                    ChunkPos(pos),
                    ft,
                    block.texture_id(ft)
                ));
            }
        }

        face_data
    }
}
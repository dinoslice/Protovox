use std::array;
use crate::block::face_type::FaceType;
use crate::chunk::CHUNK_SIZE;
use crate::chunk::pos::ChunkPos;

impl ChunkPos {
    pub fn adjacent_to_face(&self, face: FaceType) -> Result<Self, Self> {
        let [dx, dy, dz]: [i8; 3] = face.as_vector().into();

        let new_x = self.x()
            .wrapping_add(dx as u8);

        let new_y = self.y()
            .wrapping_add(dy as u8);

        let new_z = self.z()
            .wrapping_add(dz as u8);

        Self::new(new_x, new_y, new_z).or_else(|_| {
            let new = [new_x, new_y, new_z];

            let [x, y, z] = array::from_fn(|i| new[i] % CHUNK_SIZE[i]);

            Ok(ChunkPos::new(x, y, z).expect("in range"))
        })
    }
}
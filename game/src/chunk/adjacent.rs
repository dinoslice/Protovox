use crate::block::face_type::FaceType;
use crate::chunk::pos::ChunkPos;

impl ChunkPos {
    pub fn adjacent_to_face(&self, face: FaceType) -> Option<Self> {
        use FaceType as FT;

        let (dx, dy, dz): (i8, i8, i8) = match face {
            FT::Bottom => (0, -1, 0),
            FT::Top => (0, 1, 0),
            FT::Front => (0, 0, 1),
            FT::Back => (0, 0, -1),
            FT::Left => (-1, 0, 0),
            FT::Right => (1, 0, 0),
        };

        let new_x = self.x()
            .wrapping_add(dx as u8);

        let new_y = self.y()
            .wrapping_add(dy as u8);

        let new_z = self.z()
            .wrapping_add(dz as u8);

        Self::new(new_x, new_y, new_z).ok()
    }
}
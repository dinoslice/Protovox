pub type TextureId = usize;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FaceType {
    Bottom, // Y-
    Top, // Y+
    Front, // Z+
    Back, // Z-
    Left, // X-
    Right, // X+
}

impl FaceType {
    pub const ALL: [FaceType; 6] = [FaceType::Top, FaceType::Bottom, FaceType::Left, FaceType::Right, FaceType::Front, FaceType::Back];
}
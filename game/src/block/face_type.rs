use glm::TVec3;
use num_traits::{One, Signed, Zero};
use strum::FromRepr;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr)]
pub enum FaceType { // TODO: rename to direction
    Bottom, // Y-
    Top, // Y+
    Front, // Z+
    Back, // Z-
    Left, // X-
    Right, // X+
}

impl FaceType {
    pub const ALL: [FaceType; 6] = [FaceType::Top, FaceType::Bottom, FaceType::Left, FaceType::Right, FaceType::Front, FaceType::Back];
    pub const POS: [FaceType; 3] = [FaceType::Right, FaceType::Top, FaceType::Front];

    pub fn as_vector<T: One + Zero + Signed>(self) -> TVec3<T> {
        use FaceType as FT;

        match self {
            FT::Bottom => TVec3::new(T::zero(), -T::one(), T::zero()),
            FT::Top => TVec3::new(T::zero(), T::one(), T::zero()),
            FT::Front => TVec3::new(T::zero(), T::zero(), T::one()),
            FT::Back => TVec3::new(T::zero(), T::zero(), -T::one()),
            FT::Left => TVec3::new(-T::one(), T::zero(), T::zero()),
            FT::Right => TVec3::new(T::one(), T::zero(), T::zero()),
        }
    }
}
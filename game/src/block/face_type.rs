use std::ops::Neg;
use glm::TVec3;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use strum::FromRepr;

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, PartialEq, Eq, Deserialize, Serialize)]
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

impl Neg for FaceType {
    type Output = Self;

    fn neg(self) -> Self::Output {
        use FaceType as FT;

        match self {
            FT::Bottom => FT::Top,
            FT::Top => FT::Bottom,
            FT::Front => FT::Back,
            FT::Back => FT::Front,
            FT::Left => FT::Right,
            FT::Right => FT::Left,
        }
    }
}
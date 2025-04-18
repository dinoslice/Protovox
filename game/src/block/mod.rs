use std::num::NonZeroU8;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumDiscriminants};
use crate::block::face_type::{Axis, FaceType};
use static_assertions::const_assert;
use crate::item::{ItemStack, ItemType};
use crate::location::BlockLocation;
use crate::texture_ids::TextureId;

pub mod face_type;

#[repr(u16)] // TODO: eventually replace strum::EnumCount with std::mem::variant_count
#[derive(Clone, Eq, PartialEq, Default, Debug, Deserialize, Serialize, EnumCount, EnumDiscriminants)]
#[strum_discriminants(name(BlockTy), derive(strum::FromRepr, EnumCount))]
pub enum Block {
    #[default]
    Air,
    Grass,
    Dirt,
    Cobblestone,
    Stone,
    Log { rotation: Axis },
    Leaf,
    Debug,
}

impl Default for BlockTy {
    fn default() -> Self {
        Self::Air
    }
}

const_assert!(size_of::<Block>() <= 16);

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TextureType {
    Single,
    Full,
    TopSides,
    UniqueTops,
}

impl Block {
    pub fn texture_id(&self, face_type: FaceType) -> Option<TextureId> {
        use crate::texture_ids::*;

        let id = match self {
            Block::Air => return None,
            Block::Grass => match face_type {
                FaceType::Top => GRASS_TOP,
                FaceType::Bottom => DIRT,
                _ => GRASS_SIDE,
            }
            Block::Dirt => DIRT,
            Block::Cobblestone => COBBLE,
            Block::Debug => match face_type.axis() {
                Axis::X => DEBUG_RED,
                Axis::Y => DEBUG_BLUE,
                Axis::Z => DEBUG_GREEN,
            }
            Block::Log { rotation } => if face_type.axis() == *rotation {
                DEBUG_RED
            } else {
                LOG // TODO: rotate texture
            }
            Block::Stone | Block::Leaf => todo!()
        };

        Some(id)
    }

    // TODO: take in self
    pub fn on_break(self, /* break_context: BreakContext TODO: break context for fortune*/) -> Option<ItemStack> {
        use Block as B;
        use ItemType as I;

        match self {
            B::Air | B::Debug => None,
            B::Grass | B::Dirt  => Some(I::Dirt.default_one()),
            B::Cobblestone | B::Stone => Some(I::Cobblestone.default_one()),
            B::Log { .. } => Some(I::Log.default_one()),
            B::Leaf => {
                let count = thread_rng().gen_range(5..15);

                Some(I::LeafPile.default_stack(NonZeroU8::new(count).expect("0 is not in range")))
            }
        }
    }

    pub fn ty(&self) -> BlockTy {
        self.into()
    }
}

impl BlockTy {
    pub fn place(self, _loc: BlockLocation, face: FaceType) -> Option<Block> {
        match self {
            Self::Air => None,
            Self::Grass => Some(Block::Grass),
            Self::Dirt => Some(Block::Dirt),
            Self::Cobblestone => Some(Block::Cobblestone),
            Self::Stone => Some(Block::Stone),
            Self::Log => Some(Block::Log { rotation: face.axis() }),
            Self::Leaf => Some(Block::Leaf),
            Self::Debug => Some(Block::Debug),
        }
    }
}
use std::num::NonZeroU8;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumDiscriminants};
use crate::block::face_type::{Axis, FaceType};
use static_assertions::const_assert;
use crate::inventory::Inventory;
use crate::item::{ItemStack, ItemType};
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
    Crate { inventory: BlockInventory<36> },
    StoneBrick,
    Planks,
    Water,
    HematiteDeposit,
}

#[serde_with::serde_as]
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct BlockInventory<const N: usize>(
    #[serde_as(as = "Box<[_; N]>")]
    Box<[Option<ItemStack>; N]>,
);

impl<const N: usize> Default for BlockInventory<N> {
    fn default() -> Self {
        Self(Box::new([const { None }; N]))
    }
}

impl<const N: usize> Inventory for BlockInventory<N> {
    fn as_slice(&self) -> &[Option<ItemStack>] {
        self.0.as_slice()
    }

    fn as_mut_slice(&mut self) -> &mut [Option<ItemStack>] {
        self.0.as_mut_slice()
    }
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
        use TextureId as Id;

        let id = match self {
            Block::Air => return None,
            Block::Grass => match face_type {
                FaceType::Top => Id::Grass,
                FaceType::Bottom => Id::Dirt,
                _ => Id::GrassSide,
            }
            Block::Dirt => Id::Dirt,
            Block::Cobblestone => Id::Cobblestone,
            Block::Debug => match face_type.axis() {
                Axis::X => Id::DebugRed,
                Axis::Y => Id::DebugBlue,
                Axis::Z => Id::DebugGreen,
            }
            Block::Log { rotation } => if face_type.axis() == *rotation {
                Id::LogTop
            } else {
                Id::LogSide // TODO: rotate texture
            }
            Block::Leaf => Id::DebugGreen,
            Block::Stone => Id::Stone,
            Block::Crate { .. } => match face_type {
                FaceType::Top => Id::CrateTop,
                FaceType::Bottom => Id::CrateBottom,
                _ => Id::CrateSide,
            },
            Block::Water => Id::Water,
            Block::Planks => Id::Planks,
            Block::StoneBrick | Block::HematiteDeposit => Id::Missing,
        };

        Some(id)
    }

    // TODO: this should return a vec?
    pub fn on_break(self, /* break_context: BreakContext TODO: break context for fortune*/) -> Vec<ItemStack> {
        use Block as B;
        use ItemType as I;

        const NONE: Vec<ItemStack> = Vec::new();

        match self {
            B::Air | B::Debug | B::Water => NONE,
            B::Grass | B::Dirt  => vec![I::Dirt.default_one()],
            B::Cobblestone | B::Stone => vec![I::Cobblestone.default_one()],
            B::Log { .. } => vec![I::Log.default_one()],
            B::Leaf => {
                let count = thread_rng().gen_range(5..15);

                vec![I::LeafPile.default_item().with_count(NonZeroU8::new(count).expect("0 is not in range"))]
            }
            B::Crate { inventory: mut inv } => {
                inv.try_insert(I::Crate.default_one())
                    .into_iter()
                    .chain(inv.0.into_iter().flatten())
                    .collect()
            },
            B::Planks => vec![I::Planks.default_one()],
            B::StoneBrick => vec![I::StoneBricks.default_one()],
            B::HematiteDeposit => {
                let count = thread_rng().gen_range(8..30);

                vec![I::HematiteNuggets.default_item().with_count(NonZeroU8::new(count).expect("0 is not in range"))]
            }
        }
    }

    pub fn ty(&self) -> BlockTy {
        self.into()
    }
}
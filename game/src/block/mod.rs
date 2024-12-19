use std::num::NonZeroU8;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
use crate::block::face_type::FaceType;
use crate::item::{ItemStack, ItemType};

pub mod face_type;

#[repr(u16)] // TODO: eventually replace strum::EnumCount with std::mem::variant_count
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Deserialize, Serialize, EnumCount, FromRepr)]
pub enum Block {
    #[default]
    Air = 0,
    Grass,
    Dirt,
    Cobblestone,
    Stone,
    Log,
    Leaf,
    Debug
}

#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TextureType {
    Single,
    Full,
    TopSides,
    UniqueTops,
}

pub type TextureId = u8;

impl Block {
    pub const fn texture_id(&self, face_type: FaceType) -> Option<TextureId> {
        const COBBLE_TEXTURE: TextureId = 0;
        const DIRT_TEXTURE: TextureId = 1;
        const GRASS_TOP_TEXTURE: TextureId = 2;
        const GRASS_SIDE_TEXTURE: TextureId = 3;

        const DEBUG_RED: TextureId = 5;
        const DEBUG_GREEN: TextureId = 6;
        const DEBUG_BLUE: TextureId = 7;

        let id = match self {
            Block::Air => return None,
            Block::Grass => match face_type {
                FaceType::Top => GRASS_TOP_TEXTURE,
                FaceType::Bottom => DIRT_TEXTURE,
                _ => GRASS_SIDE_TEXTURE,
            }
            Block::Dirt => DIRT_TEXTURE,
            Block::Cobblestone => COBBLE_TEXTURE,
            Block::Debug => match face_type {
                FaceType::Left | FaceType::Right => DEBUG_RED,
                FaceType::Bottom | FaceType::Top => DEBUG_BLUE,
                FaceType::Front | FaceType::Back => DEBUG_GREEN,
            },
            Block::Log => DEBUG_RED,
            Block::Leaf => DEBUG_GREEN,
            Block::Stone => DEBUG_RED,
        };

        Some(id)
    }

    pub fn on_break(&self, /* break_context: BreakContext TODO: break context for fortune*/) -> Option<ItemStack> {
        use Block as B;
        use ItemType as I;

        match self {
            B::Air | B::Debug => None,
            B::Grass | B::Dirt  => Some(I::Dirt.default_one()),
            B::Cobblestone | B::Stone => Some(I::Cobblestone.default_one()),
            B::Log => Some(I::Log.default_one()),
            B::Leaf => {
                let count = thread_rng().gen_range(5..15);

                Some(I::LeafPile.default_stack(NonZeroU8::new(count).expect("0 is not in range")))
            }
        }
    }
    
    pub fn placeable(&self) -> bool {
        match self {
            Block::Air => false,
            Block::Grass => true,
            Block::Dirt => true,
            Block::Cobblestone => true,
            Block::Debug => true,
            Block::Stone => true,
            Block::Log => true,
            Block::Leaf => true,
        }
    }
}
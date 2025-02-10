use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
use crate::block::face_type::FaceType;

pub mod face_type;

// #[repr(u16)] // TODO: eventually replace strum::EnumCount with std::mem::variant_count
// #[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Deserialize, Serialize, EnumCount, FromRepr)]
// pub enum Block {
//     #[default]
//     Air = 0,
//     Grass,
//     Dirt,
//     Cobblestone,
//     Stone,
//     Log,
//     Leaf,
//     Debug
// }

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
    pub fn texture_id(&self, face_type: FaceType) -> TextureId {
        const AIR_TEXTURE: TextureId = 0;
        const COBBLE_TEXTURE: TextureId = 1;
        const DIRT_TEXTURE: TextureId = 2;
        const GRASS_TOP_TEXTURE: TextureId = 3;
        const GRASS_SIDE_TEXTURE: TextureId = 4;

        const DEBUG_RED: TextureId = 6;
        const DEBUG_GREEN: TextureId = 7;
        const DEBUG_BLUE: TextureId = 8;
        const LOG_TEXTURE: TextureId = 9;

        match self {
            Block::Air => AIR_TEXTURE,
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
            Block::Log => LOG_TEXTURE,
            Block::Leaf => DEBUG_GREEN,
            Block::Stone => DEBUG_RED,
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
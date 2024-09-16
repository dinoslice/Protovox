use serde::{Deserialize, Serialize};
use crate::block::face_type::FaceType;

pub mod face_type;

#[repr(u16)]
#[derive(Clone, Copy, Eq, PartialEq, Default, Debug, Deserialize, Serialize)]
pub enum Block {
    #[default]
    Air = 0,
    Grass,
    Dirt,
    Cobblestone,
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
    pub fn texture_id(&self, face_type: FaceType) -> TextureId {
        const AIR_TEXTURE: TextureId = 0;
        const COBBLE_TEXTURE: TextureId = 1;
        const DIRT_TEXTURE: TextureId = 2;
        const GRASS_TOP_TEXTURE: TextureId = 3;
        const GRASS_SIDE_TEXTURE: TextureId = 4;

        match self {
            Block::Air => AIR_TEXTURE,
            Block::Grass => match face_type {
                FaceType::Top => GRASS_TOP_TEXTURE,
                FaceType::Bottom => DIRT_TEXTURE,
                _ => GRASS_SIDE_TEXTURE,
            }
            Block::Dirt => DIRT_TEXTURE,
            Block::Cobblestone => COBBLE_TEXTURE,
        }
    }

}
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumDiscriminants};
use crate::block::face_type::{Axis, FaceType};

pub mod face_type;

#[repr(u16)] // TODO: eventually replace strum::EnumCount with std::mem::variant_count
#[derive(Clone, Eq, PartialEq, Default, Debug, Deserialize, Serialize, EnumCount, EnumDiscriminants)]
#[strum_discriminants(name(BlockTy))]
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
    pub fn texture_id(&self, face_type: FaceType) -> Option<TextureId> {
        const COBBLE_TEXTURE: TextureId = 0;
        const DIRT_TEXTURE: TextureId = 1;
        const GRASS_TOP_TEXTURE: TextureId = 2;
        const GRASS_SIDE_TEXTURE: TextureId = 3;

        const DEBUG_RED: TextureId = 5;
        const DEBUG_GREEN: TextureId = 6;
        const DEBUG_BLUE: TextureId = 7;
        const LOG_TEXTURE: TextureId = 8;

        let id = match self.ty() {
            BlockTy::Air => return None,
            BlockTy::Grass => match face_type {
                FaceType::Top => GRASS_TOP_TEXTURE,
                FaceType::Bottom => DIRT_TEXTURE,
                _ => GRASS_SIDE_TEXTURE,
            }
            BlockTy::Dirt => DIRT_TEXTURE,
            BlockTy::Cobblestone => COBBLE_TEXTURE,
            BlockTy::Debug => match face_type {
                FaceType::Left | FaceType::Right => DEBUG_RED,
                FaceType::Bottom | FaceType::Top => DEBUG_BLUE,
                FaceType::Front | FaceType::Back => DEBUG_GREEN,
            }
            BlockTy::Log => LOG_TEXTURE,
            BlockTy::Stone | BlockTy::Leaf => todo!()
        };

        Some(id)
    }
    
    pub fn ty(&self) -> BlockTy {
        self.into()
    }
    
    pub fn placeable(&self) -> bool {
        match self.ty() {
            BlockTy::Air => false,
            BlockTy::Grass => true,
            BlockTy::Dirt => true,
            BlockTy::Cobblestone => true,
            BlockTy::Debug => true,
            BlockTy::Stone => true,
            BlockTy::Log => true,
            BlockTy::Leaf => true,
        }
    }
}
use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
use crate::block::dyn_block::{BlockDescriptor, ItemStack, TODO};
use crate::block::face_type::FaceType;

pub mod face_type;
mod dyn_block;

pub struct BlockDyn(Box<dyn BlockDescriptor>);

impl BlockDyn {
    pub fn of(block: impl BlockDescriptor) -> Self {
        Self(Box::new(block))
    }
    pub fn is<T: BlockDescriptor>(&self) -> bool {
        self.0.is::<T>()
    }
    
    pub fn try_ref<T: BlockDescriptor>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
    
    pub fn try_mut<T: BlockDescriptor>(&mut self) -> Option<&mut T> {
        self.0.downcast_mut()
    }
    
    pub fn try_into<T: BlockDescriptor>(self) -> Result<T, Self> {
        self.try_into_boxed().map(|ok| *ok)
    }
    
    pub fn try_into_boxed<T: BlockDescriptor>(self) -> Result<Box<T>, Self> {
        self.0
            .downcast::<T>()
            .map_err(|err| Self(err))
    }
}

impl BlockDescriptor for BlockDyn {
    fn uuid(&self) -> u128 {
        self.0.uuid()
    }

    fn on_break(&self) -> Option<ItemStack> {
        self.0.on_break()
    }

    fn placeable(&self) -> bool {
        self.0.placeable()
    }

    fn raycast_ty(&self) -> TODO {
        self.0.raycast_ty()
    }

    fn texture(&self) -> TODO {
        self.0.texture()
    }
}

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
        const LOG_TEXTURE: TextureId = 8;

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
            }
            Block::Log => LOG_TEXTURE,
            Block::Stone | Block::Leaf => todo!()
        };

        Some(id)
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
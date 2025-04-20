use std::fmt::Debug;
use std::num::NonZeroU8;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
use crate::block::Block;
use crate::block::face_type::FaceType;
use crate::location::BlockLocation;
use crate::texture_ids::TextureId;

#[repr(u16)] // TODO: eventually replace strum::EnumCount with std::mem::variant_count
#[derive(Clone, Copy, Eq, PartialEq, Debug, Deserialize, Serialize, EnumCount, FromRepr)]
pub enum ItemType {
    Grass,
    Dirt,
    Cobblestone,
    Stone,
    Log,
    LeafPile,
}

impl ItemType {
    pub fn default_item(self) -> Item {
        use ItemType as IT;
        
        match self {
            IT::Grass => Item {
                ty: self,
                title: "Grass".into(),
                desc: "very grassy".into(),
                data: None,
            },
            IT::Dirt => Item {
                ty: self,
                title: "Dirt".into(),
                desc: "dirt".into(),
                data: None,
            },
            IT::Cobblestone => Item {
                ty: self,
                title: "Cobblestone".into(),
                desc: "the rocky form of stone".into(),
                data: None,
            },
            IT::Stone => Item {
                ty: self,
                title: "Stone".into(),
                desc: "found underground".into(),
                data: None,
            },
            IT::Log => Item {
                ty: self,
                title: "Log".into(),
                desc: "the basic building material".into(),
                data: None,
            },
            IT::LeafPile => Item {
                ty: self,
                title: "Leaf Pile".into(),
                desc: "gathered from trees".into(),
                data: None,
            },
        }
    }
    
    pub fn default_one(self) -> ItemStack {
        ItemStack::one(self.default_item())
    }
    
    pub const fn texture_id(self) -> TextureId {
        use ItemType as IT;
        use crate::texture_ids::*;

        match self {
            IT::Grass => GRASS_SIDE,
            IT::Dirt => DIRT,
            IT::Cobblestone => COBBLE,
            IT::Log => LOG,
            IT::LeafPile => DEBUG_GREEN,
            IT::Stone => MISSING,
        }
    }
}

// TODO: move name and description into item data provider
// TODO: make this clone
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ItemStack {
    pub item: Item,
    pub count: NonZeroU8,
}

impl ItemStack {
    pub const MAX_STACK: NonZeroU8 = NonZeroU8::MAX;

    pub fn one(item: Item) -> Self {
        Self {
            item,
            count: NonZeroU8::new(1).expect("shouldn't be zero")
        }
    }

    pub fn try_combine(&mut self, rhs: Self) -> Option<Self> {
        if self.item != rhs.item {
            return Some(rhs);
        }

        let lhs_ct = self.count.get();
        let rhs_ct = rhs.count.get();

        if lhs_ct > Self::MAX_STACK.get() {
            return Some(rhs);
        }

        match Self::MAX_STACK.get() - lhs_ct {
            0 => Some(rhs),
            n if n >= rhs_ct => {
                self.count = NonZeroU8::new(lhs_ct + rhs_ct)
                    .expect("should be nonzero");

                None
            }
            n => {
                let rem = rhs_ct - n;

                self.count = Self::MAX_STACK;

                let count = NonZeroU8::new(rem)
                    .expect("if it was zero, should've been handled in case above");

                Some(rhs.item.with_count(count))
            }
        }
    }
    
    pub fn split_exact(self, first_ct: NonZeroU8) -> Result<(Self, Option<Self>), Self> {
        if first_ct <= self.count {
            Ok(self.split_at_most(first_ct))
        } else {
            Err(self)
        }
    }
    
    pub fn split_at_most(mut self, first_ct: NonZeroU8) -> (Self, Option<Self>) {
        if first_ct >= self.count {
            (self, None)
        } else {
            let mut other = self.clone();
            
            other.count = NonZeroU8::new(self.count.get() - first_ct.get()).expect("can't be zero, since first_ct must be less than total");
            
            self.count = first_ct;

            (self, Some(other))
        }
    }
    
    pub fn split_half(self) -> (Self, Option<Self>) {
        let ct = self.count.get().div_ceil(2);
        
        self.split_at_most(NonZeroU8::new(ct).expect("shouldn't ever be zero"))
    }
    
    pub fn split_item(self) -> (Item, Option<Self>) {
        let (item, res) = self.split_at_most(1.try_into().expect("nonzero"));

        (item.item, res)
    }
}

// TODO: i don't like this
#[derive(Debug)]
pub struct Item {
    pub ty: ItemType,
    pub title: String,
    pub desc: String,
    pub data: Option<Box<dyn ItemDataProvider>>,
}

impl Item {
    pub fn without_data(ty: ItemType, title: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            ty,
            title: title.into(),
            desc: desc.into(),
            data: None,
        }
    }
    
    pub fn with_count(self, count: NonZeroU8) -> ItemStack {
        ItemStack {
            item: self,
            count,
        }
    }
    
    pub fn stack_one(self) -> ItemStack {
        ItemStack::one(self)
    }

    // TODO: should this have a similar discriminant design to block
    pub fn place(self, _loc: BlockLocation, face: FaceType) -> Result<Block, Self> {
        use ItemType as IT;
        
        match self.ty {
            IT::Grass => Ok(Block::Grass),
            IT::Dirt => Ok(Block::Dirt),
            IT::Cobblestone => Ok(Block::Cobblestone),
            IT::Stone => Ok(Block::Stone),
            IT::Log => Ok(Block::Log { rotation: face.axis() }),
            IT::LeafPile => Ok(Block::Leaf),
        }
    }
}

impl PartialEq<Self> for Item {
    fn eq(&self, rhs: &Self) -> bool {
        let data_eq = match (&self.data, &rhs.data) {
            (None, None) => true,
            (Some(rhs), Some(lhs)) => rhs.hash() == lhs.hash(),
            _ => false,
        };

        self.ty == rhs.ty && self.title == rhs.title && self.desc == rhs.desc && data_eq
    }
}

impl Eq for Item {}

impl Clone for Item {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty,
            title: self.title.clone(),
            desc: self.desc.clone(),
            data: self.data.as_ref().map(|d| d.clone_boxed()),
        }
    }
}

pub trait ItemDataProvider: Debug + Send + Sync {
    // TODO: better way to check for equality
    fn hash(&self) -> u64;
    
    fn clone_boxed(&self) -> Box<dyn ItemDataProvider>;
}
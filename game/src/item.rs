use std::fmt::Debug;
use std::num::NonZeroU8;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, FromRepr};
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
    pub fn default_stack(self, count: NonZeroU8) -> ItemStack {
        ItemStack {
            ty: self,
            count,
            title: self.default_name().into(),
            desc: self.default_desc().into(),
            data: self.default_data(),
        }
    }

    pub fn default_one(self) -> ItemStack {
        self.default_stack(NonZeroU8::new(1).expect("one is not zero"))
    }

    pub const fn default_name(self) -> &'static str {
        use ItemType as IT;

        match self {
            IT::Grass => "Grass",
            IT::Dirt => "Dirt",
            IT::Cobblestone => "Cobblestone",
            IT::Stone => "Stone",
            IT::Log => "Log",
            IT::LeafPile => "Leaf Pile",
        }
    }

    pub const fn default_desc(self) -> &'static str {
        use ItemType as IT;

        match self {
            IT::Grass => "very grassy",
            IT::Dirt => "dirt",
            IT::Cobblestone => "rocky form of stone",
            IT::Stone => "found underground",
            IT::Log => "the basic building material",
            IT::LeafPile => "gathered from trees",
        }
    }

    pub const fn default_data(self) -> Option<Box<dyn ItemDataProvider>> {
        #[allow(unused_imports, reason = "alias for when ItemTypes define their default data")]
        use ItemType as IT;

        #[allow(clippy::match_single_binding, reason = "meant to show that each ItemType should provide its own default data")]
        match self {
            _ => None,
        }
    }

    pub const fn texture_id(self) -> TextureId {
        use ItemType as IT;
        use crate::texture_ids::*;

        match self {
            IT::Grass => GRASS_SIDE,
            IT::Dirt => DIRT,
            IT::Cobblestone => COBBLE,
            IT::Stone => DEBUG_RED,
            IT::Log => LOG,
            IT::LeafPile => DEBUG_GREEN,
        }
    }
}

// TODO: move name and description into item data provider
// TODO: make this clone
#[derive(Debug)]
pub struct ItemStack {
    pub ty: ItemType,
    pub count: NonZeroU8,
    pub title: String,
    pub desc: String,
    pub data: Option<Box<dyn ItemDataProvider>>,
}

impl ItemStack {
    pub const MAX_STACK: NonZeroU8 = NonZeroU8::MAX;

    pub fn new_one_without_data(ty: ItemType, title: impl Into<String>, desc: impl Into<String>) -> Self {
        Self::new_one(ty, title, desc, None)
    }

    pub fn new_one(ty: ItemType, title: impl Into<String>, desc: impl Into<String>, data: Option<Box<dyn ItemDataProvider>>) -> Self {
        Self {
            ty,
            count: NonZeroU8::new(1).expect("one is not zero"),
            title: title.into(),
            desc: desc.into(),
            data,
        }
    }

    pub fn with_count(self, count: NonZeroU8) -> Self {
        Self {
           count,
            .. self
        }
    }

    pub fn eq_data(&self, rhs: &Self) -> bool {
        let Self { ty: lhs_ty, count: _, title: lhs_title, desc: lhs_desc, data: lhs_data } = self;
        let Self { ty: rhs_ty, count: _, title: rhs_title, desc: rhs_desc, data: rhs_data } = rhs;

        let data_eq = |lhs: &Option<Box<dyn ItemDataProvider>>, rhs: &Option<Box<dyn ItemDataProvider>>| -> bool {
            match (lhs, rhs) {
                (None, None) => true,
                (Some(rhs), Some(lhs)) => rhs.hash() == lhs.hash(),
                _ => false,
            }
        };

        lhs_ty == rhs_ty && lhs_title == rhs_title && lhs_desc == rhs_desc && data_eq(lhs_data, rhs_data)
    }

    pub fn try_combine(&mut self, rhs: Self) -> Option<Self> {
        if !self.eq_data(&rhs) {
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

                Some(rhs.with_count(count))
            }
        }
    }
    
    pub fn split(mut self, first_ct: NonZeroU8) -> (Self, Option<Self>) {
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
        let ct = self.count.get() / 2 + self.count.get() % 2;
        
        self.split(NonZeroU8::new(ct).expect("shouldn't ever be zero"))
    }
}

impl Clone for ItemStack {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty,
            count: self.count,
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
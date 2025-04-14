use std::{any, mem};
use std::any::TypeId;
use crate::block::dyn_block::{BlockDescriptor, ItemStack, TODO};

pub struct Air;

impl BlockDescriptor for Air {
    fn uuid(&self) -> u128 {
        let id = TypeId::of::<Self>();

        unsafe { mem::transmute(id) } // can't convert otherwise + solve the stability problem later
    }

    fn on_break(&self) -> Option<ItemStack> {
        todo!()
    }

    fn placeable(&self) -> bool {
        todo!()
    }

    fn raycast_ty(&self) -> TODO {
        todo!()
    }

    fn texture(&self) -> TODO {
        todo!()
    }
}
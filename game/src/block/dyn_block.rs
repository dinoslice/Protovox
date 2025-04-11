use downcast_rs::{impl_downcast, Downcast};
use crate::block::BlockDyn;

pub struct TODO;

pub trait BlockDescriptor: Downcast {
    fn uuid(&self) -> u128;
    fn on_break(&self) -> Option<ItemStack>;
    fn placeable(&self) -> bool;
    fn raycast_ty(&self) -> TODO; // TODO: solid, fluid, skip?
    fn texture(&self) -> TODO; // TextureDescriptor
    
    fn into_dyn(self) -> BlockDyn
    where Self: Sized {
        BlockDyn::of(self)
    }
}

impl_downcast!(BlockDescriptor);

// TODO: this is from the inventory branch
pub struct ItemStack(u128);
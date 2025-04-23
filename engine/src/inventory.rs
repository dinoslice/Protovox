use std::num::NonZeroUsize;
use shipyard::Component;
use game::inventory::Inventory;
use game::item::{Item, ItemStack};

#[derive(Component, Debug)]
pub struct PlayerInventory(Box<[Option<ItemStack>]>);

impl Inventory for PlayerInventory {
    fn as_slice(&self) -> &[Option<ItemStack>] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [Option<ItemStack>] {
        &mut self.0
    }
}

impl PlayerInventory {
    pub fn new(size: NonZeroUsize) -> Self {
        let mut v = Vec::new();

        v.resize_with(size.get(), || None);

        Self(v.into_boxed_slice())
    }
}
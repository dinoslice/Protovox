use std::num::{NonZeroU8, NonZeroUsize};
use shipyard::Component;
use game::item::{Item, ItemStack};

#[derive(Component, Debug)]
pub struct Inventory(Box<[Option<ItemStack>]>);

impl Inventory {
    pub fn spaces(&self) -> impl Iterator<Item = Option<&'_ ItemStack>> {
        self.0.iter().map(Option::as_ref)
    }

    pub fn items(&self) -> impl Iterator<Item = &ItemStack> {
        self.0.iter()
            .filter_map(Option::as_ref)
    }

    pub fn space(&self) -> usize {
        self.0.len()
    }

    pub fn try_insert(&mut self, item_stack: ItemStack) -> Option<ItemStack> {
        let mut residual = item_stack;

        // loop through every slot that already has items
        for inv_stack in self.0.iter_mut()
            .filter_map(Option::as_mut)
        {
            // try to put data into the stack, return None if all the data has been interested
            match inv_stack.try_combine(residual) {
                Some(res) => residual = res,
                None => return None,
            }
        }


        match self.0.iter_mut().find(|s| s.is_none()) {
            None => return Some(residual),
            Some(empty) => *empty = Some(residual),
        }

        None
    }

    pub fn as_slice(&self) -> &[Option<ItemStack>] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [Option<ItemStack>] {
        &mut self.0
    }

    // TODO: refactor these helper methods
    pub fn split_item_at(&mut self, slot: usize) -> Option<Item> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(it) = slot.take() {
            let (item, rem) = it.split_item();

            *slot = rem;

            Some(item)
        } else {
            None
        }
    }

    pub fn split_at_most_at(&mut self, slot: usize, ct: NonZeroU8) -> Option<ItemStack> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(it) = slot.take() {
            let (it, rem) = it.split_at_most(ct);

            *slot = rem;

            Some(it)
        } else {
            None
        }
    }

    pub fn split_exact_at(&mut self, slot: usize, ct: NonZeroU8) -> Option<ItemStack> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(it) = slot.take() {
            match it.split_exact(ct) {
                Ok((it, rem)) => {
                    *slot = rem;
                    
                    Some(it)
                }
                Err(rem) => {
                    *slot = Some(rem);
                    
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn try_insert_at(&mut self, slot: usize, it: ItemStack) -> Option<ItemStack> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(slot) = slot {
            slot.try_combine(it)
        } else {
            *slot = Some(it);
            
            None
        }
    }
}

impl Inventory {
    pub fn new(size: NonZeroUsize) -> Self {
        let mut v = Vec::new();

        v.resize_with(size.get(), || None);

        Self(v.into_boxed_slice())
    }
}
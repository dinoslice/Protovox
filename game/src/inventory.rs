use std::num::NonZeroU8;
use crate::block::Block;
use crate::block::face_type::FaceType;
use crate::item::{Item, ItemStack};
use crate::location::BlockLocation;

pub trait Inventory {
    fn as_slice(&self) -> &[Option<ItemStack>];

    fn as_mut_slice(&mut self) -> &mut [Option<ItemStack>];

    fn items(&self) -> impl Iterator<Item = &ItemStack> {
        self.as_slice().iter().filter_map(Option::as_ref)
    }

    fn items_mut(&mut self) -> impl Iterator<Item = &mut ItemStack> {
        self.as_mut_slice().iter_mut().filter_map(Option::as_mut)
    }

    fn size(&self) -> usize {
        self.as_slice().len()
    }

    fn try_insert(&mut self, item_stack: ItemStack) -> Option<ItemStack> {
        let mut residual = item_stack;

        // loop through every slot that already has items
        for inv_stack in self.items_mut()
        {
            // try to put data into the stack, return None if all the data has been interested
            match inv_stack.try_combine(residual) {
                Some(res) => residual = res,
                None => return None,
            }
        }


        match self.as_mut_slice()
            .iter_mut()
            .find(|s| s.is_none())
        {
            None => return Some(residual),
            Some(empty) => *empty = Some(residual),
        }

        None
    }

    // TODO: refactor these helper methods
    fn split_item_at(&mut self, slot: usize) -> Option<Item> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(it) = slot.take() {
            let (item, rem) = it.split_item();

            *slot = rem;

            Some(item)
        } else {
            None
        }
    }

    fn split_at_most_at(&mut self, slot: usize, ct: NonZeroU8) -> Option<ItemStack> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(it) = slot.take() {
            let (it, rem) = it.split_at_most(ct);

            *slot = rem;

            Some(it)
        } else {
            None
        }
    }

    fn split_exact_at(&mut self, slot: usize, ct: NonZeroU8) -> Option<ItemStack> {
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

    fn try_insert_at(&mut self, slot: usize, it: ItemStack) -> Option<ItemStack> {
        let slot = self.as_mut_slice().get_mut(slot).expect("TODO: better error, slot out of range");

        if let Some(slot) = slot {
            slot.try_combine(it)
        } else {
            *slot = Some(it);

            None
        }
    }

    fn try_get_place_at(&mut self, slot: usize, location: BlockLocation, face_type: FaceType) -> Option<Block> {
        if let Some(item) = self.split_item_at(slot) {
            match item.place(location, face_type) {
                Ok(block) => Some(block),
                Err(err_item) => {
                    let rem = self.try_insert_at(slot, err_item.stack_one());

                    assert!(rem.is_none(), "should have at least one free slot");

                    None
                }
            }
        } else {
            None
        }
    }
}
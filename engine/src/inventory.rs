use std::num::NonZeroUsize;
use shipyard::Component;
use game::item::ItemStack;

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
}

impl Inventory {
    pub fn new(size: NonZeroUsize) -> Self {
        let mut v = Vec::new();

        v.resize_with(size.get(), || None);

        Self(v.into_boxed_slice())
    }
}
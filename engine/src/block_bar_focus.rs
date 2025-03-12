#[derive(Debug, Clone, shipyard::Unique)]
pub struct BlockBarFocus {
    // TODO: don't make this public
    pub focus: [Option<usize>; 9],
    pub inventory_size: usize,
}

impl BlockBarFocus {
    pub fn new(inventory_size: usize) -> Self {
        Self {
            focus: Default::default(),
            inventory_size,
        }
    }
}
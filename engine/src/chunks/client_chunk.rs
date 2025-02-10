use crate::game::chunk::data::ChunkData;

pub struct ClientChunk {
    pub data: ChunkData,
    pub bake: BakeState,
}

impl ClientChunk {
    pub fn set_dirty(&mut self) {
        if self.bake == BakeState::Baked {
            self.bake = BakeState::NeedsBaking;
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BakeState {
    DontBake, // chunk out of render distance
    NeedsBaking, // chunk in render distance, not baked yet
    Baked, // chunk in render distance and baked
}

/*
    transitions:
        dont bake -> needs baking: enter rend dist
        dont bake -> baked: (can't happen)

        need bake -> don't bake: no longer within render distance
        need bake -> baked: finished baking

        bake -> don't bake: free buffer, no longer in render dist
        bake -> needs baking: modified
*/
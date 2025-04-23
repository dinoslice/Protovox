use shipyard::{Unique, UniqueViewMut, UniqueView, View, IntoIter};
use game::block::Block;
use game::location::BlockLocation;
use crate::chunks::chunk_manager::ChunkManager;
use crate::chunks::raycast::RaycastHit;
use crate::looking_at_block::LookingAtBlock;

#[derive(Unique, Debug, Default)]
pub struct CurrentlyFocusedBlock(pub Option<BlockLocation>);

pub fn focus_interactable_block(

    v_local_player: View<LookingAtBlock>,
    v_looking_at_block: View<LookingAtBlock>,
    world: UniqueView<ChunkManager>,
    mut interactable: UniqueViewMut<CurrentlyFocusedBlock>,
) {
    let (raycast, ..) = (&v_looking_at_block, &v_local_player).iter()
        .next()
        .expect("LocalPlayer should have looking at block");

    let Some(raycast) = &raycast.0 else {
        return;
    };

    let RaycastHit::Block { location, .. } = &raycast.hit else {
        return;
    };

    if matches!(world.get_block_ref(location), Some(Block::Crate { .. })) {
        interactable.0 = Some(location.clone());
    } else {
        interactable.0 = None;
    }
}
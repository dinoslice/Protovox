use shipyard::Component;
use crate::chunks::raycast::BlockRaycastResult;

#[derive(Debug, Component)]
pub struct LookingAtBlock(pub Option<BlockRaycastResult>);
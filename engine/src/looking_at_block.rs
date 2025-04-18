use shipyard::Component;
use crate::chunks::raycast::RaycastResult;

#[derive(Debug, Component)]
pub struct LookingAtBlock(pub Option<RaycastResult>);
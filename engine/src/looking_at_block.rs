use glm::Vec3;
use shipyard::Component;
use game::location::BlockLocation;
use crate::chunks::raycast::RaycastResult;

#[derive(Debug, Component)]
pub struct LookingAtBlock(pub Option<RaycastResult>);

#[derive(shipyard::Unique, Default)]
pub struct RaycastDebug {
    pub checks: Vec<BlockLocation>,
    pub start: Vec3,
    pub end: Vec3,
}
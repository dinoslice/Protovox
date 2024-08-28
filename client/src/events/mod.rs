use game::chunk::{data::ChunkData, location::ChunkLocation};
use shipyard::Component;

#[derive(Debug, Component)]
pub struct ChunkGenRequestEvent(pub ChunkLocation);

pub struct ChunkGenEvent(pub ChunkData);
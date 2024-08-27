use game::chunk::location::ChunkLocation;
use shipyard::Component;

#[derive(Debug, Component)]
pub struct ChunkGenRequestEvent(pub ChunkLocation);
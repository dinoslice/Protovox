use shipyard::Unique;
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::render_distance::RenderDistance;

#[derive(Unique, Debug, Clone)]
pub struct WorldGenVisualizerParams {
    pub generate_center: ChunkLocation,
    pub render_distance: RenderDistance,

    pub cam_offset: WorldLocation,
    pub request_reframe: bool,
}
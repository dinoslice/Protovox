use glm::Vec3;
use game::block::Block;
use game::location::{BlockLocation, WorldLocation};
use crate::chunks::chunk_manager::ChunkManager;

#[derive(Debug, Clone)]
pub struct BlockRaycastResult {
    pub hit_block: BlockLocation,
    pub prev_air: Option<BlockLocation>,
    pub distance: f32,
}

impl ChunkManager {
    pub fn raycast(&self, origin: &Vec3, direction: &Vec3, max_dist: f32, step: f32) -> Option<BlockRaycastResult> {
        let delta = direction.normalize() * step;

        let mut curr = *origin;
        let mut traversed = 0.0;

        let mut prev_air = None;

        while traversed < max_dist {
            let block_loc = WorldLocation(curr).into();

            // TODO: should I early return if the chunk doesn't exist? or should you be able to raycast through it?
            let block = self.get_block(&block_loc)?;

            if block != Block::Air {
                return Some(BlockRaycastResult {
                    hit_block: block_loc,
                    prev_air,
                    distance: traversed,
                })
            } else {
                traversed += step;
                prev_air = Some(block_loc);
                curr += delta;
            }
        }

        None
    }
}
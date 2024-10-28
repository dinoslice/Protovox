use glm::Vec3;
use game::block::Block;
use game::location::WorldLocation;
use crate::chunks::chunk_manager::ChunkManager;

impl ChunkManager {
    // TODO: eventually don't return a floating point type?
    pub fn raycast(&self, origin: &Vec3, direction: &Vec3, max_dist: f32, step: f32) -> Option<WorldLocation> {
        let delta = direction.normalize() * step;

        let mut curr = *origin;
        let mut traversed = 0.0;

        while traversed < max_dist {
            let world_loc = WorldLocation(curr);

            // TODO: should I early return if the chunk doesn't exist? or should you be able to raycast through it?
            let block = self.get_block_ref_from_world_loc(&world_loc)?;

            if *block != Block::Air {
                return Some(world_loc);
            } else {
                traversed += step;
                curr += delta;
            }
        }

        None
    }
}
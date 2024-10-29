use glm::Vec3;
use game::block::Block;
use game::location::WorldLocation;
use crate::chunks::chunk_manager::ChunkManager;

pub enum RaycastResult {
    Hit {
        hit_position: WorldLocation,
        prev_air: Option<WorldLocation>,
        distance: f32,
    },
    None,
}

impl ChunkManager {
    // TODO: eventually don't return a floating point type?
    pub fn raycast(&self, origin: &Vec3, direction: &Vec3, max_dist: f32, step: f32) -> RaycastResult {
        let delta = direction.normalize() * step;

        let mut curr = *origin;
        let mut traversed = 0.0;

        let mut prev_air = None;

        while traversed < max_dist {
            let world_loc = WorldLocation(curr);

            // TODO: should I early return if the chunk doesn't exist? or should you be able to raycast through it?
            // let block = self.get_block_ref_from_world_loc(&world_loc)?;
            let Some(block) = self.get_block_ref_from_world_loc(&world_loc) else {
                return RaycastResult::None;
            };

            if *block != Block::Air {
                return RaycastResult::Hit {
                    hit_position: world_loc,
                    prev_air,
                    distance: traversed,
                }
            } else {
                traversed += step;
                prev_air = Some(world_loc);
                curr += delta;
            }
        }

        RaycastResult::None
    }
}
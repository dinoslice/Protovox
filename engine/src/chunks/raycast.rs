use glm::Vec3;
use game::block::Block;
use game::block::face_type::FaceType;
use game::location::{BlockLocation, WorldLocation};
use crate::chunks::chunk_manager::ChunkManager;

#[derive(Debug, Clone)]
pub struct RaycastResult {
    pub distance: f32,
    pub hit: RaycastHit,
}


#[derive(Debug, Clone)]
pub enum RaycastHit {
    Block {
        location: BlockLocation,
        face: Option<FaceType>,
    },
    Entity {
        // TODO
    }
}

impl ChunkManager {
    pub fn raycast(&self, origin: &Vec3, direction: &Vec3, max_dist: f32, step: f32) -> Option<BlockRaycastResult> {
        let delta = direction.normalize() * step;

        let mut curr = origin;
        let mut traversed = 0.0;

        while traversed < max_dist {
            let block_loc = WorldLocation(curr).into();

            // TODO: should I early return if the chunk doesn't exist? or should you be able to raycast through it?
            let block = self.get_block_ref(&block_loc)?;

            if *block != Block::Air {
                return Some(RaycastResult {
                    distance: traversed,
                    hit: RaycastHit::Block {
                        location: block_loc.clone(),
                        face: (traversed.abs() < 1e-6).then(|| determine_hit_face(curr, delta, block_loc)),
                    },
                });
            } else {
                traversed += step;
                curr += delta;
            }
        }

        None
    }
}

fn determine_hit_face(position: Vec3, delta: Vec3, block_loc: BlockLocation) -> FaceType {
    let (block_min, block_max) = block_loc.get_aabb_bounds();

    // Calculate previous position (before entering the block)
    let mut prev_pos = position;

    for i in 0..10 {
        prev_pos -= delta;
        dbg!(i);

        if prev_pos.x < block_min.x && position.x >= block_min.x {
            return FaceType::Left;
        }
        if prev_pos.x >= block_max.x && position.x < block_max.x {
            return FaceType::Right;
        }
        if prev_pos.y < block_min.y && position.y >= block_min.y {
            return FaceType::Bottom;
        }
        if prev_pos.y >= block_max.y && position.y < block_max.y {
            return FaceType::Top;
        }
        if prev_pos.z < block_min.z && position.z >= block_min.z {
            return FaceType::Back;
        }
        if prev_pos.z >= block_max.z && position.z < block_max.z {
            return FaceType::Front;
        }
    }

    unreachable!("this function should not be called if distance = 0")
}
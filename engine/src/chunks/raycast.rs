use glm::Vec3;
use game::block::Block;
use game::block::face_type::{Axis, FaceType};
use game::location::{BlockLocation, WorldLocation};
use crate::chunks::chunk_manager::ChunkManager;
use crate::looking_at_block::RaycastDebug;

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
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_dist: f32, mut rc_dbg: Option<&mut RaycastDebug>) -> Option<RaycastResult> {
        rc_dbg.as_mut().map(|d| d.checks.clear());

        let direction = direction.normalize();

        let mut voxel = BlockLocation::from(WorldLocation(origin));

        let step = direction.map(|n| n.signum() as i32);

        let voxel_f = voxel.0.cast::<f32>();

        let mut t_max = Vec3::from_fn(|i, _| {
            if direction[i] != 0.0 {
                let next_boundary = if direction[i] >= 0.0 {
                    voxel_f[i] + 1.0
                } else {
                    voxel_f[i]
                };

                (next_boundary - origin[i]) / direction[i]
            } else {
                f32::INFINITY
            }
        });

        let t_delta = direction.map(|n| n.recip().abs());

        let mut t = 0.0;

        let mut face = None;

        while t < max_dist {
            rc_dbg.as_mut().map(|d| d.checks.push(voxel.clone()));

            if *self.get_block_ref(&voxel)? != Block::Air {
                rc_dbg.map(|d| {
                    d.start = origin;
                    d.end = origin + direction * t;
                });

                return Some(RaycastResult {
                    distance: t,
                    hit: RaycastHit::Block {
                        location: voxel,
                        face,
                    },
                })
            }

            let min_comp = if t_max.x < t_max.y && t_max.x < t_max.z {
                0
            } else if t_max.y < t_max.z {
                1
            } else {
                2
            };

            voxel.0[min_comp] += step[min_comp];
            t = t_max[min_comp];
            t_max[min_comp] += t_delta[min_comp];
            face = Some(FaceType::from_axis_and_sign(
                Axis::from_repr(min_comp as _).expect("min_comp returns [0,3)"),
                step[min_comp].is_negative()
            ));
        }

        None
    }
}
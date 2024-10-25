use glm::Vec3;
use shipyard::{EntitiesViewMut, IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use game::block::Block;
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{Entity, Hitbox, Transform, Velocity};
use crate::rendering::gizmos::{BoxGizmo, GizmoLifetime, GizmoStyle};

pub fn move_with_collision(
    vm_hitbox: View<Hitbox>,
    mut vm_transform: ViewMut<Transform>,
    vm_entity: View<Entity>,
    mut vm_velocity: ViewMut<Velocity>,
    mut world: UniqueViewMut<ChunkManager>,

    mut entities: EntitiesViewMut,
    mut vm_box_gizmos: ViewMut<BoxGizmo>,

    delta_time: UniqueView<LastDeltaTime>,
) {
    for (hitbox, transform, vel, _) in (&vm_hitbox, &mut vm_transform, &mut vm_velocity, &vm_entity).iter() {
        let half_hitbox = hitbox.0 * 0.5;

        let min_extent = transform.position - half_hitbox;
        let max_extent = transform.position + half_hitbox;

        entities.add_entity(&mut vm_box_gizmos, BoxGizmo::from_corners(
            min_extent,
            max_extent,
            GizmoStyle::stroke(rgb::Rgb { r: 0.0, g: 0.0, b: 1.0 }),
            GizmoLifetime::SingleFrame,
        ));

        // Helper function to check if the given position collides with a block in the world
        let mut check_collision = |pos: Vec3| -> bool {
            let min_extent = pos - half_hitbox;
            let max_extent = pos + half_hitbox;

            let min_floor = min_extent.map(|n| n.floor() as i32);
            let max_floor = max_extent.map(|n| n.floor() as i32);

            for x in min_floor.x..=max_floor.x {
                for y in min_floor.y..=max_floor.y {
                    for z in min_floor.z..=max_floor.z {
                        let world_loc = WorldLocation(Vec3::new(x as f32, y as f32, z as f32));
                        let chunk_loc = ChunkLocation::from(&world_loc);

                        if let Some(chunk) = world.get_chunk_ref_from_location_mut(&chunk_loc) {
                            let chunk_pos = ChunkPos::from(&world_loc);

                            if chunk.data.get_block(chunk_pos) != Block::Air {
                                return true; // Collision detected
                            }
                        }
                    }
                }
            }
            false // No collision
        };

        let frame_vel = vel.0 * delta_time.0.as_secs_f32();

        // 1. Handle X-axis movement
        let new_position_x = Vec3::new(transform.position.x + frame_vel.x, transform.position.y, transform.position.z);
        if !check_collision(new_position_x) {
            transform.position.x += frame_vel.x;
        } else {
            vel.0.x = 0.0; // Stop movement in the X axis due to collision
        }

        // 2. Handle Y-axis movement (gravity, jumping, falling)
        let new_position_y = Vec3::new(transform.position.x, transform.position.y + frame_vel.y, transform.position.z);
        if !check_collision(new_position_y) {
            transform.position.y += frame_vel.y;
        } else {
            vel.0.y = 0.0; // Stop movement in the Y axis due to collision
        }

        // 3. Handle Z-axis movement
        let new_position_z = Vec3::new(transform.position.x, transform.position.y, transform.position.z + frame_vel.z);
        if !check_collision(new_position_z) {
            transform.position.z += frame_vel.z;
        } else {
            vel.0.z = 0.0; // Stop movement in the Z axis due to collision
        }

        // After processing all axes, update the transform position

        let min_extent = transform.position - half_hitbox;
        let max_extent = transform.position + half_hitbox;

        entities.add_entity(&mut vm_box_gizmos, BoxGizmo::from_corners(
            min_extent,
            max_extent,
            GizmoStyle::stroke(rgb::Rgb { r: 1.0, g: 0.0, b: 0.0 }),
            GizmoLifetime::SingleFrame,
        ));
    }
}

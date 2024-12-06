use shipyard::{IntoIter, IntoWorkload, UniqueView, View, ViewMut, Workload};
use crate::application::delta_time::LastDeltaTime;
use crate::components::{Entity, GravityAffected, Hitbox, IsOnGround, Transform, Velocity};

pub mod movement;
mod collision_response;
pub mod collision;

pub fn process_physics() -> Workload {
    (
        collision_response::move_with_collision,
        move_non_colliding_entities,
        apply_gravity,
    ).into_sequential_workload()
}

fn move_non_colliding_entities(
    mut vm_transform: ViewMut<Transform>,
    vm_entity: View<Entity>,
    v_velocity: View<Velocity>,
    v_hitbox: View<Hitbox>,
    
    delta_time: UniqueView<LastDeltaTime>,
) {
    for (transform, velocity, ..) in (&mut vm_transform, &v_velocity, &vm_entity, !&v_hitbox).iter() {
        transform.position += velocity.0 * delta_time.0.as_secs_f32();
    }
}

fn apply_gravity(
    mut vm_velocity: ViewMut<Velocity>,
    v_transform: View<Transform>,
    v_entity: View<Entity>,
    v_gravity_affected: View<GravityAffected>,
    v_is_on_ground: View<IsOnGround>,
    delta_time: UniqueView<LastDeltaTime>
) {
    let dt_secs = delta_time.0.as_secs_f32();

    // TODO: add version of this system for entities without is_on_ground
    for (velocity, _, _, is_on_ground, _) in (&mut vm_velocity, &v_transform, &v_entity, &v_is_on_ground, &v_gravity_affected).iter() {
        // TODO: due to the collision response this doesn't always work
        if !is_on_ground.0 {
            velocity.0.y -= 9.8 * dt_secs;
        }
    }
}
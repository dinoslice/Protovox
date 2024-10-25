use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::application::delta_time::LastDeltaTime;
use crate::components::{Entity, GravityAffected, Transform, Velocity};

pub mod movement;
pub mod collision_response;

pub fn process_physics(mut vm_velocity: ViewMut<Velocity>, mut vm_transform: ViewMut<Transform>, delta_time: UniqueView<LastDeltaTime>) {
    for (velocity, transform) in (&mut vm_velocity, &mut vm_transform).iter() {
        // TODO
    }
}

pub fn apply_gravity(mut vm_velocity: ViewMut<Velocity>, v_transform: View<Transform>, v_entity: View<Entity>, v_gravity_affected: View<GravityAffected>, delta_time: UniqueView<LastDeltaTime>) {
    let dt_secs = delta_time.0.as_secs_f32();

    for (velocity, _, _, _) in (&mut vm_velocity, &v_transform, &v_entity, &v_gravity_affected).iter() {
        velocity.0.y -= 9.8 * dt_secs;
    }
}
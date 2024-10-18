use shipyard::{IntoIter, ViewMut};
use crate::components::{Transform, Velocity};

pub mod movement;

pub fn process_physics(mut vm_velocity: ViewMut<Velocity>, mut vm_transform: ViewMut<Transform>) {
    for (velocity, transform) in (&mut vm_velocity, &mut vm_transform).iter() {
        transform.position += velocity.0;
        *velocity = Velocity::default();
    }
}

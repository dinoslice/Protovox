use std::cmp::Ordering;
use glm::Vec3;
use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::application::delta_time::LastDeltaTime;
use crate::components::{LocalPlayer, PlayerSpeed, Transform, Velocity};
use crate::input::action_map::Action;
use crate::input::InputManager;

pub fn process_movement(input: UniqueView<InputManager>, delta_time: UniqueView<LastDeltaTime>, v_local_player: View<LocalPlayer>, mut vm_transform: ViewMut<Transform>, mut vm_velocity: ViewMut<Velocity>, v_player_speed: View<PlayerSpeed>) {
    let dt_secs = delta_time.0.as_secs_f32();

    let (_, transform, mut velocity, player_speed) = (&v_local_player, &mut vm_transform, &mut vm_velocity, &v_player_speed)
        .iter()
        .next()
        .expect("TODO: local player didn't have transform, velocity, player speed");

    let movement = Vec3::new(
        input.action_map.get_axis(Action::MoveRight, Action::MoveLeft) as f32,
        input.action_map.get_axis(Action::Jump, Action::Sneak) as f32,
        input.action_map.get_axis(Action::MoveForward, Action::MoveBackward) as f32,
    );

    let (yaw_sin, yaw_cos) = transform.yaw.sin_cos();
    let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

    let movement_scaled = movement * player_speed.0 * dt_secs;

    velocity.0 += (forward * movement_scaled.z) + (right * movement_scaled.x) + Vec3::y_axis().into_inner() * movement_scaled.y;
}

pub fn apply_camera_input(input: UniqueView<InputManager>, delta_time: UniqueView<LastDeltaTime>, v_local_player: View<LocalPlayer>, mut vm_transform: ViewMut<Transform>) {
    let dt_secs = delta_time.0.as_secs_f32();

    let (_, transform) = (&v_local_player, &mut vm_transform)
        .iter()
        .next()
        .expect("TODO: local player didn't have transform to modify");

    let rotate_scaled = input.mouse_manager.rotate * input.mouse_manager.sensitivity * dt_secs;

    transform.yaw += rotate_scaled.x;
    transform.pitch -= rotate_scaled.y;

    const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;
    transform.pitch = transform.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
}

pub fn adjust_fly_speed(input: UniqueView<InputManager>, v_local_player: View<LocalPlayer>, mut vm_player_speed: ViewMut<PlayerSpeed>) {
    let (_, player_speed) = (&v_local_player, &mut vm_player_speed)
        .iter()
        .next()
        .expect("TODO: local player didn't have player speed to modify");

    const SCROLL_SCALE: f32 = 0.32;
    const SCROLL_THRESHOLD: f32 = 0.2;

    player_speed.0 = match input.mouse_manager.scroll.partial_cmp(&0.0).unwrap_or(Ordering::Equal) {
        Ordering::Less => match player_speed.0 >= SCROLL_THRESHOLD {
            true => player_speed.0 * (1.0 + SCROLL_SCALE),
            false => SCROLL_THRESHOLD,
        },
        Ordering::Greater => match player_speed.0 >= SCROLL_THRESHOLD {
            true => player_speed.0 * (1.0 - SCROLL_SCALE),
            false => 0.0,
        }
        _ => player_speed.0,
    }.clamp(0.0, 125.0);
}
use std::cmp::Ordering;
use std::time::Duration;
use glm::Vec3;
use crate::components::{PlayerSpeed, Transform};
use crate::input::action_map::Action;
use crate::input::InputManager;

pub fn process_movement(transform: &mut Transform, player_speed: &mut PlayerSpeed, input: &InputManager, delta_time: Duration) {
    let dt_secs = delta_time.as_secs_f32();
    let player_speed = &mut player_speed.0;

    let movement = Vec3::new(
        input.action_map.get_axis(Action::MoveRight, Action::MoveLeft) as f32,
        input.action_map.get_axis(Action::Jump, Action::Sneak) as f32,
        input.action_map.get_axis(Action::MoveForward, Action::MoveBackward) as f32,
    );

    let movement_scaled = movement * (*player_speed) * dt_secs;
    let rotate_scaled = input.mouse_manager.rotate * input.mouse_manager.sensitivity * dt_secs;

    let (yaw_sin, yaw_cos) = transform.yaw.sin_cos();
    let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();

    transform.position += right * movement_scaled.x;
    transform.position.y += movement_scaled.y;
    transform.position += forward * movement_scaled.z;


    // Rotate
    transform.yaw += rotate_scaled.x;
    transform.pitch -= rotate_scaled.y;

    const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;
    transform.pitch = transform.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);


    const SCROLL_SCALE: f32 = 0.32;
    const SCROLL_THRESHOLD: f32 = 0.2;

    *player_speed = match input.mouse_manager.scroll.partial_cmp(&0.0).unwrap_or(Ordering::Equal) {
        Ordering::Less => match *player_speed >= SCROLL_THRESHOLD {
            true => *player_speed * (1.0 + SCROLL_SCALE),
            false => SCROLL_THRESHOLD,
        },
        Ordering::Greater => match *player_speed >= SCROLL_THRESHOLD {
            true => *player_speed * (1.0 - SCROLL_SCALE),
            false => 0.0,
        }
        _ => *player_speed,
    }.clamp(0.0, 125.0);
}
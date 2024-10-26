use std::cmp::Ordering;
use glm::{RealNumber, Vec2, Vec3};
use shipyard::{IntoIter, UniqueView, View, ViewMut};
use na::SVector;
use crate::application::delta_time::LastDeltaTime;
use crate::components::{IsOnGround, LocalPlayer, PlayerSpeed, Transform, Velocity};
use crate::input::action_map::Action;
use crate::input::InputManager;

pub fn process_movement(input: UniqueView<InputManager>, delta_time: UniqueView<LastDeltaTime>, v_local_player: View<LocalPlayer>, mut vm_transform: ViewMut<Transform>, mut vm_velocity: ViewMut<Velocity>, v_player_speed: View<PlayerSpeed>, v_is_on_ground: View<IsOnGround>) {
    let dt_secs = delta_time.0.as_secs_f32();

    let (_, transform, velocity, player_speed, is_on_ground) = (&v_local_player, &mut vm_transform, &mut vm_velocity, &v_player_speed, &v_is_on_ground)
        .iter()
        .next()
        .expect("TODO: local player didn't have transform, velocity, player speed");

    let input_vec = Vec2::new(
        input.action_map.get_axis(Action::MoveForward, Action::MoveBackward) as f32,
        input.action_map.get_axis(Action::MoveRight, Action::MoveLeft) as f32,
    )
        .try_normalize(f32::EPSILON)
        .unwrap_or_default();

    let plane_dir = glm::rotate_vec2(&input_vec, transform.yaw);

    let xz = if input_vec != Vec3::zeros() {
        move_towards(&velocity.0.xz(), &(plane_dir * player_speed.max_vel), player_speed.accel)
    } else {
        move_towards(&velocity.0.xz(), &Vec2::zeros(), player_speed.friction * dt_secs)
    };

    let jump = if input.action_map.get_action(Action::Jump) && is_on_ground.0 {
        player_speed.jump_vel
    } else {
        0.0
    };

    velocity.0 = Vec3::new(xz.x, velocity.0.y + jump, xz.y);
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

    player_speed.max_vel = match input.mouse_manager.scroll.partial_cmp(&0.0).unwrap_or(Ordering::Equal) {
        Ordering::Less => match player_speed.max_vel >= SCROLL_THRESHOLD {
            true => player_speed.max_vel * (1.0 + SCROLL_SCALE),
            false => SCROLL_THRESHOLD,
        },
        Ordering::Greater => match player_speed.max_vel >= SCROLL_THRESHOLD {
            true => player_speed.max_vel * (1.0 - SCROLL_SCALE),
            false => 0.0,
        }
        _ => player_speed.max_vel,
    }.clamp(0.0, 125.0);
}

pub fn move_towards<T: RealNumber, const N: usize> (current: &SVector<T, N>, target: &SVector<T, N>, max_dist: T) -> SVector<T, N> {
    let dist = target - current;
    let mag = dist.norm();

    if mag <= max_dist || mag.is_zero() {
        *target
    } else {
        current + dist.normalize() * max_dist
    }
}
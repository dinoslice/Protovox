use std::cmp::Ordering;
use glm::{RealNumber, Vec2, Vec3};
use shipyard::{IntoIter, UniqueView, View, ViewMut};
use na::SVector;
use crate::application::delta_time::LastDeltaTime;
use crate::components::{IsOnGround, LocalPlayer, PlayerSpeed, SpectatorSpeed, Transform, Velocity};
use crate::gamemode::Gamemode;
use crate::input::action_map::Action;
use crate::input::InputManager;

pub fn process_movement(
    input: UniqueView<InputManager>,
    delta_time: UniqueView<LastDeltaTime>,
    v_local_player: View<LocalPlayer>,
    v_transform: View<Transform>,
    mut vm_velocity: ViewMut<Velocity>,
    v_player_speed: View<PlayerSpeed>,
    v_spectator_speed: View<SpectatorSpeed>,
    v_is_on_ground: View<IsOnGround>,
    v_gamemode: View<Gamemode>,
) {
    let dt_secs = delta_time.0.as_secs_f32();

    let (_, transform, velocity, player_speed, spectator_speed, is_on_ground, gamemode) = (&v_local_player, &v_transform, &mut vm_velocity, &v_player_speed, &v_spectator_speed, &v_is_on_ground, &v_gamemode)
        .iter()
        .next()
        .expect("TODO: local player didn't have transform, velocity, player speed");

    match gamemode {
        Gamemode::Survival => {
            let input_vec = Vec2::new(
                input.pressed().get_axis(Action::MoveForward, Action::MoveBackward) as f32,
                input.pressed().get_axis(Action::MoveRight, Action::MoveLeft) as f32,
            );

            let xz = match input_vec.try_normalize(f32::EPSILON) {
                Some(norm_input) => {
                    let plane_dir = glm::rotate_vec2(&norm_input, transform.yaw());

                    move_towards(&velocity.0.xz(), &(plane_dir * player_speed.max_vel), player_speed.accel * dt_secs)
                }
                None => move_towards(&velocity.0.xz(), &Vec2::zeros(), player_speed.friction * dt_secs),
            };

            let jump = if input.pressed().get_action(Action::Jump) && is_on_ground.0 {
                player_speed.jump_vel
            } else {
                0.0
            };

            velocity.0 = Vec3::new(xz.x, velocity.0.y + jump, xz.y);
        }
        Gamemode::Spectator => {
            let input_vec = Vec3::new(
                input.pressed().get_axis(Action::MoveForward, Action::MoveBackward) as f32,
                input.pressed().get_axis(Action::Jump, Action::Sneak) as f32,
                input.pressed().get_axis(Action::MoveRight, Action::MoveLeft) as f32,
            );
            
            let xyz = match input_vec.try_normalize(f32::EPSILON) {
                Some(norm_input) => {
                    let plane_dir = glm::rotate_vec2(&norm_input.xz(), transform.yaw());

                    let target = Vec3::new(plane_dir.x, norm_input.y, plane_dir.y);

                    move_towards(&velocity.0, &(target * spectator_speed.curr_speed), spectator_speed.accel() * dt_secs)
                }
                None => move_towards(&velocity.0, &Vec3::zeros(), velocity.0.magnitude() / spectator_speed.friction_time * dt_secs),
            };

            velocity.0 = xyz;
        }
    }
}

pub fn apply_camera_input(input: UniqueView<InputManager>, delta_time: UniqueView<LastDeltaTime>, v_local_player: View<LocalPlayer>, mut vm_transform: ViewMut<Transform>) {
    let dt_secs = delta_time.0.as_secs_f32();

    let (_, transform) = (&v_local_player, &mut vm_transform)
        .iter()
        .next()
        .expect("TODO: local player didn't have transform to modify");

    let rotate_scaled = input.mouse_manager.rotate * input.mouse_manager.sensitivity * dt_secs;

    *transform.yaw_mut() += rotate_scaled.x;
    *transform.pitch_mut() -= rotate_scaled.y;

    const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;
    *transform.pitch_mut() = transform.pitch().clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
}

pub fn adjust_spectator_fly_speed(input: UniqueView<InputManager>, v_local_player: View<LocalPlayer>, mut vm_spectator_speed: ViewMut<SpectatorSpeed>) {
    let (_, spectator_speed) = (&v_local_player, &mut vm_spectator_speed)
        .iter()
        .next()
        .expect("TODO: local player didn't have player speed to modify");

    const SCROLL_SCALE: f32 = 0.25;
    const SCROLL_THRESHOLD: f32 = 0.2;

    spectator_speed.curr_speed = match input.mouse_manager.scroll.partial_cmp(&0.0).unwrap_or(Ordering::Equal) {
        Ordering::Less => match spectator_speed.curr_speed >= SCROLL_THRESHOLD {
            true => spectator_speed.curr_speed * (1.0 + SCROLL_SCALE),
            false => SCROLL_THRESHOLD,
        },
        Ordering::Greater => match spectator_speed.curr_speed >= SCROLL_THRESHOLD {
            true => spectator_speed.curr_speed * (1.0 - SCROLL_SCALE),
            false => 0.0,
        }
        Ordering::Equal => spectator_speed.curr_speed,
    }
        .clamp(0.0, spectator_speed.maximum_speed);
    
    spectator_speed.curr_speed = spectator_speed.curr_speed.clamp(0.0, spectator_speed.maximum_speed);
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
use glm::Vec3;
use serde::{Deserialize, Serialize};
use shipyard::Component;
use game::block::BlockTy;
use game::location::WorldLocation;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct LocalPlayer;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct Player;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct Entity;

#[derive(Copy, Clone, Hash, Component, Debug, Default)]
pub struct GravityAffected;

#[derive(Clone, Component, Debug, Default, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Transform {
    pub fn get_loc<T: From<WorldLocation>>(&self) -> T {
        WorldLocation(self.position).into()
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct Velocity(pub Vec3);

#[derive(Clone, Component, Debug)]
pub struct PlayerSpeed {
    pub max_vel: f32,
    pub jump_vel: f32,
    pub accel: f32,
    pub friction: f32,
}

impl PlayerSpeed {
    pub fn from_observed(max_vel: f32, jump_height: f32, gravity: f32, accel_time: f32, friction_time: f32) -> Self {
        assert!(max_vel >= 0.0, "max_vel must be non negative");
        assert!(gravity >= 0.0, "gravity must be non negative");
        assert!(jump_height >= 0.0, "jump_height must be non-negative");
        assert!(accel_time >= 0.0, "accel_time must be non-negative");
        assert!(friction_time >= 0.0, "friction_time must be non-negative");

        let jump_vel = (2.0 * gravity * jump_height).sqrt();
        let accel = max_vel / accel_time;
        let friction = max_vel / friction_time;

        Self { max_vel, jump_vel, accel, friction }
    }
}

impl Default for PlayerSpeed {
    fn default() -> Self {
        Self::from_observed(
            4.32,
            1.25,
            9.8,
            0.2,
            0.18
        )
    }
}

#[derive(Clone, Component, Debug)]
pub struct SpectatorSpeed {
    pub curr_speed: f32,
    pub maximum_speed: f32,

    pub accel_time: f32,
    pub friction_time: f32,
}

impl Default for SpectatorSpeed {
    fn default() -> Self {
        Self {
            curr_speed: 5.5,
            maximum_speed: 384.0,
            accel_time: 0.2,
            friction_time: 0.1,
        }
    }
}

impl SpectatorSpeed {
    pub fn accel(&self) -> f32 {
        self.curr_speed / self.accel_time
    }
    
    pub fn friction(&self) -> f32 {
        self.curr_speed / self.friction_time
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct Hitbox(pub Vec3);

impl Hitbox {
    pub fn default_player() -> Self {
        Self(Vec3::new(0.6, 1.8, 0.6))
    }
}

#[derive(Copy, Clone, Hash, Component, Debug, Default, Eq, PartialEq)]
pub struct IsOnGround(pub bool);

#[derive(Clone, Component, Debug, Default, Eq, PartialEq)]
pub struct HeldBlock(pub usize); // inventory index, TODO: improve api

#[derive(Copy, Clone, Component, Debug, Default, PartialEq)]
pub struct Health {
    pub curr: f32,
    pub max: f32,
}

impl Health {
    pub fn percentage(&self) -> f32 {
        self.curr / self.max
    }

    pub fn percentage_clamped(&self) -> f32 {
        self.percentage().clamp(0.0, 1.0)
    }
}

#[derive(Copy, Clone, Component, Debug, Default, PartialEq)]
pub struct Mana {
    pub curr: f32,
    pub max: f32,
}

impl Mana {
    pub fn percentage(&self) -> f32 {
        self.curr / self.max
    }

    pub fn percentage_clamped(&self) -> f32 {
        self.percentage().clamp(0.0, 1.0)
    }
}
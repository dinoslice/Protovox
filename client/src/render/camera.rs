use std::time::Duration;
use glm::{Mat4, Vec3};
use na::{Perspective3, UnitQuaternion};
use crate::input::action_map::Action;
use crate::input::InputManager;

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub perspective: Perspective3<f32>,
}

impl Camera {
    pub fn view_matrix(&self) -> Mat4 {
        let direction = UnitQuaternion::from_euler_angles(-self.pitch, self.yaw, 0.0) * Vec3::z_axis();

        let target = self.position + direction.zyx();

        glm::look_at_rh(&self.position, &target, &Vec3::y_axis())
    }

    pub fn update_with_input(&mut self, input: &mut InputManager, delta_time: &Duration) {
        let dt_secs = delta_time.as_secs_f32();

        let movement = Vec3::new(
            input.action_map.get_axis(Action::MoveRight, Action::MoveLeft) as f32,
            input.action_map.get_axis(Action::Jump, Action::Sneak) as f32,
            input.action_map.get_axis(Action::MoveForward, Action::MoveBackward) as f32,
        );

        let movement_scaled = movement * self.speed * dt_secs;
        let rotate_scaled = input.mouse_manager.rotate * input.mouse_manager.sensitivity * dt_secs;

        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward * movement_scaled.z;
        self.position += right * movement_scaled.x;

        // move along view vector, "zooms" into object
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let scrollward = Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position += scrollward * input.mouse_manager.scroll * self.speed * input.mouse_manager.sensitivity * dt_secs;

        self.position.y += movement_scaled.y;

        // Rotate
        self.yaw += rotate_scaled.x;
        self.pitch -= rotate_scaled.y;

        const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

        self.pitch = self.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);

        input.mouse_manager.reset_scroll_rotate();
    }

    pub fn as_uniform(&self) -> [[f32; 4]; 4] {
        const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        );

        (OPENGL_TO_WGPU_MATRIX * self.perspective.as_matrix() * self.view_matrix()).into()
    }
}
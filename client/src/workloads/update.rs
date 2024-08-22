use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload};
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::input::InputManager;

pub fn update() -> Workload {
    (
        update_camera_movement,
        reset_mouse_manager_state,
    ).into_sequential_workload()
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, mut camera: UniqueViewMut<Camera>, input_manager: UniqueView<InputManager>) {
    camera.update_with_input(&input_manager, delta_time.0);
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}
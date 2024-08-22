use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload};
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::input::InputManager;

pub fn update() -> Workload {
    (
        update_camera_movement
    ).into_workload()
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, mut camera: UniqueViewMut<Camera>, mut input_manager: UniqueViewMut<InputManager>) {
    camera.update_with_input(&mut input_manager, delta_time.0);
}
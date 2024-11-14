use shipyard::{AllStoragesView, IntoWorkload, UniqueViewMut, Workload};
use crate::application::CaptureState;
use crate::application::delta_time::LastDeltaTime;
use crate::input::InputManager;
use crate::input::mouse_manager::MouseManager;

pub fn startup_core() -> Workload {
    (
        initialize_application_systems
    ).into_workload()
}

fn initialize_application_systems(storages: AllStoragesView) {
    storages.add_unique(InputManager::with_mouse_manager(MouseManager::new(0.75, 50.0)));
    storages.add_unique(CaptureState::default());
    storages.add_unique(LastDeltaTime::default());
}

pub fn update_core() -> Workload {
    (
        update_input_manager,
    ).into_workload()
}

fn update_input_manager(mut input: UniqueViewMut<InputManager>) {
    input.process();
}
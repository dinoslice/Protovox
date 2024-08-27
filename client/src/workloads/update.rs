use shipyard::{AllStoragesViewMut, IntoWorkload, SystemModificator, UniqueView, UniqueViewMut, Workload};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::chunk_manager::ChunkManager;
use crate::input::InputManager;

pub fn update() -> Workload {
    (
        update_camera_movement,
        reset_mouse_manager_state,
        update_chunk_manager.after_all(update_camera_movement),
    ).into_sequential_workload()
}

fn update_chunk_manager(delta_time: UniqueView<LastDeltaTime>, mut chunk_mgr: UniqueViewMut<ChunkManager>, camera: UniqueView<Camera>, mut all_storages: AllStoragesViewMut) {
    let current_chunk = ChunkLocation::from(WorldLocation(camera.position));

    let reqs = chunk_mgr.update(current_chunk, delta_time.0, Vec::default());

    if reqs.len() > 0 {
        tracing::debug!("requesting {}", reqs.len());
        tracing::debug!("{reqs:?}")
    }

    all_storages.bulk_add_entity(reqs);
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, mut camera: UniqueViewMut<Camera>, input_manager: UniqueView<InputManager>) {
    camera.update_with_input(&input_manager, delta_time.0);
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}
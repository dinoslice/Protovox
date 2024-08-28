use crate::chunks::chunk_manager::ChunkManager;
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::input::InputManager;
use crate::rendering::graphics_context::GraphicsContext;

pub fn update() -> Workload {
    (
        update_camera_movement,
        reset_mouse_manager_state,
        update_chunk_manager.after_all(update_camera_movement),
    ).into_sequential_workload()
}

fn update_chunk_manager(delta_time: UniqueView<LastDeltaTime>, mut chunk_mgr: UniqueViewMut<ChunkManager>, camera: UniqueView<Camera>, g_ctx: UniqueView<GraphicsContext>) {
    let current_chunk = ChunkLocation::from(WorldLocation(camera.position));

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, Vec::default(), None, &g_ctx);

    if reqs.len() > 0 {
        tracing::debug!("requesting {}", reqs.len());
        tracing::debug!("{reqs:?}")
    }


    for req in reqs {
        chunk_mgr.request_chunk(&req);
    }
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, mut camera: UniqueViewMut<Camera>, input_manager: UniqueView<InputManager>) {
    camera.update_with_input(&input_manager, delta_time.0);
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}
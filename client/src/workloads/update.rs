use crate::chunks::chunk_manager::ChunkManager;
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator, AllStoragesViewMut};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::input::InputManager;
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;

pub fn update() -> Workload {
    (
        update_camera_movement,
        reset_mouse_manager_state,
        update_chunk_manager.after_all(update_camera_movement),
    ).into_sequential_workload()
}


// TODO: fix borrowing of storages
fn update_chunk_manager(/*delta_time: UniqueView<LastDeltaTime>, mut chunk_mgr: UniqueViewMut<ChunkManager>, camera: UniqueView<Camera>, g_ctx: UniqueView<GraphicsContext>,*/ mut all_storages: AllStoragesViewMut) {
    let delta_time = all_storages.borrow::<UniqueView<LastDeltaTime>>().unwrap();
    let mut chunk_mgr = all_storages.borrow::<UniqueViewMut<ChunkManager>>().unwrap();
    let camera = all_storages.borrow::<UniqueView<Camera>>().unwrap();
    let g_ctx = all_storages.borrow::<UniqueView<GraphicsContext>>().unwrap();

    let world_gen = all_storages.borrow::<UniqueViewMut<WorldGenerator>>().unwrap();

    let recv = world_gen.receive_chunks();

    if !recv.is_empty() {
        tracing::debug!("Received {} chunks.", recv.len());
    }

    let current_chunk = ChunkLocation::from(WorldLocation(camera.position));

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, recv, None, &g_ctx);

    if reqs.len() > 0 {
        tracing::debug!("requesting {}", reqs.len());
        tracing::debug!("{reqs:?}")
    }

    for req in reqs {
        world_gen.spawn_generate_task(req.0);
    }

    // TODO: add to ecs so we can support multiple sources

    // drop(delta_time);
    // drop(chunk_mgr);
    // drop(camera);
    // drop(g_ctx);

    // all_storages.bulk_add_entity(reqs.into_iter());

    // for req in reqs {
    //     all_storages.add_entity(req);
    // }
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, mut camera: UniqueViewMut<Camera>, input_manager: UniqueView<InputManager>) {
    camera.update_with_input(&input_manager, delta_time.0);
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}
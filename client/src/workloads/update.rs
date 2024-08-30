use glm::all;
use crate::chunks::chunk_manager::ChunkManager;
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator, AllStoragesViewMut, ViewMut};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent};
use crate::input::InputManager;
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;

pub fn update() -> Workload {
    (
        update_camera_movement,
        reset_mouse_manager_state,
        get_generated_chunks,
        chunk_manager_update_and_request.after_all(update_camera_movement),
        request_chunks,
    ).into_sequential_workload()
}

fn get_generated_chunks(mut all_storages: AllStoragesViewMut) {
    let world_gen = all_storages
        .borrow::<UniqueView<WorldGenerator>>()
        .expect("Failed to borrow world generator");

    let chunks = world_gen.receive_chunks();

    drop(world_gen);

    if !chunks.is_empty() {
        all_storages.bulk_add_entity(chunks.into_iter());
    }
}

fn request_chunks(mut reqs: ViewMut<ChunkGenRequestEvent>, world_generator: UniqueView<WorldGenerator>) {
    for req in reqs.drain() {
        world_generator.spawn_generate_task(req.0);
    }
}

// TODO: fix borrowing of storages
fn chunk_manager_update_and_request(mut all_storages: AllStoragesViewMut) {
    if let Some(reqs) = all_storages.run(chunk_manager_update) {
        all_storages.bulk_add_entity(reqs.into_iter());
    }
}

fn chunk_manager_update(delta_time: UniqueView<LastDeltaTime>, mut chunk_mgr: UniqueViewMut<ChunkManager>, camera: UniqueView<Camera>, g_ctx: UniqueView<GraphicsContext>, mut chunk_gen_event: ViewMut<ChunkGenEvent>) -> Option<Vec<ChunkGenRequestEvent>> {
    let current_chunk = ChunkLocation::from(WorldLocation(camera.position));

    let recv = chunk_gen_event.drain().collect();

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, recv, None, &g_ctx);

    (!reqs.is_empty()).then_some(reqs)
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, mut camera: UniqueViewMut<Camera>, input_manager: UniqueView<InputManager>) {
    camera.update_with_input(&input_manager, delta_time.0);
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}
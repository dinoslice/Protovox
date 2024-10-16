use glm::all;
use laminar::Packet;
use crate::chunks::chunk_manager::{ChunkManager, chunk_index_in_render_distance};
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator, AllStoragesViewMut, ViewMut, IntoIter, View, IntoWithId, EntitiesViewMut, WorkloadModificator};
use tracing::debug;
use game::block::Block;
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use game::location::WorldLocation;
use packet::Packet as _;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::components::{Entity, Hitbox, LocalPlayer, PlayerSpeed, Transform};
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent};
use crate::input::InputManager;
use crate::multiplayer::server_connection::ServerConnection;
use crate::{movement, networking};
use crate::networking::server_socket::{process_network_events_system, ServerHandler};
use crate::render_distance::RenderDistance;
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;

pub fn update() -> Workload {
    (
        update_camera_movement,
        reset_mouse_manager_state,
        networking::update_networking,
        get_generated_chunks.run_if(is_hosted),
        broadcast_chunks.run_if(is_hosted),
        chunk_manager_update_and_request.after_all(update_camera_movement),
        generate_chunks.run_if(is_hosted),
        client_request_chunks_from_server.run_if(is_multiplayer_client),
        check_collision,
    ).into_sequential_workload()
}

fn get_generated_chunks(world_gen: UniqueView<WorldGenerator>, mut vm_entities: EntitiesViewMut, vm_chunk_gen_evt: ViewMut<ChunkGenEvent>) {
    let chunks = world_gen.receive_chunks();

    drop(world_gen);

    if !chunks.is_empty() {
        vm_entities.bulk_add_entity(vm_chunk_gen_evt, chunks.into_iter());
    }
}

fn generate_chunks(mut reqs: ViewMut<ChunkGenRequestEvent>, world_generator: UniqueView<WorldGenerator>) {
    for req in reqs.drain() {
        world_generator.spawn_generate_task(req.0);
    }
}

fn client_request_chunks_from_server(mut reqs: ViewMut<ChunkGenRequestEvent>, server_connection: UniqueView<ServerConnection>) {
    let sender = &server_connection.tx;
    let addr = server_connection.server_addr;

    for req in reqs.drain() {
        let p = Packet::reliable_unordered(
            addr,
            req
                .serialize_packet()
                .unwrap()
        );

        if sender.try_send(p).is_err() {
            tracing::debug!("Failed to send chunk request to server");
        }
    }
}

fn broadcast_chunks(v_render_dist: View<RenderDistance>, v_world_loc: View<WorldLocation>, v_chunk_gen_event: View<ChunkGenEvent>, server_handler: UniqueView<ServerHandler>) {
    let sender = &server_handler.tx;

    for (id, (render_dist, world_loc)) in (&v_render_dist, &v_world_loc).iter().with_id() {
        let Some(&addr) = server_handler.clients.get_by_right(&id) else {
            continue;
        };

        for evt in v_chunk_gen_event.iter() {
            if chunk_index_in_render_distance(&evt.0.location, &world_loc.into(), render_dist).is_some() {
                let p = Packet::reliable_unordered(addr, evt.serialize_and_compress_packet().unwrap());

                if sender.try_send(p).is_err() {
                    tracing::debug!("There was an error sending a chunk {:?} to {:?}", evt.0.location, addr);
                } else {
                    tracing::debug!("Successfully sent chunk packet");
                }
            }
        }
    }
}

// TODO: fix borrowing of storages
fn chunk_manager_update_and_request(mut all_storages: AllStoragesViewMut) {
    if let Some(reqs) = all_storages.run(chunk_manager_update) {
        all_storages.bulk_add_entity(reqs.into_iter());
    }
}

fn chunk_manager_update(delta_time: UniqueView<LastDeltaTime>, mut chunk_mgr: UniqueViewMut<ChunkManager>, vm_transform: View<Transform>, vm_local_player: View<LocalPlayer>, g_ctx: UniqueView<GraphicsContext>, mut chunk_gen_event: ViewMut<ChunkGenEvent>) -> Option<Vec<ChunkGenRequestEvent>> {
    let transform = (&vm_local_player, &vm_transform)
        .iter()
        .next()
        .expect("TODO: local player with transform didn't exist")
        .1;

    let current_chunk = ChunkLocation::from(WorldLocation(transform.position));

    let recv = chunk_gen_event.drain().collect();

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, recv, None, &g_ctx);

    (!reqs.is_empty()).then_some(reqs)
}

fn update_camera_movement(delta_time: UniqueView<LastDeltaTime>, local_player: View<LocalPlayer>, mut transform: ViewMut<Transform>, mut player_speed: ViewMut<PlayerSpeed>, input_manager: UniqueView<InputManager>) {
    let (_, transform, player_speed) = (&local_player, &mut transform, &mut player_speed)
        .iter()
        .next()
        .expect("TODO: local player did not have camera to render to");

    movement::process_movement(transform, player_speed, &input_manager, delta_time.0);
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}

fn check_collision(vm_hitbox: View<Hitbox>, mut vm_transform: ViewMut<Transform>, vm_entity: View<Entity>, world: UniqueView<ChunkManager>) {
    for (hitbox, transform, _) in (&vm_hitbox, &mut vm_transform, &vm_entity).iter() {
        let mut floored = transform.position.map(f32::floor);
        floored.y -= hitbox.0.y;
        let world_location = WorldLocation(floored);
        let chunk_location: ChunkLocation = world_location.clone().into();
        let chunk = world.get_chunk_ref_from_location(&chunk_location);
        if let Some(chunk) = chunk {
            if chunk.data.get_block(ChunkPos::from(world_location.clone())) != Block::Air {
                transform.position.y = world_location.0.y + hitbox.0.y + 1.0;
            }
        }

    }
}
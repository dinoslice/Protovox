use laminar::Packet;
use std::net::SocketAddr;
use crossbeam::channel::Sender;
use glm::Vec3;
use na::Perspective3;
use crate::chunks::chunk_manager::{ChunkManager, chunk_index_in_render_distance};
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator, AllStoragesViewMut, ViewMut, IntoIter, View, IntoWithId, EntitiesViewMut, EntitiesView};
use game::chunk::data::ChunkData;
use game::block::Block;
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use game::location::WorldLocation;
use packet::Packet as _;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::components::{Entity, Hitbox, LocalPlayer, Player, PlayerSpeed, Transform, Velocity};
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent, ClientChunkRequest, ClientInformationRequestEvent};
use crate::events::event_bus::EventBus;
use crate::input::InputManager;
use crate::multiplayer::server_connection::ServerConnection;
use crate::networking;
use crate::physics::movement::{adjust_fly_speed, apply_camera_input, process_movement};
use crate::networking::server_socket::ServerHandler;
use crate::physics::process_physics;
use crate::render_distance::RenderDistance;
use crate::rendering::gizmos;
use crate::rendering::gizmos::{BoxGizmo, GizmoLifetime, GizmoStyle};
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;

pub fn update() -> Workload {
    (
        process_input,
        process_physics,
        reset_mouse_manager_state,
        networking::update_networking,
        server_handle_client_chunk_reqs.run_if(is_hosted),
        get_generated_chunks.run_if(is_hosted),
        broadcast_chunks.run_if(is_hosted),
        chunk_manager_update_and_request,
        generate_chunks.run_if(is_hosted),
        client_request_chunks_from_server.run_if(is_multiplayer_client),
        spawn_multiplayer_player,
        gizmos::update,
        check_collision,
    ).into_sequential_workload()
}

pub fn process_input() -> Workload {
    (
        apply_camera_input,
        process_movement,
        adjust_fly_speed,
    ).into_workload()
}

fn spawn_multiplayer_player(
    // TODO: for now using this event to spawn the player's components
    mut vm_info_req_evt: ViewMut<ClientInformationRequestEvent>,

    mut entities: EntitiesViewMut,
    mut vm_player: ViewMut<Player>,
    mut vm_entity: ViewMut<Entity>,
    mut vm_transform: ViewMut<Transform>,
    mut vm_velocity: ViewMut<Velocity>,
    mut vm_player_speed: ViewMut<PlayerSpeed>,
    mut vm_hitbox: ViewMut<Hitbox>,
) {
    for (id, _) in vm_info_req_evt.drain().with_id() {
        entities.add_component(id,
            (
                &mut vm_player,
                &mut vm_entity,
                &mut vm_transform,
                &mut vm_velocity,
                &mut vm_player_speed,
                &mut vm_hitbox
            ),
            (
                Player,
                Entity,
                Transform::default(),
                Velocity::default(),
                PlayerSpeed(8.0),
                Hitbox(Vec3::new(0.6, 2.0, 0.6))
            )
        );
    }
}

fn get_generated_chunks(world_gen: UniqueView<WorldGenerator>, mut vm_entities: EntitiesViewMut, vm_chunk_gen_evt: ViewMut<ChunkGenEvent>) {
    let chunks = world_gen.receive_chunks();

    drop(world_gen);

    if !chunks.is_empty() {
        vm_entities.bulk_add_entity(vm_chunk_gen_evt, chunks);
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

    for req in reqs.drain().map(ClientChunkRequest::from) {
        let p = Packet::reliable_unordered(
            addr,
            req
                .serialize_packet()
                .expect("packet serialization failed")
        );

        if sender.try_send(p).is_err() {
            tracing::debug!("Failed to send chunk request to server");
        }
    }
}

fn server_handle_client_chunk_reqs(mut reqs: ViewMut<EventBus<ClientChunkRequest>>, mut gen_reqs: ViewMut<ChunkGenRequestEvent>, entities: EntitiesView, chunk_mgr: UniqueView<ChunkManager>, server_handler: UniqueView<ServerHandler>) {
    let sender = &server_handler.tx;

    for (id, events) in (&mut reqs).iter().with_id() {
        let Some(&addr) = server_handler.clients.get_by_right(&id) else {
            continue;
        };

        for ClientChunkRequest(loc) in events.0.drain(..) {
            match chunk_mgr.get_chunk_ref_from_location(&loc) {
                Some(cc) => {
                    use std::mem;

                    assert_eq!(mem::size_of::<ChunkData>(), mem::size_of::<ChunkGenEvent>());

                    let gen_evt = unsafe { mem::transmute::<&ChunkData, &ChunkGenEvent>(&cc.data) }; // TODO: eventually figure out how to get rid of this transmute without copying

                    send_chunk(sender, addr, gen_evt);
                }
                None => {
                    tracing::debug!("server didn't have chunk at {loc:?}, asking world generator to generate.");
                    entities.add_component(id, &mut gen_reqs, ChunkGenRequestEvent(loc));
                }
            }
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
                send_chunk(sender, addr, evt);
            }
        }
    }
}

fn send_chunk(sender: &Sender<Packet>, client_addr: SocketAddr, gen_evt: &ChunkGenEvent) {
    let p = Packet::reliable_unordered(client_addr, gen_evt.serialize_and_compress_packet().expect("packet serialization failed"));

    if sender.try_send(p).is_err() {
        tracing::debug!("There was an error sending a chunk {:?} to {:?}", gen_evt.0.location, client_addr);
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

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}

fn check_collision(
    vm_hitbox: View<Hitbox>,
    mut vm_transform: ViewMut<Transform>,
    vm_entity: View<Entity>,
    mut world: UniqueViewMut<ChunkManager>,

    mut entities: EntitiesViewMut,
    mut vm_box_gizmos: ViewMut<BoxGizmo>,
) {
    for (hitbox, transform, _) in (&vm_hitbox, &mut vm_transform, &vm_entity).iter() {
        let half_hitbox = hitbox.0 * 0.5;

        let min_extent = transform.position - half_hitbox;
        let max_extent = transform.position + half_hitbox;

        let min_floor = min_extent.map(|n| n.floor() as i32);
        let max_floor = max_extent.map(|n| n.floor() as i32);

        for x in min_floor.x..=max_floor.x {
            for y in min_floor.y..=max_floor.y {
                for z in min_floor.z..=max_floor.z {
                    let world_loc = WorldLocation(Vec3::new(x as f32, y as f32, z as f32));
                    let chunk_loc = ChunkLocation::from(&world_loc);

                    if let Some(chunk) = world.get_chunk_ref_from_location_mut(&chunk_loc) {
                        let chunk_pos = ChunkPos::from(&world_loc);

                        if chunk.data.get_block(chunk_pos) != Block::Air {
                            chunk.data.set_block(chunk_pos, Block::Air);
                            chunk.dirty = true;
                        }
                    }
                }
            }
        }

        entities.add_entity(&mut vm_box_gizmos, BoxGizmo::from_corners(
            min_extent,
            max_extent,
            GizmoStyle::stroke(rgb::Rgb { r: 1.0, g: 0.0, b: 0.0 }),
            GizmoLifetime::SingleFrame,
        ));
    }
}
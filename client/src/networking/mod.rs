use std::net::SocketAddr;
use crossbeam::channel::Sender;
use laminar::Packet;
use packet::Packet as _;
use shipyard::{AllStoragesView, EntitiesView, IntoIter, IntoWithId, IntoWorkload, SystemModificator, UniqueView, View, ViewMut, Workload, WorkloadModificator};
use game::chunk::data::ChunkData;
use game::location::WorldLocation;
use crate::application::exit::ExitRequested;
use crate::chunks::chunk_manager::{chunk_index_in_render_distance, ChunkManager};
use crate::components::{LocalPlayer, Transform};
use crate::events::{BlockUpdateEvent, ChunkGenEvent, ChunkGenRequestEvent, ClientChunkRequest, ClientSettingsRequestEvent, ClientTransformUpdate, ConnectionRequest, ConnectionSuccess, KickedByServer};
use crate::events::event_bus::EventBus;
use crate::events::render_distance::RenderDistanceUpdateEvent;
use crate::networking::chat::{client_handle_chat_messages, send_chat_message};
use crate::networking::keep_alive::server_send_keep_alive;
use crate::networking::server_connection::{client_process_network_events_multiplayer, ServerConnection};
use crate::networking::server_handler::{server_process_network_events, ServerHandler};
use crate::render_distance::RenderDistance;

pub mod types;
pub mod server_handler;
pub mod keep_alive;
pub mod server_connection;
pub mod chat;

pub fn update_networking_server() -> Workload {
    (
        server_process_network_events,
        (
            server_broadcast_chunks,
            server_broadcast_block_updates,
            send_chat_message,
            server_process_client_connection_req,
            server_update_client_transform,
            server_request_client_settings,
            server_process_render_dist_update,
            server_handle_client_chunk_reqs,
            server_send_keep_alive,
        ).into_workload()
    ).into_sequential_workload()
}

pub fn update_networking_client() -> Workload {
    (
        // PRE-PACKET RECV NETWORKING
        (
            client_send_block_updates,
        ).into_workload(),
        
        client_process_network_events_multiplayer,
        
        // POST-PACKET RECV NETWORKING
        (
            client_handle_kicked_by_server,
            client_handle_chat_messages,
            client_acknowledge_connection_success,
            client_update_position,
            client_request_chunks_from_server,
            client_send_settings,
        ).into_workload(),
    ).into_sequential_workload()
}

fn client_send_block_updates(server_connection: UniqueView<ServerConnection>, v_block_update_evt: View<BlockUpdateEvent>) {
    let tx = &server_connection.tx;
    let server_addr = server_connection.server_addr;
    
    for evt in (&v_block_update_evt).iter() {
        let packet = Packet::reliable_unordered(
            server_addr,
            evt.serialize_packet()
                .expect("packet serialization failed")
        );
        
        if tx.try_send(packet).is_err() {
            tracing::error!("Failed to send {evt:?} to server");
        }
    }
}

fn server_broadcast_block_updates(server_handler: UniqueView<ServerHandler>, v_block_update_evt: View<BlockUpdateEvent>, v_block_update_evt_bus: View<EventBus<BlockUpdateEvent>>) {
    let tx = &server_handler.tx;
    
    for (&addr, &id) in &server_handler.clients {
        for evt in (&v_block_update_evt).iter() {
            
            let packet = Packet::reliable_unordered(
                addr,
                evt.serialize_packet()
                    .expect("packet serialization failed"),
            );

            if tx.try_send(packet).is_err() {
                tracing::error!("Failed to send block update to client {addr:?}");
            }
        }
        
        for (bus_id, bus) in (&v_block_update_evt_bus).iter().with_id() {
            if bus_id == id {
                continue;
            }
            
            for evt in &bus.0 {
                let packet = Packet::reliable_unordered(
                    addr,
                    evt.serialize_packet()
                        .expect("packet serialization failed"),
                );

                if tx.try_send(packet).is_err() {
                    tracing::error!("Failed to send block update to client {addr:?}");
                }
            }
        }
    }
}

fn server_process_client_connection_req(mut vm_conn_req: ViewMut<ConnectionRequest>, server_handler: UniqueView<ServerHandler>) {
    vm_conn_req.retain(|id, _| {
        match server_handler.clients.get_by_right(&id) {
            None => {
                tracing::debug!("Client has disconnected!");
                false
            },
            Some(&addr) => {
                let payload = ConnectionSuccess.serialize_packet().expect("packet serialization failed");

                let p = Packet::reliable_unordered(addr, payload);

                if let Err(err) = server_handler.tx.try_send(p) {
                    tracing::warn!("There was an error sending to client: {addr:?}, err: {err:?}");
                    true
                } else {
                    tracing::debug!("Sent ConnectionSuccess to {addr:?}");
                    false
                }
            }
        }
    });
}

fn server_request_client_settings(mut vm_client_settings_req: ViewMut<ClientSettingsRequestEvent>, server_handler: UniqueView<ServerHandler>) {
    vm_client_settings_req.retain(|id, evt| {
        match server_handler.clients.get_by_right(&id) {
            None => {
                tracing::debug!("Client has disconnected!");
                false
            },
            Some(&addr) => {
                let payload = evt.serialize_packet().expect("packet serialization failed");

                let p = Packet::reliable_unordered(addr, payload);

                if let Err(err) = server_handler.tx.try_send(p) {
                    tracing::error!("failed to send packet to client at {addr:?}: {err:?}");
                    true
                } else {
                    false
                }
            }
        }
    });
}

fn client_send_settings(mut vm_client_settings_req: ViewMut<ClientSettingsRequestEvent>, server_connection: UniqueView<ServerConnection>, chunk_mgr: UniqueView<ChunkManager>) {
    if vm_client_settings_req.drain().next().is_some() {
        let p = Packet::reliable_unordered(
            server_connection.server_addr,
            RenderDistanceUpdateEvent(chunk_mgr.render_distance().clone()) // TODO: handle a different way
                .serialize_packet()
                .expect("packet serialization failed")
        );

        if let Err(err) = server_connection.tx.try_send(p) {
            tracing::error!("failed to send packet to server: {err:?}");
        }
    }
}

fn server_process_render_dist_update(mut vm_render_distance_update: ViewMut<RenderDistanceUpdateEvent>, entities: EntitiesView, mut vm_render_dist: ViewMut<RenderDistance>) {
    vm_render_distance_update.drain().with_id().for_each(|(id, evt)| {
        entities.add_component(id, &mut vm_render_dist, evt.0);
    });
}

fn client_acknowledge_connection_success(mut vm_conn_success: ViewMut<ConnectionSuccess>) {
    vm_conn_success.drain().for_each(|evt| tracing::debug!("Received {evt:?}"));
}

fn client_update_position(local_player: View<LocalPlayer>, vm_transform: View<Transform>, server_conn: UniqueView<ServerConnection>) {
    let transform = (&local_player, &vm_transform)
        .iter()
        .next()
        .expect("TODO: local player did not have transform")
        .1;

    let p = Packet::unreliable_sequenced(
        server_conn.server_addr,
        ClientTransformUpdate(transform.clone())
            .serialize_packet()
            .expect("packet serialization failed"),
        None,
    );

    server_conn.tx
        .try_send(p)
        .expect("packet serialization failed");
}

fn server_update_client_transform(mut vm_client_pos_update: ViewMut<ClientTransformUpdate>, mut vm_transform: ViewMut<Transform>, entities: EntitiesView) {
    vm_client_pos_update.drain().with_id().for_each(|(id, evt)| {
        entities.add_component(id, &mut vm_transform, evt.0);
    });
}

fn server_handle_client_chunk_reqs(mut reqs: ViewMut<EventBus<ClientChunkRequest>>, mut gen_reqs: ViewMut<ChunkGenRequestEvent>, entities: EntitiesView, chunk_mgr: UniqueView<ChunkManager>, server_handler: UniqueView<ServerHandler>) {
    let sender = &server_handler.tx;

    for (id, events) in (&mut reqs).iter().with_id() {
        let Some(&addr) = server_handler.clients.get_by_right(&id) else {
            continue;
        };

        for ClientChunkRequest(loc) in events.0.drain(..) {
            match chunk_mgr.get_chunk_ref(&loc) {
                Some(cc) => {
                    use std::mem;

                    assert_eq!(mem::size_of::<ChunkData>(), mem::size_of::<ChunkGenEvent>());

                    let gen_evt = unsafe { mem::transmute::<&ChunkData, &ChunkGenEvent>(&cc.data) }; // TODO: eventually figure out how to get rid of this transmute without copying

                    send_chunk(sender, addr, gen_evt);
                }
                None => {
                    entities.add_component(id, &mut gen_reqs, ChunkGenRequestEvent(loc));
                }
            }
        }
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

fn server_broadcast_chunks(v_render_dist: View<RenderDistance>, v_world_loc: View<WorldLocation>, v_chunk_gen_event: View<ChunkGenEvent>, server_handler: UniqueView<ServerHandler>) {
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

fn client_handle_kicked_by_server(v_kick_evt: View<KickedByServer>, storages: AllStoragesView) {
    if let Some(KickedByServer(reason)) = (&v_kick_evt)
        .iter()
        .next()
    {
        tracing::debug!("You have been kicked from the server: {reason}");
        storages.add_unique(ExitRequested);
    }
}

// macro_rules! retain_unsent {
//     ($vm:expr, $server_handler:expr) => {
//         $vm.retain(|id, evt|) {
//             match $server_handler.get_by_right(&id) {
//                 None => {
//                     tracing::debug!("Client has disconnected");
//                     false
//                 }
//                 Some(&addr) {
//                     laminar::Packet::new(
//
//                     )
//                 }
//             }
//         }
//     }
// }
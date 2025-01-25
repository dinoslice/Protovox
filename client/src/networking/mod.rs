use std::net::SocketAddr;
use crossbeam::channel::Sender;
use laminar::Packet;
use packet::Packet as _;
use shipyard::{AllStoragesView, EntitiesView, EntitiesViewMut, IntoIter, IntoWithId, IntoWorkload, UniqueView, View, ViewMut, Workload};
use game::chunk::data::ChunkData;
use crate::application::exit::ExitRequested;
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{LocalPlayer, Transform};
use crate::events::{BlockUpdateEvent, ChunkGenEvent, ChunkGenRequestEvent, ClientChunkRequest, ClientSettingsRequestEvent, ClientTransformUpdate, ConnectionRequest, ConnectionSuccess, KickedByServer};
use crate::events::event_bus::EventBus;
use crate::events::render_distance::RenderDistanceUpdateEvent;
use crate::networking::keep_alive::server_send_keep_alive;
use crate::networking::server_connection::{client_process_network_events_multiplayer, ServerConnection};
use crate::networking::server_handler::{server_process_network_events, ServerHandler};
use crate::render_distance::RenderDistance;

pub mod types;
pub mod server_handler;
pub mod keep_alive;
pub mod server_connection;

pub fn client_send_block_updates(server_connection: UniqueView<ServerConnection>, v_block_update_evt: View<BlockUpdateEvent>) {
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

pub fn server_broadcast_block_updates(server_handler: UniqueView<ServerHandler>, v_block_update_evt: View<BlockUpdateEvent>, v_block_update_evt_bus: View<EventBus<BlockUpdateEvent>>) {
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

pub fn server_process_client_connection_req(mut vm_conn_req: ViewMut<ConnectionRequest>, server_handler: UniqueView<ServerHandler>) {
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

pub fn server_request_client_settings(mut vm_client_settings_req: ViewMut<ClientSettingsRequestEvent>, server_handler: UniqueView<ServerHandler>) {
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

pub fn client_send_settings(mut vm_client_settings_req: ViewMut<ClientSettingsRequestEvent>, server_connection: UniqueView<ServerConnection>, v_local_player: View<LocalPlayer>, v_render_dist: View<RenderDistance>) {
    if vm_client_settings_req.drain().next().is_some() {
        let (render_dist, ..) = (&v_render_dist, &v_local_player)
            .iter()
            .next()
            .expect("local player must have render distance");


        let p = Packet::reliable_unordered(
            server_connection.server_addr,
            RenderDistanceUpdateEvent(render_dist.clone()) // TODO: handle a different way
                .serialize_packet()
                .expect("packet serialization failed")
        );

        if let Err(err) = server_connection.tx.try_send(p) {
            tracing::error!("failed to send packet to server: {err:?}");
        }
    }
}

pub fn server_process_render_dist_update(mut vm_render_distance_update: ViewMut<RenderDistanceUpdateEvent>, entities: EntitiesView, mut vm_render_dist: ViewMut<RenderDistance>) {
    vm_render_distance_update.drain().with_id().for_each(|(id, evt)| {
        entities.add_component(id, &mut vm_render_dist, evt.0);
    });
}

pub fn client_acknowledge_connection_success(mut vm_conn_success: ViewMut<ConnectionSuccess>) {
    vm_conn_success.drain().for_each(|evt| tracing::debug!("Received {evt:?}"));
}

pub fn client_update_position(local_player: View<LocalPlayer>, vm_transform: View<Transform>, server_conn: UniqueView<ServerConnection>) {
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

pub fn server_update_client_transform(mut vm_client_pos_update: ViewMut<ClientTransformUpdate>, mut vm_transform: ViewMut<Transform>, entities: EntitiesView) {
    vm_client_pos_update.drain().with_id().for_each(|(id, evt)| {
        entities.add_component(id, &mut vm_transform, evt.0);
    });
}

pub fn server_handle_client_chunk_reqs(mut reqs: ViewMut<EventBus<ClientChunkRequest>>, mut gen_reqs: ViewMut<ChunkGenRequestEvent>, mut entities: EntitiesViewMut, chunk_mgr: UniqueView<ChunkManager>, server_handler: UniqueView<ServerHandler>) {
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
                    // TODO: eventually maybe revert this back to add component
                    entities.add_entity(&mut gen_reqs, ChunkGenRequestEvent(loc));
                }
            }
        }
    }
}

pub fn client_request_chunks_from_server(mut reqs: ViewMut<ChunkGenRequestEvent>, server_connection: UniqueView<ServerConnection>) {
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

pub fn server_broadcast_chunks(v_render_dist: View<RenderDistance>, v_transform: View<Transform>, v_chunk_gen_event: View<ChunkGenEvent>, server_handler: UniqueView<ServerHandler>) {
    let sender = &server_handler.tx;

    for (id, (render_dist, transform)) in (&v_render_dist, &v_transform).iter().with_id() {
        let Some(&addr) = server_handler.clients.get_by_right(&id) else {
            continue;
        };

        for evt in v_chunk_gen_event.iter() {
            if ChunkManager::in_render_distance_with(&evt.0.location, &transform.get_loc(), render_dist) {
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

pub fn client_handle_kicked_by_server(v_kick_evt: View<KickedByServer>, storages: AllStoragesView) {
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
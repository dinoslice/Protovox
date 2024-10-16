use laminar::Packet;
use packet::Packet as _;
use shipyard::{EntitiesView, IntoWorkload, SystemModificator, UniqueView, ViewMut, Workload};
use game::location::WorldLocation;
use crate::camera::Camera;
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::events::{ChunkGenRequestEvent, ClientPositionUpdate, ClientSettingsRequestEvent, ConnectionRequest, ConnectionSuccess};
use crate::events::render_distance::RenderDistanceUpdateEvent;
use crate::multiplayer::server_connection::{process_network_events_multiplayer_client, ServerConnection};
use crate::networking::server_socket::{process_network_events_system, ServerHandler};
use crate::render_distance::RenderDistance;

pub mod types;
pub mod server_socket;

pub fn update_networking() -> Workload {
    (
        process_network_events_system, // internally, run_if(is_hosted)
        process_network_events_multiplayer_client, // internally, run_if(is_multiplayer_client)
        server_process_client_connection_req.run_if(is_hosted),
        client_acknowledge_connection_success.run_if(is_multiplayer_client),
        client_update_position.run_if(is_multiplayer_client),
        server_update_client_pos.run_if(is_hosted),
        server_request_client_settings.run_if(is_hosted),
        client_send_settings.run_if(is_multiplayer_client),
        server_process_render_dist_update.run_if(is_hosted)
    ).into_workload()
}

fn server_process_client_connection_req(mut vm_conn_req: ViewMut<ConnectionRequest>, server_handler: UniqueView<ServerHandler>) {
    vm_conn_req.retain(|id, _| {
        match server_handler.clients.get_by_right(&id) {
            None => {
                tracing::debug!("Client has disconnected!");
                false
            },
            Some(&addr) => {
                let payload = ConnectionSuccess.serialize_packet().unwrap();

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
                let payload = evt.serialize_packet().unwrap();

                let p = Packet::reliable_unordered(addr, payload);

                if let Err(err) = server_handler.tx.try_send(p) {
                    tracing::warn!("There was an error sending to client: {addr:?}, err: {err:?}");
                    true
                } else {
                    tracing::debug!("Sent ClientSettingsRequestEvent to {addr:?}");
                    false
                }
            }
        }
    });
}

fn client_send_settings(mut vm_client_settings_req: ViewMut<ClientSettingsRequestEvent>, server_connection: UniqueView<ServerConnection>) {
    if let Some(_) = vm_client_settings_req.drain().next() {
        let p = Packet::reliable_unordered(
            server_connection.server_addr,
            RenderDistanceUpdateEvent(RenderDistance::default()) // TODO: handle a different way
                .serialize_packet()
                .unwrap()
        );

        server_connection
            .tx
            .try_send(p)
            .unwrap();
    }
}

fn server_process_render_dist_update(mut vm_render_distance_update: ViewMut<RenderDistanceUpdateEvent>, entities: EntitiesView, mut vm_render_dist: ViewMut<RenderDistance>) {
    use shipyard::Get;

    vm_render_distance_update.drain().with_id().for_each(|(id, evt)| {
        entities.add_component(id, &mut vm_render_dist, evt.0);
    });
}

fn client_acknowledge_connection_success(mut vm_conn_success: ViewMut<ConnectionSuccess>) {
    vm_conn_success.drain().for_each(|evt| tracing::debug!("Received {evt:?}"));
}

#[allow(dead_code)]
fn client_request_chunk_gen(mut vm_chunk_gen_req: ViewMut<ChunkGenRequestEvent>, server_handler: UniqueView<ServerHandler>) {
    vm_chunk_gen_req.retain(|id, evt| {
        match server_handler.clients.get_by_right(&id) {
            None => {
                tracing::debug!("Client has disconnected!");
                false
            },
            Some(&addr) => {
                let payload = evt.serialize_packet().unwrap();

                let p = Packet::reliable_unordered(addr, payload);

                if let Err(err) = server_handler.tx.try_send(p) {
                    tracing::warn!("There was an error sending to client: {addr:?}, err: {err:?}");
                    true
                } else {
                    tracing::debug!("Sent {evt:?} to {addr:?}");
                    false
                }
            }
        }
    });
}

pub fn client_update_position(cam: UniqueView<Camera>, server_conn: UniqueView<ServerConnection>) {
    let world_pos = WorldLocation(cam.position);

    let p = Packet::unreliable_sequenced(
        server_conn.server_addr,
        ClientPositionUpdate(world_pos)
            .serialize_packet()
            .unwrap(),
        None,
    );

    server_conn.tx
        .try_send(p)
        .unwrap();
}

pub fn server_update_client_pos(mut vm_client_pos_update: ViewMut<ClientPositionUpdate>, mut vm_world_pos: ViewMut<WorldLocation>, entities: EntitiesView) {
    vm_client_pos_update.drain().with_id().for_each(|(id, evt)| {
        entities.add_component(id, &mut vm_world_pos, evt.0);
    });
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
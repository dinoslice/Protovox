use laminar::Packet;
use packet::Packet as _;
use shipyard::{EntitiesView, IntoIter, IntoWorkload, SystemModificator, UniqueView, View, ViewMut, Workload};
use crate::components::{LocalPlayer, Transform};
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::events::{ClientTransformUpdate, ClientSettingsRequestEvent, ConnectionRequest, ConnectionSuccess};
use crate::events::render_distance::RenderDistanceUpdateEvent;
use crate::multiplayer::server_connection::{process_network_events_multiplayer_client, ServerConnection};
use crate::networking::keep_alive::send_keep_alive;
use crate::networking::server_socket::{process_network_events_system, ServerHandler};
use crate::render_distance::RenderDistance;

pub mod types;
pub mod server_socket;
pub mod keep_alive;

pub fn update_networking() -> Workload {
    (
        process_network_events_system, // internally, run_if(is_hosted)
        process_network_events_multiplayer_client, // internally, run_if(is_multiplayer_client)
        server_process_client_connection_req.run_if(is_hosted),
        client_acknowledge_connection_success.run_if(is_multiplayer_client),
        client_update_position.run_if(is_multiplayer_client),
        server_update_client_transform.run_if(is_hosted),
        server_request_client_settings.run_if(is_hosted),
        client_send_settings.run_if(is_multiplayer_client),
        server_process_render_dist_update.run_if(is_hosted),
        send_keep_alive.run_if(is_hosted),
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

fn client_send_settings(mut vm_client_settings_req: ViewMut<ClientSettingsRequestEvent>, server_connection: UniqueView<ServerConnection>) {
    if vm_client_settings_req.drain().next().is_some() {
        let p = Packet::reliable_unordered(
            server_connection.server_addr,
            RenderDistanceUpdateEvent(RenderDistance::default()) // TODO: handle a different way
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
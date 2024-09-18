use std::net::SocketAddr;
use laminar::Packet;
use packet::Packet as _;
use shipyard::{IntoWorkload, SystemModificator, UniqueView, ViewMut, Workload, WorkloadModificator};
use crate::environment::is_hosted;
use crate::events::{ConnectionRequest, ConnectionSuccess};
use crate::multiplayer::server_connection::process_network_events_multiplayer_client;
use crate::networking::server_socket::{process_network_events_system, ServerHandler};

pub mod types;
pub mod server_socket;

pub fn update_networking() -> Workload {
    (
        process_network_events_system, // internally, run_if(is_hosted)
        process_network_events_multiplayer_client, // internally, run_if(is_multiplayer_client)
        server_process_client_connection_req.run_if(is_hosted)
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
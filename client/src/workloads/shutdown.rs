use laminar::Packet;
use shipyard::{SystemModificator, UniqueViewMut};
use shipyard::{IntoWorkload, Workload};
use packet::Packet as _;
use crate::environment::is_hosted;
use crate::events::KickedByServer;
use crate::networking::server_handler::ServerHandler;

pub fn shutdown() -> Workload {
    (
        // -- SHUTDOWN -- //
        disconnect_connected_players.run_if(is_hosted),
    ).into_sequential_workload()
}

pub fn disconnect_connected_players(server_handler: UniqueViewMut<ServerHandler>) {
    let tx = &server_handler.tx;
    
    let kick_packet = KickedByServer("Server closed".into())
        .serialize_packet()
        .expect("packet serialization failed");
    
    for addr in server_handler.clients.left_values() {
        if tx.try_send(Packet::reliable_unordered(*addr, kick_packet.clone())).is_err() {
            tracing::error!("Failed to send kick packet to {addr:?}");
        }
    }
}
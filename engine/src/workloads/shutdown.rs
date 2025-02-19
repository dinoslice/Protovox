use laminar::Packet;
use shipyard::{UniqueView, UniqueViewMut};
use networking::{PacketRegistry, RuntimePacket};
use crate::events::KickedByServer;
use crate::networking::server_handler::ServerHandler;

pub fn disconnect_connected_players(server_handler: UniqueViewMut<ServerHandler>, registry: UniqueView<PacketRegistry>) {
    let tx = &server_handler.tx;

    let id = registry
        .identifier_of()
        .expect("should be registered");
    
    let kick_packet = KickedByServer("Server closed".into())
        .serialize_uncompressed_with_id(id)
        .expect("packet serialization failed");
    
    for addr in server_handler.clients.left_values() {
        if tx.try_send(Packet::reliable_unordered(*addr, kick_packet.clone())).is_err() {
            tracing::error!("Failed to send kick packet to {addr:?}");
        }
    }
}
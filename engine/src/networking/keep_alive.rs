use std::time::{Duration, Instant};
use laminar::Packet;
use shipyard::{Unique, UniqueOrDefaultViewMut, UniqueView};
use networking::{PacketRegistry, RuntimePacket};
use packet::Packet as _;
use crate::events::{ClientTransformUpdate, KeepAlive};
use crate::networking::server_handler::ServerHandler;

#[derive(Unique, Debug)]
pub struct LastKeepAlive(pub Instant);

impl Default for LastKeepAlive {
    fn default() -> Self {
        Self(Instant::now())
    }
}

pub fn server_send_keep_alive(mut last_keep_alive: UniqueOrDefaultViewMut<LastKeepAlive>, server_handler: UniqueView<ServerHandler>, registry: UniqueView<PacketRegistry>) {
    let tx = &server_handler.tx;

    let id = registry
        .identifier_of()
        .expect("should be registered");

    if last_keep_alive.0.elapsed() > Duration::from_secs(5) {
        for &addr in server_handler.clients.left_values() {
            let keep_alive = Packet::unreliable(
                addr,
                KeepAlive
                    .serialize_uncompressed_with_id(id)
                    .expect("Packet Serialization Error")
            );

            if tx.send(keep_alive).is_err() {
                tracing::error!("There was an error sending keep alive packet to {addr:?}")
            }
        }

        last_keep_alive.0 = Instant::now();
    }
}
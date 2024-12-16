use std::time::{Duration, Instant};
use laminar::Packet;
use shipyard::{Unique, UniqueOrDefaultViewMut, UniqueView};
use packet::Packet as _;
use crate::events::KeepAlive;
use crate::networking::server_handler::ServerHandler;

#[derive(Unique, Debug)]
pub struct LastKeepAlive(pub Instant);

impl Default for LastKeepAlive {
    fn default() -> Self {
        Self(Instant::now())
    }
}

pub fn server_send_keep_alive(mut last_keep_alive: UniqueOrDefaultViewMut<LastKeepAlive>, server_handler: UniqueView<ServerHandler>) {
    let tx = &server_handler.tx;

    if last_keep_alive.0.elapsed() > Duration::from_secs(5) {
        for &addr in server_handler.clients.left_values() {
            let keep_alive = Packet::unreliable(
                addr,
                KeepAlive
                    .serialize_packet()
                    .expect("Packet Serialization Error")
            );

            if tx.send(keep_alive).is_err() {
                tracing::error!("There was an error sending keep alive packet to {addr:?}")
            }
        }

        last_keep_alive.0 = Instant::now();
    }
}
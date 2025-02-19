use std::time::Duration;
use std::net::SocketAddr;
use std::thread;
use crossbeam::channel::{Receiver, Sender};
use laminar::{Packet, Socket, SocketEvent};
use shipyard::{AllStoragesViewMut, Unique, UniqueView};
use networking::{PacketIdentifier, PacketRegistry, RuntimePacket};
use packet::Packet as _;
use crate::events::ConnectionRequest;

#[derive(Unique)]
pub struct ServerConnection {
    pub server_addr: SocketAddr,
    pub tx: Sender<Packet>,
    pub rx: Receiver<SocketEvent>,
}

impl ServerConnection {
    pub fn bind(server_addr: impl Into<SocketAddr>, packet_id: PacketIdentifier<ConnectionRequest>) -> Self {
        let config = laminar::Config {
            max_packet_size: 64 * 1024,
            max_fragments: 64,
            fragment_size: 1024,
            idle_connection_timeout: Duration::from_secs(6),
            .. Default::default()
        };

        let mut socket = Socket::bind_any_with_config(config)
            .expect("unable to bind to address");

        let tx = socket.get_packet_sender();
        let rx = socket.get_event_receiver();

        let server_addr = server_addr.into();

        tracing::debug!("Connected client at {} to server at {server_addr:?}", socket.local_addr().expect("server addr should be initialized"));

        let _ = thread::spawn(move || socket.start_polling());

        let connection_req = Packet::reliable_ordered(
            server_addr,
            ConnectionRequest
                .serialize_uncompressed_with_id(packet_id)
                .expect("packet serialization failed"),
            None, // TODO: configure stream ids
        );

        tx.send(connection_req).expect("sent connection req");

        Self {
            server_addr,
            tx,
            rx,
        }
    }
}

pub fn client_process_network_events_multiplayer(mut storages: AllStoragesViewMut) {
    let addr = storages
        .borrow::<UniqueView<ServerConnection>>()
        .expect("server conn reborrow")
        .server_addr;

    loop {
        let server_conn = storages
            .borrow::<UniqueView<ServerConnection>>()
            .expect("server conn reborrow");

        let res = server_conn.rx.try_recv();

        let Ok(evt) = res else {
            break;
        };

        drop(server_conn);

        match evt {
            SocketEvent::Packet(packet) => {
                assert_eq!(packet.addr(), addr);

                let payload = packet.payload();

                if let Some(type_id) = PacketRegistry::untyped_identifier_from(payload) {
                    let opt = storages
                        .borrow::<UniqueView<PacketRegistry>>()
                        .expect("registry to be initialized")
                        .deserializer_for_untyped_id(type_id);

                    if let Some((deserializer, _)) = opt {
                        let _id = deserializer(payload, &mut storages)
                            .expect("didn't fail");
                    }
                }
            }
            SocketEvent::Connect(addr) => {
                tracing::debug!("something just connected to the client, {addr:?}");
            }
            SocketEvent::Timeout(addr) => {
                tracing::debug!("socket timeout, {addr:?}");
            }
            SocketEvent::Disconnect(addr) => {
                tracing::debug!("disconnected from the client, {addr:?}");
            }
        }
    }
}
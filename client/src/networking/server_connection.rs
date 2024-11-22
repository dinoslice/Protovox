use std::time::Duration;
use std::net::SocketAddr;
use std::thread;
use crossbeam::channel::{Receiver, Sender};
use laminar::{Packet, Socket, SocketEvent};
use shipyard::{AllStoragesViewMut, Unique, UniqueView};
use packet::Packet as _;
use crate::environment::is_multiplayer_client;
use crate::events;

#[derive(Unique)]
pub struct ServerConnection {
    pub server_addr: SocketAddr,
    pub tx: Sender<Packet>,
    pub rx: Receiver<SocketEvent>,
}

impl ServerConnection {
    pub fn bind(server_addr: impl Into<SocketAddr>) -> Self {
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
            events::ConnectionRequest.serialize_packet().expect("packet serialization failed"),
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
                add_packet(packet.payload(), &mut storages);
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

fn add_packet(buffer: &[u8], storages: &mut AllStoragesViewMut) {
    use crate::networking::types::PacketType;
    use packet::{PacketHeader, Packet as _};

    macro_rules! register_packets {
            ($bytes:expr, $storages:expr, { $($packet_type:ident => $decompress:expr),* $(,)? }) => {
                register_packets!($bytes, $storages, { $($packet_type => $packet_type => $decompress),* });
            };
            ($bytes:expr, $storages:expr, { $($packet_type:ident => $packet_struct:ident => $decompress:expr),* $(,)? }) => {
                match PacketType::from_buffer($bytes) {
                    Some(ty) => match ty {
                        $(
                            PacketType::$packet_type => match $decompress {
                                true => if let Some(packet) = $packet_struct::decompress_and_deserialize_unchecked($bytes) {
                                    $storages.add_entity(packet);
                                } else {
                                    println!("{ty:?} data was malformed");
                                },
                                false => if let Some(packet) = $packet_struct::deserialize_unchecked($bytes) {
                                    $storages.add_entity(packet);
                                } else {
                                    println!("{ty:?} data was malformed");
                                },
                            }
                        )*
                        _ => println!("Packet {:?} isn't registered", ty),
                    },
                    None => println!("Packet ID couldn't be determined"),
                }
            };
        }

    use crate::events::*;
    use crate::events::render_distance::*;

    register_packets!(buffer, storages, {
        ChunkGenEvent => true,

        RenderDistanceRequestEvent => false,
        ClientSettingsRequestEvent => false,
        
        BlockUpdateEvent => false,

        ConnectionSuccess => false,
        
        KickedByServer => false,

        KeepAlive => false,
        ChatMessage => false,
    });
}
use std::net::SocketAddr;
use std::thread;
use bimap::{BiHashMap, BiMap, Overwritten};
use crossbeam::channel::{Receiver, Sender, TryRecvError};
use glm::U16Vec3;
use hashbrown::HashMap;
use laminar::{Socket, SocketEvent};
use shipyard::{AllStoragesViewMut, EntityId, Unique, UniqueView, UniqueViewMut};
use game::location::WorldLocation;
use packet::PacketHeader;
use crate::environment::{Environment, is_hosted};
use crate::events::{ClientInformationRequestEvent, ClientSettingsRequestEvent, ConnectionRequest, PacketType};
use crate::events::render_distance::RenderDistanceRequestEvent;

#[derive(Unique)]
pub struct ServerHandler {
    pub tx: Sender<laminar::Packet>,
    pub rx: Receiver<SocketEvent>,
    pub clients: BiHashMap<SocketAddr, EntityId>,
}

impl ServerHandler {
    pub fn new() -> Self {
        let mut socket = Socket::bind_any().unwrap();
        tracing::debug!("Bound server to socket {:?}", socket.local_addr());
        let tx = socket.get_packet_sender();
        let rx = socket.get_event_receiver();

        let _polling = thread::spawn(move || socket.start_polling());

        Self {
            tx,
            rx,
            clients: BiHashMap::default(),
        }
    }
}

pub fn process_network_events_system(mut storages: AllStoragesViewMut) {
    // TODO: temp fix bc of run_if crash
    if !storages.run(is_hosted) {
        return;
    }

    // TODO: fix insane battle with borrow checker
    loop {
        let mut server_handler = storages
            .borrow::<UniqueViewMut<ServerHandler>>()
            .expect("ServerHandler initialized");

        let res = server_handler.rx.try_recv();
        drop(server_handler);

        if let Ok(evt) = res {
            match evt {
                SocketEvent::Packet(p) => {
                    let mut server_handler = storages
                        .borrow::<UniqueViewMut<ServerHandler>>()
                        .expect("ServerHandler re-borrowed");

                    let id = server_handler
                        .clients
                        .get_by_left(&p.addr())
                        .copied();

                    drop(server_handler);

                    let payload = p.payload();
                    let addr = p.addr();

                    match id {
                        None => {
                            tracing::debug!("received packet from new address");

                            let Some(PacketType::ConnectionRequest) = PacketType::from_buffer(payload) else {
                                tracing::warn!("First packet received from {:?} was not ConnectionRequest", addr);
                                continue;
                            };

                            let id = storages.add_entity((
                                ConnectionRequest,
                                ClientInformationRequestEvent,
                                ClientSettingsRequestEvent,
                            ));

                            tracing::debug!("new joined client from {addr:?} has id {id:?}");

                            let mut server_handler = storages
                                .borrow::<UniqueViewMut<ServerHandler>>()
                                .expect("ServerHandler re-borrowed");

                            if let Err((addr, id)) = server_handler.clients.insert_no_overwrite(addr, id) {
                                drop(server_handler);
                                tracing::error!("Multiple clients connected from address {addr:?}, throwing away current connection request.");
                                storages.delete_entity(id);
                            } else {
                                tracing::debug!("Successfully connected client from {addr:?}!");
                            }
                        }
                        Some(id) => add_packet(payload, id, &mut storages),
                    }
                }
                SocketEvent::Connect(addr) => {
                    tracing::debug!("SocketEvent::Connect from {addr:?}")
                }
                SocketEvent::Timeout(_) => {
                    // Handle timeout event here
                }
                SocketEvent::Disconnect(addr) => {
                    let mut server_handler = storages
                        .borrow::<UniqueViewMut<ServerHandler>>()
                        .expect("ServerHandler re-borrowed");

                    if let Some((_, id)) = server_handler.clients.remove_by_left(&addr) {
                        drop(server_handler);
                        storages.delete_entity(id);
                    } else {
                        tracing::error!("Client disconnected at {addr:?}, but it never existed.");
                    }
                }
            }
        } else {
            break;
        }
    }
}

fn add_packet(buffer: &[u8], id: EntityId, storages: &mut AllStoragesViewMut) {
    use crate::networking::types::PacketType;
    use packet::{PacketHeader, Packet};

    macro_rules! register_packets {
            ($bytes:expr, $storages:expr, $id:expr, { $($packet_type:ident),* $(,)? }) => {
                register_packets!($bytes, $storages, $id, { $($packet_type => $packet_type),* });
            };
            ($bytes:expr, $storages:expr, $id:expr, { $($packet_type:ident => $packet_struct:ident),* $(,)? }) => {
                match PacketType::from_buffer($bytes) {
                    Some(ty) => match ty {
                        $(
                            PacketType::$packet_type => if let Some(packet) = $packet_struct::deserialize_unchecked($bytes) {
                                $storages.add_component($id, packet);
                            } else {
                                println!("{ty:?} data was malformed");
                            },
                        )*
                        _ => println!("Packet {:?} isn't registered", ty),
                    },
                    None => println!("Packet ID couldn't be determined"),
                }
            };
        }

    use crate::events::*;
    use crate::events::render_distance::*;

    register_packets!(buffer, storages, id, {
        ConnectionRequest,

        ChunkGenRequestEvent,

        RenderDistanceUpdateEvent,

        ClientInformationRequestEvent,
        ClientInformationUpdateEvent,

        ClientSettingsUpdateEvent,
        ClientPositionUpdate,
    });
}

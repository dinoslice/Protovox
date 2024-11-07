use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use bimap::BiHashMap;
use crossbeam::channel::{Receiver, Sender};
use laminar::{Socket, SocketEvent};
use shipyard::{AllStoragesViewMut, EntityId, Unique, UniqueViewMut, ViewMut};
use packet::PacketHeader;
use crate::environment::is_hosted;
use crate::events::{ClientInformationRequestEvent, ClientSettingsRequestEvent, ConnectionRequest, PacketType};
use crate::events::event_bus::EventBus;

#[derive(Unique)]
pub struct ServerHandler {
    pub tx: Sender<laminar::Packet>,
    pub rx: Receiver<SocketEvent>,
    pub clients: BiHashMap<SocketAddr, EntityId>,
}

impl ServerHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let config = laminar::Config {
            max_packet_size: 64 * 1024,
            max_fragments: 64,
            fragment_size: 1024,
            idle_connection_timeout: Duration::from_secs(6),
            .. Default::default()
        };

        let mut socket = Socket::bind_any_with_config(config)
            .expect("unable to bind to address");

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

pub fn server_process_network_events(mut storages: AllStoragesViewMut) {
    // TODO: fix insane battle with borrow checker
    loop {
        let server_handler = storages
            .borrow::<UniqueViewMut<ServerHandler>>()
            .expect("ServerHandler initialized");

        let res = server_handler.rx.try_recv();
        drop(server_handler);

        match res {
            Ok(evt) => match evt {
                SocketEvent::Packet(p) => {
                    let server_handler = storages
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
                SocketEvent::Timeout(addr) => {
                    tracing::debug!("timeout at {addr:?}")
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
            Err(_) => {
                break;
            }
        }
    }
}

fn add_packet(buffer: &[u8], id: EntityId, storages: &mut AllStoragesViewMut) {
    use crate::networking::types::PacketType;
    use packet::{PacketHeader, Packet};

    macro_rules! register_packets {
        ($bytes:expr, $storages:expr, $id:expr, { $($packet:ident => $($modifier:ident)|*),* $(,)? }) => {
            register_packets!($bytes, $storages, $id, { $($packet => $packet => $($modifier)|*),* });
        };
        ($bytes:expr, $storages:expr, $id:expr, { $($packet_type:ident => $packet_struct:ident => $($modifier:ident)|*),* $(,)? }) => {
            match PacketType::from_buffer($bytes) {
                Some(ty) => match ty {
                    $(
                        #[allow(unused_mut)]
                        PacketType::$packet_type => {
                            let mut decompress = false;
                            let mut use_bus = false;

                            $(
                                match stringify!( $modifier ) {
                                    "compressed" => decompress = true,
                                    "bus" => use_bus = true,
                                    _ => {}
                                }
                            )*

                            let res = match decompress {
                                true => $packet_struct::decompress_and_deserialize_unchecked($bytes),
                                false => $packet_struct::deserialize_unchecked($bytes),
                            };

                            match res {
                                None => tracing::error!("{ty:?} data was malformed"),
                                Some(data) => match use_bus {
                                    false => { $storages.add_component($id, data); }
                                    true => match storages.borrow::<ViewMut<EventBus<$packet_struct>>>() {
                                        Ok(mut vm_evt_bus) => vm_evt_bus.get_or_insert_with(id, Default::default).0.push(data),
                                        Err(_) => tracing::error!("Failed to borrow event bus storage"),
                                    }
                                }
                            };
                        }
                    )*
                    _ => tracing::debug!("Packet {:?} isn't registered", ty),
                },
                None => tracing::debug!("Packet ID couldn't be determined"),
            }
        };
    }

    use crate::events::*;
    use crate::events::render_distance::*;

    register_packets!(buffer, storages, id, {
        ConnectionRequest =>,

        ClientChunkRequest => bus,
        
        BlockUpdateEvent => bus,

        RenderDistanceUpdateEvent =>,

        ClientInformationRequestEvent =>,
        ClientInformationUpdateEvent =>,

        ClientSettingsUpdateEvent =>,
        ClientTransformUpdate =>,
    });
}

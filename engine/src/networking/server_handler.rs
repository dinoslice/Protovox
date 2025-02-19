use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
use bimap::BiHashMap;
use crossbeam::channel::{Receiver, Sender};
use laminar::{Socket, SocketEvent};
use shipyard::{AllStoragesViewMut, EntityId, Unique, UniqueView, UniqueViewMut, ViewMut};
use networking::{PacketRegistry, RuntimePacket};
use packet::PacketHeader;
use crate::components::{LocalPlayer, Transform};
use crate::events::{ClientInformationRequestEvent, ClientSettingsRequestEvent, ConnectionRequest, PacketType};
use crate::events::event_bus::EventBus;

#[derive(Unique)]
pub struct ServerHandler {
    pub tx: Sender<laminar::Packet>,
    pub rx: Receiver<SocketEvent>,
    pub local_addr: SocketAddr,
    pub clients: BiHashMap<SocketAddr, EntityId>,
}

impl ServerHandler {
    pub fn new(host_addr: Option<SocketAddr>) -> Self {
        let config = laminar::Config {
            max_packet_size: 64 * 1024,
            max_fragments: 64,
            fragment_size: 1024,
            idle_connection_timeout: Duration::from_secs(6),
            .. Default::default()
        };
        
        let mut socket = match host_addr {
            None => Socket::bind_any_with_config(config),
            Some(addr) => Socket::bind_with_config(addr, config),
        }
            .expect("unable to bind to address");
        
        let local_addr = socket.local_addr()
            .expect("failed to get local_addr?");
        
        tracing::debug!("Bound server to socket {local_addr:?}");
        let tx = socket.get_packet_sender();
        let rx = socket.get_event_receiver();
        
        let _polling = thread::spawn(move || socket.start_polling());

        Self {
            tx,
            rx,
            local_addr,
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

                            let conn_req_id = storages
                                .borrow::<UniqueView<PacketRegistry>>()
                                .expect("registry to be initialized")
                                .identifier_of::<ConnectionRequest>()
                                .expect("packet to be registered");

                            if Some(conn_req_id.untyped) != PacketRegistry::untyped_identifier_from(payload) {
                                tracing::warn!("First packet received from {:?} was not ConnectionRequest", addr);
                                continue;
                            }

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
                        Some(id) => {
                            if let Some(type_id) = PacketRegistry::untyped_identifier_from(payload) {
                                let opt = storages
                                    .borrow::<UniqueView<PacketRegistry>>()
                                    .expect("registry to be initialized")
                                    .deserializer_for_untyped_id(type_id);

                                if let Some((_, deserializer)) = opt {
                                    deserializer(payload, id, &mut storages)
                                        .expect("didn't fail");
                                }
                            }
                        }
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
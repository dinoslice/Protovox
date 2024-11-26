use laminar::Packet;
use shipyard::{AllStoragesView, Unique, UniqueView, UniqueViewMut, View, ViewMut};
use packet::Packet as _;
use crate::events::{ChatMessage, KeepAlive};
use crate::events::event_bus::EventBus;
use crate::networking::server_connection::ServerConnection;
use crate::networking::server_handler::ServerHandler;

#[derive(Unique, Default)]
pub struct ChatRecord {
    pub record: Vec<(String, String)>,
    pub unsent: Vec<String>
}

#[derive(Unique)]
pub struct CurrentChatInput(pub String);

pub fn initialize_chat_system(storages: AllStoragesView) {
    storages.add_unique(ChatRecord::default());
    storages.add_unique(CurrentChatInput("".to_string()));
}

pub fn client_send_chat_message(mut chat_record: UniqueViewMut<ChatRecord>, connection: UniqueViewMut<ServerConnection>) {
    let tx = &connection.tx;

    let mut processed = vec![];

    for message in chat_record.unsent.drain(..) {
        let addr = connection.server_addr;

        let keep_alive = Packet::unreliable(
            addr,
            ChatMessage(("A Person".to_string(), message.clone()))
                .serialize_packet()
                .expect("Packet Serialization Error")
        );

        if tx.send(keep_alive).is_err() {
            tracing::error!("There was an error sending chat packet to {addr:?}")
        }

        processed.push(("Me".to_string(), message));
    }

    chat_record.record.append(&mut processed);
}

pub fn server_send_chat_message(mut chat_record: UniqueViewMut<ChatRecord>, server_handler: UniqueView<ServerHandler>) {
    let tx = &server_handler.tx;
    
    let mut processed = vec![];
    
    for message in chat_record.unsent.drain(..) {
        for &addr in server_handler.clients.left_values() {
            let keep_alive = Packet::unreliable(
                addr,
                ChatMessage(("A Person".to_string(), message.clone()))
                    .serialize_packet()
                    .expect("Packet Serialization Error")
            );

            if tx.send(keep_alive).is_err() {
                tracing::error!("There was an error sending chat packet to {addr:?}")
            }
        }
        
        processed.push(("Me".to_string(), message));
    }
    
    chat_record.record.append(&mut processed);
}

pub fn client_handle_chat_messages(mut chat_record: UniqueViewMut<ChatRecord>, mut vm_messages: ViewMut<ChatMessage>) {
    for (sender, message) in vm_messages.drain().map(|msg| msg.0) {
        chat_record.record.push((sender, message));
    }
}

pub fn server_handle_chat_messages(mut chat_record: UniqueViewMut<ChatRecord>, mut vm_messages: ViewMut<EventBus<ChatMessage>>, server_handler: UniqueView<ServerHandler>) {
    let tx = &server_handler.tx;

    for (id, messages) in vm_messages.drain().into_iter().with_id() {
        for (sender, message) in messages.0.into_iter().map(|m| m.0) {
            for client in server_handler.clients.right_values() {
                if client == &id {
                    continue;
                }

                let addr = *server_handler.clients.get_by_right(client)
                    .expect("Client should exist");

                let keep_alive = Packet::unreliable(
                    addr,
                    ChatMessage((sender.clone(), message.clone()))
                        .serialize_packet()
                        .expect("Packet Serialization Error")
                );

                if tx.send(keep_alive).is_err() {
                    tracing::error!("There was an error sending chat packet to {addr:?}")
                }
            }

            chat_record.record.push((sender, message));
        }
    }
}
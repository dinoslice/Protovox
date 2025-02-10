pub mod render_distance;
pub mod event_bus;

use serde::{Deserialize, Serialize};
use shipyard::Component;
use packet_derive::Packet;
use packet::Packet;
use resources::ResourceKey;
use crate::base_types::block::Block;
use crate::components::Transform;
use crate::game::chunk::data::ChunkData;
use crate::game::chunk::location::ChunkLocation;
use crate::game::location::{BlockLocation, WorldLocation};
pub use crate::networking::types::PacketType;
use crate::render_distance::RenderDistance;

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ChunkGenRequestEvent)]
pub struct ChunkGenRequestEvent(pub ChunkLocation);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ChunkGenEvent)]
#[repr(transparent)]
pub struct ChunkGenEvent(pub ChunkData);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::BlockUpdateEvent)]
pub struct BlockUpdateEvent(pub BlockLocation, pub ResourceKey<Block>);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ClientInformationRequestEvent)]
pub struct ClientInformationRequestEvent;

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ClientInformationUpdateEvent)]
pub struct ClientInformationUpdateEvent(pub WorldLocation);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ClientSettingsRequestEvent)]
pub struct ClientSettingsRequestEvent;

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ClientSettingsUpdateEvent)]
pub struct ClientSettingsUpdateEvent(pub RenderDistance);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ConnectionRequest)]
pub struct ConnectionRequest;

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ConnectionSuccess)]
pub struct ConnectionSuccess;

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ClientTransformUpdate)]
pub struct ClientTransformUpdate(pub Transform);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::ClientChunkRequest)]
pub struct ClientChunkRequest(pub ChunkLocation);

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::KeepAlive)]
pub struct KeepAlive;

#[derive(Debug, Component, Packet, Serialize, Deserialize)]
#[packet_type(PacketType::KickedByServer)]
pub struct KickedByServer(pub String);

impl From<ChunkGenRequestEvent> for ClientChunkRequest {
    fn from(evt: ChunkGenRequestEvent) -> Self {
        Self(evt.0)
    }
}

impl From<ClientChunkRequest> for ChunkGenRequestEvent {
    fn from(evt: ClientChunkRequest) -> Self {
        Self(evt.0)
    }
}
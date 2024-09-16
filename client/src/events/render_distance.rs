use serde::{Deserialize, Serialize};
use shipyard::Component;
use packet::Packet;
use packet_derive::Packet;
pub use crate::networking::types::PacketType;
use crate::render_distance::RenderDistance;

#[derive(Packet, Component, Debug, Serialize, Deserialize)]
#[packet_type(PacketType::RenderDistanceRequestEvent)]
pub struct RenderDistanceRequestEvent;

#[derive(Packet, Component, Debug, Serialize, Deserialize)]
#[packet_type(PacketType::RenderDistanceUpdateEvent)]
pub struct RenderDistanceUpdateEvent(pub RenderDistance);

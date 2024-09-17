use serde::{Deserialize, Serialize};
use strum::FromRepr;
use packet::PacketHeader;

#[repr(u16)]
#[derive(Debug, Serialize, Deserialize, FromRepr, Copy, Clone)]
pub enum PacketType {
    ConnectionRequest,
    ConnectionSuccess,

    ChunkGenRequestEvent,
    ChunkGenEvent,

    RenderDistanceRequestEvent,
    RenderDistanceUpdateEvent,

    ClientInformationRequestEvent,
    ClientInformationUpdateEvent,

    ClientSettingsRequestEvent,
    ClientSettingsUpdateEvent,
}

impl PacketHeader for PacketType {
    fn repr(&self) -> u16 {
        *self as _
    }

    #[inline]
    fn from_header_repr(repr: u16) -> Option<Self> {
        Self::from_repr(repr)
    }
}
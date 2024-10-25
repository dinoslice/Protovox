use serde::{Deserialize, Serialize};
use strum::FromRepr;
use packet::PacketHeader;

#[repr(u16)]
#[derive(Debug, Serialize, Deserialize, FromRepr, Copy, Clone, Eq, PartialEq)]
pub enum PacketType {
    ConnectionRequest,
    ConnectionSuccess,

    ClientChunkRequest,
    ChunkGenRequestEvent,
    ChunkGenEvent,

    RenderDistanceRequestEvent,
    RenderDistanceUpdateEvent,

    ClientInformationRequestEvent,
    ClientInformationUpdateEvent,

    ClientSettingsRequestEvent,
    ClientSettingsUpdateEvent,

    ClientTransformUpdate,

    KeepAlive,
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
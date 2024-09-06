use std::mem;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::{PacketDeserializationError, PacketHeader};

pub trait Packet<H: PacketHeader> {
    const TYPE: H;

    fn serialize_packet(&self) -> Option<Vec<u8>> where Self: Serialize {
        let id = Self::TYPE.repr();

        let buffer = id.to_le_bytes().to_vec();

        postcard::to_extend(self, buffer).ok()
    }

    fn deserialize_checked(bytes: &[u8]) -> Result<Self, PacketDeserializationError> where Self: DeserializeOwned {
        let id_bytes = bytes.get(..mem::size_of::<u16>())
            .ok_or(PacketDeserializationError::MalformedData)?;

        let id = u16::from_le_bytes(id_bytes.try_into().expect("must be 2 bytes"));

        if id != Self::TYPE.repr() {
            return Err(PacketDeserializationError::WrongType);
        }

        Self::deserialize_unchecked(bytes)
            .ok_or(PacketDeserializationError::DeserializationError)
    }

    fn deserialize_unchecked(bytes: &[u8]) -> Option<Self> where Self: DeserializeOwned {
        postcard::from_bytes(bytes.get(2..)?).ok()
    }
}

pub fn get_id(buffer: &[u8]) -> Option<u16> {
    let first_two_bytes: [u8; 2] = buffer.get(..2)?
        .try_into()
        .ok()?;

    Some(u16::from_le_bytes(first_two_bytes))
}
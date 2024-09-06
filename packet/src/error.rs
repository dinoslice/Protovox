#[derive(Debug, thiserror::Error, PartialEq, Eq, Hash, Clone)]
pub enum PacketDeserializationError {
    #[error("The data wasn't properly formatted to construct a packet.")]
    MalformedData,
    #[error("The data type didn't match the expected packet type.")]
    WrongType,
    #[error("There was an error deserializing the packet from bytes.")]
    DeserializationError,
}
pub trait PacketHeader: Sized {
    fn repr(&self) -> u16;

    fn get_id(buffer: &[u8]) -> Option<Self> {
        let first_two_bytes: [u8; 2] = buffer.get(..2)?
            .try_into()
            .ok()?;

        let id = u16::from_le_bytes(first_two_bytes);

        Self::from_header_repr(id)
    }

    fn from_header_repr(repr: u16) -> Option<Self>;

    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        let first_two_bytes: [u8; 2] = buffer.get(..2)?
            .try_into()
            .ok()?;

        let id = u16::from_le_bytes(first_two_bytes);

        Self::from_header_repr(id)
    }
}
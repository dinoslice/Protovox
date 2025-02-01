use std::marker::PhantomData;
use std::num::NonZeroU16;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct PacketIdentifier<T: ?Sized> {
    pub untyped: UntypedPacketIdentifier,
    ty: PhantomData<T>,
}

impl<T> Clone for PacketIdentifier<T> {
    fn clone(&self) -> Self {
        Self {
            untyped: self.untyped.clone(),
            ty: Default::default(),
        }
    }
}

impl<T> Copy for PacketIdentifier<T> {}

#[derive(Debug, Clone, Hash, Copy, Eq, PartialEq)]
pub struct UntypedPacketIdentifier(pub(crate) NonZeroU16);

impl UntypedPacketIdentifier {
    pub(crate) fn add_type<T>(self) -> PacketIdentifier<T>  {
        PacketIdentifier {
            untyped: self,
            ty: PhantomData::default(),
        }
    }
}
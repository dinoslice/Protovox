use std::marker::PhantomData;
use std::num::NonZeroU16;

pub struct PacketIdentifier<T> {
    id: NonZeroU16,
    ty: PhantomData<T>,
}
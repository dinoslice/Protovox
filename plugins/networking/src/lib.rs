mod type_id;

use std::any;
use std::any::TypeId;
use std::io::Read;
use std::num::NonZeroU16;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use hashbrown::HashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shipyard::{AllStorages, Component, EntityId, Unique, ViewMut};

type DeserializerClient = fn(&[u8], &mut AllStorages) -> Option<EntityId>;
type DeserializerServer = fn(&[u8], EntityId, &mut AllStorages) -> Option<()>; // TODO: errors

pub use type_id::{PacketIdentifier, UntypedPacketIdentifier};

// TODO(urgent): need to make sure identifiers are synced between client and server?
// packets are order dependent? maybe send over hash? reserve Id of 1 for this packet?
// if not synced, try to fix or kick player if not possible
#[derive(Debug, Default, Clone, Unique)]
pub struct PacketRegistry { // TODO: only needs one of DeserializerClient or DeserializerServer
    map: Vec<(DeserializerClient, DeserializerServer)>, // TypeIdentifier -> Deserializer
    ids: HashMap<TypeId, UntypedPacketIdentifier>,
}

impl PacketRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: custom error
    // returns whether the packet is already registered
    pub fn register<T: RuntimePacket + DeserializeOwned + Component, const C: bool, const B: bool>(&mut self) -> bool {
        assert_eq!(size_of::<UntypedPacketIdentifier>(), size_of::<u16>());
        assert_eq!(size_of::<UntypedPacketIdentifier>(), size_of::<PacketIdentifier<T>>());

        let type_id = TypeId::of::<T>();

        if self.ids.contains_key(&type_id) {
            return true;
        }

        let deserializer_client: DeserializerClient = |bytes, storages| {
            Some(storages.add_entity(<T as RuntimePacket>::deserialize::<C>(bytes)?))
        };

        let deserializer_server: DeserializerServer = if B {
            |bytes, id, storages| {
                match storages.borrow::<ViewMut<EventBus<T>>>() {
                    Ok(mut vm_evt_bus) => match vm_evt_bus.get_or_insert_with(id, Default::default) {
                        Some(mut bus) => bus.0.push(<T as RuntimePacket>::deserialize::<C>(bytes)?),
                        None => tracing::error!("Tried to insert {:?} event to {id:?}, but it was dead", any::type_name::<T>()),
                    },
                    Err(_) => tracing::error!("Failed to borrow event bus storage"),
                }
                Some(())
            }
        } else {
            |bytes, id, storages| {
                storages.add_component(id, <T as RuntimePacket>::deserialize::<C>(bytes)?);
                Some(())
            }
        };

        self.map.push((deserializer_client, deserializer_server));

        let id = NonZeroU16::new(self.map.len() as _)
            .expect("shouldn't be nonzero");

        let prev = self.ids.insert(type_id, UntypedPacketIdentifier(id));

        debug_assert_eq!(prev, None, "if there was something previously here, it should've been caught earlier by the guard if statement");

        false
    }

    pub fn identifier_of<T: 'static>(&self) -> Option<PacketIdentifier<T>> {
        self.ids
            .get(&TypeId::of::<T>())
            .copied()
            .map(UntypedPacketIdentifier::add_type)
    }

    pub fn deserializer_for_untyped_id(&self, type_id: UntypedPacketIdentifier) -> Option<(DeserializerClient, DeserializerServer)> {
        self.map
            .get(type_id.0.get() as usize - 1)
            .copied()
    }

    pub fn deserializer_for_ty<T: 'static>(&self) -> Option<(DeserializerClient, DeserializerServer)> {
        self.deserializer_for_untyped_id(self.identifier_of::<T>()?.untyped)
    }

    pub fn untyped_identifier_from(bytes: &[u8]) -> Option<UntypedPacketIdentifier> {
        let first_two_bytes = bytes.get(..size_of::<UntypedPacketIdentifier>())?
            .try_into()
            .ok()?;

        let inner = u16::from_le_bytes(first_two_bytes)
            .try_into()
            .ok()?;

        Some(UntypedPacketIdentifier(inner))
    }
}

pub trait RuntimePacket {
    fn deserialize<const C: bool>(bytes: &[u8]) -> Option<Self> where Self: DeserializeOwned {
        const ID_SIZE: usize = size_of::<UntypedPacketIdentifier>();

        if C {
            let mut z = ZlibDecoder::new(bytes.get(ID_SIZE..)?);
            let mut bytes = Vec::new();
            z.read_to_end(&mut bytes).ok()?;

            postcard::from_bytes(&bytes).ok()
        } else {
            postcard::from_bytes(bytes.get(ID_SIZE..)?).ok()
        }
    }

    fn serialize_uncompressed_with_id(&self, id: PacketIdentifier<Self>) -> Option<Vec<u8>> where Self: Serialize {
        self.serialize_with_id::<false>(id)
    }

    fn serialize_with_id<const C: bool>(&self, id: PacketIdentifier<Self>) -> Option<Vec<u8>> where Self: Serialize {
        let mut buffer = id.untyped.0.get().to_le_bytes().to_vec();

        if C {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            postcard::to_io(self, &mut encoder).ok()?;

            buffer.append(&mut encoder.finish().ok()?);

            Some(buffer)
        } else {
            postcard::to_extend(self, buffer).ok()
        }
    }
}


// TODO: remove this impl once we fix it
impl<T: Serialize + DeserializeOwned> RuntimePacket for T {}

#[derive(Clone, Debug, Component)]
pub struct EventBus<T: Sync + Send + 'static>(pub Vec<T>);

impl<T: Sync + Send + 'static> Default for EventBus<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}
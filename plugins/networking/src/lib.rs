use std::any;
use std::any::TypeId;
use std::io::Read;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use hashbrown::HashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shipyard::{AllStorages, Component, EntityId, Unique, ViewMut};

type DeserializerClient = fn(&[u8], &mut AllStorages) -> Option<EntityId>;
type DeserializerServer = fn(&[u8], EntityId, &mut AllStorages) -> Option<()>; // TODO: errors

pub type TypeIdentifier = u16; // TODO: switch to NonZeroU16 for niche value optimization, wrapper type to eliminate incorrect usage, rename *Header
// TODO: add phantom type to public api to force correctness

#[derive(Debug, Default, Clone, Unique)]
pub struct PacketRegistry { // TODO: only needs one of DeserializerClient or DeserializerServer
    map: Vec<(DeserializerClient, DeserializerServer)>, // TypeIdentifier -> Deserializer
    ids: HashMap<TypeId, TypeIdentifier>,
}

impl PacketRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: custom error
    // returns whether the packet is already registered
    pub fn register<T: RuntimePacket + DeserializeOwned + Component, const C: bool, const B: bool>(&mut self) -> bool {
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


        let id = self.map.len();

        self.map.push((deserializer_client, deserializer_server));

        let prev = self.ids.insert(type_id, id as _);

        debug_assert_eq!(prev, None, "if there was something previously here, it should've been caught earlier by the guard if statement");

        false
    }

    pub fn identifier_of<T: 'static>(&self) -> Option<TypeIdentifier> {
        self.ids
            .get(&TypeId::of::<T>())
            .copied()
    }

    pub fn deserializer_for_id(&self, type_id: TypeIdentifier) -> Option<(DeserializerClient, DeserializerServer)> {
        self.map
            .get(type_id as usize)
            .copied()
    }

    pub fn deserializer_for_ty<T: 'static>(&self) -> Option<(DeserializerClient, DeserializerServer)> {
        self.deserializer_for_id(self.identifier_of::<T>()?)
    }

    pub fn client_consume_packet_into(&self, bytes: &[u8], storages: &mut AllStorages) -> Option<EntityId> { // TODO: type state
        let id = Self::identifier_from(bytes)?; // err: malformed data

        let deserializer= self.map.get(id as usize)?.0; // err: unregistered type

        deserializer(bytes, storages) // err: deserialization error
    }

    pub fn server_consume_packet_into(&self, bytes: &[u8], entity_id: EntityId, storages: &mut AllStorages) -> Option<()> {
        let id = Self::identifier_from(bytes)?; // err: malformed data

        let deserializer= self.map.get(id as usize)?.1; // err: unregistered type

        deserializer(bytes, entity_id, storages) // err: deserialization error
    }

    pub fn identifier_from(bytes: &[u8]) -> Option<TypeIdentifier> {
        let first_two_bytes = bytes.get(..size_of::<TypeIdentifier>())?
            .try_into()
            .ok()?;

        Some(u16::from_le_bytes(first_two_bytes))
    }
}

pub trait RuntimePacket {
    fn deserialize<const C: bool>(bytes: &[u8]) -> Option<Self> where Self: DeserializeOwned {
        const ID_SIZE: usize = size_of::<TypeIdentifier>();

        if C {
            let mut z = ZlibDecoder::new(bytes.get(ID_SIZE..)?);
            let mut bytes = Vec::new();
            z.read_to_end(&mut bytes).ok()?;

            postcard::from_bytes(&bytes).ok()
        } else {
            postcard::from_bytes(bytes.get(ID_SIZE..)?).ok()
        }
    }

    fn serialize_uncompressed_with_id(&self, id: TypeIdentifier) -> Option<Vec<u8>> where Self: Serialize {
        self.serialize_with_id::<false>(id)
    }

    fn serialize_with_id<const C: bool>(&self, id: TypeIdentifier) -> Option<Vec<u8>> where Self: Serialize {
        let mut buffer = id.to_le_bytes().to_vec();

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
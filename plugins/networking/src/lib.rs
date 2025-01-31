use std::any::TypeId;
use std::io::Read;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use hashbrown::HashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use shipyard::{AllStoragesViewMut, Component, EntityId};
type Deserializer = fn(&[u8], AllStoragesViewMut) -> Option<EntityId>;

type TypeIdentifier = u16; // TODO: switch to NonZeroU16 for niche value optimization, wrapper type to eliminate incorrect usage, rename *Header
// TODO: add phantom type to public api to force correctness

#[derive(Debug, Default, Clone)]
pub struct PacketRegistry {
    map: Vec<Deserializer>, // TypeIdentifier -> Deserializer
    ids: HashMap<TypeId, TypeIdentifier>,
}

impl PacketRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: custom error
    // returns whether the packet is already registered
    pub fn register<T: RuntimePacket + DeserializeOwned + Component, const C: bool>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();

        if self.ids.contains_key(&type_id) {
            return true;
        }

        let deserializer = |bytes: &[u8], mut storages: AllStoragesViewMut| {
            Some(storages.add_entity(<T as RuntimePacket>::deserialize::<C>(bytes)?))
        };

        let id = self.map.len();

        self.map.push(deserializer);

        let prev = self.ids.insert(type_id, id as _);

        debug_assert_eq!(prev, None, "if there was something previously here, it should've been caught earlier by the guard if statement");

        false
    }

    pub fn identifier_of<T: 'static>(&self) -> Option<TypeIdentifier> {
        self.ids
            .get(&TypeId::of::<T>())
            .copied()
    }

    pub fn deserializer_for<T: 'static>(&self) -> Option<Deserializer> {
        self.map
            .get(self.identifier_of::<T>()? as usize)
            .copied()
    }

    pub fn consume_packet_into(&self, bytes: &[u8], storages: AllStoragesViewMut) -> Option<EntityId> {
        let id = Self::identifier_from(bytes)?; // err: malformed data

        let deserializer= self.map.get(id as usize)?; // err: unregistered type

        deserializer(bytes, storages) // err: deserialization error
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

    fn serialize<const C: bool>(&self, id: TypeIdentifier) -> Option<Vec<u8>> where Self: Serialize {
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
use std::any::{Any, TypeId};
use std::collections::HashMap;
use shipyard::Unique;
use crate::key::ResourceKey;
use crate::resource_type::ResourceType;

#[derive(Default, Unique)]
pub struct Registry {
    storage: HashMap<TypeId, HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl Registry {
    pub fn register<T: ResourceType + 'static>(&mut self, key: ResourceKey<T>, data: T) {
        if !data.is_valid(self) {
            return;
        }

        let inner = self.storage.entry(TypeId::of::<T>()).or_default();
        inner.insert(key.to_string(), Box::new(data));
    }
    
    pub fn get<T: ResourceType + 'static>(&self, key: &ResourceKey<T>) -> Option<&T> {
        self.storage.get(&TypeId::of::<T>())?
            .get(&key.to_string())?
            .downcast_ref::<T>()
    }
    
    pub fn get_unchecked<T: ResourceType + 'static>(&self, key: &ResourceKey<T>) -> &T {
        self.get(key).unwrap_or_else(|| panic!("No data existed in the registry for the ResourceKey '{}'", key))
    }

    pub fn get_mut<T: ResourceType + 'static>(&mut self, key: &ResourceKey<T>) -> Option<&mut T> {
        self.storage.get_mut(&TypeId::of::<T>())?
            .get_mut(&key.to_string())?
            .downcast_mut::<T>()
    }

    pub fn get_mut_unchecked<T: ResourceType + 'static>(&mut self, key: &ResourceKey<T>) -> &mut T {
        self.get_mut(key).unwrap_or_else(|| panic!("No data existed in the registry for the ResourceKey '{}'", key))
    }

    pub fn iter<T: ResourceType + 'static>(&self) -> impl Iterator<Item=(ResourceKey<T>, &T)> {
        self.storage
            .get(&TypeId::of::<T>())
            .unwrap()
            .iter()
            .map(|(k, v)| (k.try_into().unwrap(), v.downcast_ref::<T>().unwrap()))
    }

    pub fn iter_mut<T: ResourceType + 'static>(&mut self) -> impl Iterator<Item = (ResourceKey<T>, &mut T)> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .iter_mut()
            .map(|(k, v)| (k.try_into().unwrap(), v.downcast_mut::<T>().unwrap()))
    }
}

#[derive(Default, Clone)]
struct Block {
    id: usize,
    name: String,
}


impl ResourceType for Block {
    fn resource_name() -> &'static str {
        "block"
    }

    fn default_resource() -> ResourceKey<Self>
    where
        Self: Sized,
    {
        ResourceKey::new("engine", "cobblestone")
    }
}

pub fn test() {
    let mut registry = Registry::default();
    registry.register(ResourceKey::new("test", "test_block"), Block { id: 0, name: "test_block".to_string() });
    for (key, data) in registry.iter::<Block>() {
        println!("key: {}, data: {} {}", key, data.id, data.name);
    }

    for (_, data) in registry.iter_mut::<Block>() {
        data.id = 1;
        data.name = "test_block_test2".to_string();
    }

    for (key, data) in registry.iter::<Block>() {
        println!("key: {}, data: {} {}", key, data.id, data.name);
    }
}
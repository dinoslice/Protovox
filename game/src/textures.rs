use std::hash::{Hash, Hasher};
use hashbrown::HashMap;

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct TextureKey(u64);

impl Hash for TextureKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0)
    }
}

impl TextureKey {
    pub const fn new(path: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(path))
    }
    
    pub const fn was_from(&self, path: &str) -> bool {
        self.0 == Self::new(path).0
    }
}

impl nohash::IsEnabled for TextureKey {}

pub struct TextureRegistrar(HashMap<TextureKey, &'static str>);

impl TextureRegistrar {
    pub fn register(&mut self, key: TextureKey, path: &'static str) {
        self.try_register(key, path)
            .expect("path didn't match texture key")
    }
    
    pub fn try_register(&mut self, key: TextureKey, path: &'static str) -> Option<()> {
        let try_key = TextureKey::new(path);
        
        if try_key == key {
            self.0.insert(key, path);
            Some(())
        } else {
            None
        }
    }
    
    pub fn into_raw(self) -> HashMap<TextureKey, &'static str> {
        self.0
    }
}

pub struct TextureLookup(HashMap<TextureKey, u16>);
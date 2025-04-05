use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use shipyard::Unique;
use game::chunk::data::ChunkData;
use game::chunk::location::ChunkLocation;

#[derive(Unique)]
pub struct WorldSaver {
    default_cache_time: Duration,
    cache: HashMap<ChunkLocation, (Instant, ChunkSaveCache)>,
    saver: Box<dyn ChunkSaver + Send + Sync + 'static>,
}

impl WorldSaver {
    pub fn new(default_cache_time: Duration, saver: impl ChunkSaver + Send + Sync + 'static) -> Self {
        Self {
            default_cache_time,
            cache: HashMap::default(),
            saver: Box::new(saver),
        }
    }
    
    pub fn cache_with_duration(&mut self, loc: ChunkLocation, data: ChunkData, duration: Duration) {
        let expired_at = Instant::now() + duration;
        
        let chunk_save_cache = ChunkSaveCache { data };
        
        let prev = self.cache.insert(loc, (expired_at, chunk_save_cache));
        
        if let Some((expiration, data)) = prev {
            let loc = data.data.location;
            
            let time_rem = Instant::now() - expiration;
            
            tracing::warn!("Tried to save Chunk at {loc:?}, which would expire in {time_rem:?}");
        }
    }
    
    pub fn cache(&mut self, loc: ChunkLocation, data: ChunkData) {
        self.cache_with_duration(loc, data, self.default_cache_time)
    }
    
    pub fn process(&mut self) {
        for (_, (_, cache)) in self.cache.extract_if(|_, (time, _)| Instant::now() >= *time) {
            self.saver.save(cache);
        }
    }
    
    pub fn save_all(&mut self) {
        for (_, (_, cache)) in self.cache.drain() {
            self.saver.save(cache);
        }
    }
    
    pub fn try_remove(&mut self, loc: &ChunkLocation) -> Option<ChunkSaveCache> {
        self.cache.remove(loc).map(|v| v.1)
    }
}

impl Default for WorldSaver {
    fn default() -> Self {
        Self::new(Duration::from_secs(45), ChunkSaveToFile::new("./world/").expect("path is a directory"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSaveCache {
    pub data: ChunkData,
}

pub trait ChunkSaver {
    fn save(&self, data: ChunkSaveCache) -> bool;
}

pub struct ChunkSaveToFile {
    path: PathBuf,
}

impl ChunkSaveToFile {
    pub fn new(path: impl Into<PathBuf>) -> Option<Self> {
        let path = path.into();
        
        path.is_dir().then_some(Self { path })
    }
}

impl ChunkSaver for ChunkSaveToFile {
    fn save(&self, data: ChunkSaveCache) -> bool {
        let loc = &data.data.location.0;

        let file_name = format!("{}_{}_{}.cff", loc.x, loc.y, loc.z);

        let save_path = self.path.join(&file_name);

        let bytes = match postcard::to_allocvec(&data.data) {
            Ok(bytes) => bytes, 
            Err(err) => {
                tracing::error!("Failed to serialize chunk at {:?}: {err}", &data.data.location);
                return false;
            }
        };

        match fs::write(&save_path, &bytes) {
            Ok(_) => true,
            Err(err) => {
                tracing::error!("Failed to create and write to file at {save_path:?}: {err}");
                false
            }
        }
    }
}
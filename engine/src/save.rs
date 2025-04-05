use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use glm::IVec3;
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use shipyard::Unique;
use game::chunk::data::ChunkData;
use game::chunk::location::ChunkLocation;

#[derive(Unique)]
pub struct WorldSaver {
    default_cache_time: Duration,
    cache: HashMap<ChunkLocation, (Instant, ChunkSaveCache)>,
    saved: HashSet<ChunkLocation>,
    saver: Box<dyn ChunkSaver + Send + Sync + 'static>,
}

impl WorldSaver {
    pub fn new(default_cache_time: Duration, saver: impl ChunkSaver + Send + Sync + 'static) -> Self {
        let mut saved = HashSet::default();
        
        let saver = Box::new(saver);
        
        saver.update_saved(&mut saved);
        
        Self {
            default_cache_time,
            cache: HashMap::default(),
            saved,
            saver,
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
        for (loc, (_, cache)) in self.cache.extract_if(|_, (time, _)| Instant::now() >= *time) {
            self.saver.save(cache);
            self.saved.insert(loc);
        }
    }
    
    pub fn save_all(&mut self) {
        for (loc, (_, cache)) in self.cache.drain() {
            self.saver.save(cache);
            self.saved.insert(loc);
        }
    }
    
    pub fn try_get(&mut self, loc: &ChunkLocation) -> Option<ChunkSaveCache> {
        if let Some(cache) = self.cache.remove(loc).map(|v| v.1) {
            Some(cache)
        } else {
            self.get_from_saved(loc)
        }
    }
    
    fn get_from_saved(&mut self, loc: &ChunkLocation) -> Option<ChunkSaveCache> {
        if self.saved.contains(loc) {
            let opt = self.saver.retrieve(loc);
            
            if opt.is_none() {
                self.saved.remove(loc);
            }
            
            opt
        } else {
            None
        }
    }
}

impl Default for WorldSaver {
    fn default() -> Self {
        Self::new(Duration::from_secs(45), ChunkSaveToFile::new("target/save/world/").expect("path is a directory"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSaveCache {
    pub data: ChunkData,
}

impl ChunkSaveCache {
    pub fn new(data: ChunkData) -> Self {
        Self { data }
    }
}

pub trait ChunkSaver {
    fn save(&self, data: ChunkSaveCache) -> bool;
    
    fn retrieve(&self, loc: &ChunkLocation) -> Option<ChunkSaveCache>;

    fn update_saved(&self, saved: &mut HashSet<ChunkLocation>);
}

pub struct ChunkSaveToFile {
    path: PathBuf,
}

impl ChunkSaveToFile {
    pub fn new(path: impl Into<PathBuf>) -> Option<Self> {
        let path = path.into();
        
        // TODO: check to make sure this usage is right
        match fs::create_dir_all(&path) {
            Ok(_) => { Some(Self { path }) }
            Err(err) => {
                tracing::error!("failed to create dir at {path:?}: {err}");
                None
            }
        }
    }

    fn loc_to_file_name(loc: &ChunkLocation) -> PathBuf {
        PathBuf::from(format!("{}_{}_{}.cff", loc.0.x, loc.0.y, loc.0.z))
    }

    fn file_name_to_loc(file: &Path) -> Option<ChunkLocation> {
        if file.extension().is_none_or(|ext| ext != "cff") {
            tracing::warn!("extension was incorrect; TODO: errors");
            return None;
        }
        
        let Some(stem) = file.file_stem() else {
            tracing::warn!("no file name; TODO: errors");
            return None;
        };
        
        let stem = stem.to_string_lossy();
        
        let mut components = stem.split('_');
        
        let (Some((x, y, z)), None) = (components.next_tuple(), components.next()) else {
            tracing::warn!("invalid name; TODO: errors");
            return None;
        };
        
        let (Ok(x), Ok(y), Ok(z)) = (x.parse(), y.parse(), z.parse()) else {
            tracing::warn!("failed to parse coordinates; TODO: errors");
            return None;
        };
        
        let loc = ChunkLocation(IVec3::new(x, y, z));
        
        // performance cost here is negligible since this function is only called on startup
        assert_eq!(Self::loc_to_file_name(&loc).file_name(), file.file_name(), "not inverse operations");
        
        Some(loc)
    }
}

impl ChunkSaver for ChunkSaveToFile {
    fn save(&self, data: ChunkSaveCache) -> bool {
        let save_path = self.path.join(Self::loc_to_file_name(&data.data.location));

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

    fn retrieve(&self, loc: &ChunkLocation) -> Option<ChunkSaveCache> {
        let saved_path = self.path.join(Self::loc_to_file_name(&loc));
        
        let Ok(bytes) = fs::read(&saved_path) else {
            tracing::error!("failed to open & read chunk {loc:?} at {saved_path:?}");
            return None;
        };
        
        let bytes = match fs::read(&saved_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::error!("failed to open & read chunk {loc:?} at {saved_path:?}: {err}");
                return None;
            }
        };

        match postcard::from_bytes(&bytes) {
            Ok(bytes) => Some(bytes),
            Err(err) => {
                tracing::error!("failed to deserialize {loc:?} at {saved_path:?}: {err}");
                None
            }
        }
    }

    fn update_saved(&self, saved: &mut HashSet<ChunkLocation>) {
        let Ok(read_dir) = fs::read_dir(&self.path) else {
            tracing::warn!("invalid directory; TODO: errors");
            return;
        };
        
        for entry in read_dir {
            let Ok(entry) = entry else {
                tracing::warn!("unable to get dir entry");
                continue;
            };
            
            let path = entry.path();
            
            if let Some(location) = Self::file_name_to_loc(&path) {
                saved.insert(location);
            } else {
                tracing::warn!("file \"{path:?}\" wasn't a chunk save file");
            }
        }
    }
}
use std::time::Duration;
use glm::{IVec3, U16Vec3};
use game::chunk::data::ChunkData;
use hashbrown::HashMap;
use shipyard::Unique;
use tracing::trace;
use game::chunk::location::ChunkLocation;

const REQ_TIMEOUT: f32 = 5.0;

#[derive(Unique)]
pub struct ChunkManager {
    // TODO: add handle to gpu buffer?
    loaded_chunks: Vec<Option<ChunkData>>,
    // TODO: eventually change to unsigned type, rn doing this to eliminate casting
    render_distance: IVec3,

    recently_requested: HashMap<ChunkLocation, f32>,

    // TODO: should we store this?
    center: ChunkLocation,
}

impl ChunkManager {
    pub fn new(render_distance: &U16Vec3, center: ChunkLocation) -> Self {
        let size = render_distance.iter()
            .map(|n| (2 * n + 1) as usize)
            .product();

        tracing::debug!("attempting to allocate size for {size} chunks.");

        let mut loaded_chunks = Vec::new();
        loaded_chunks.resize_with(size, || None);

        let render_distance = render_distance.cast();

        Self {
            loaded_chunks,
            render_distance,
            recently_requested: HashMap::default(),
            center,
        }
    }

    pub fn chunk_capacity(&self) -> usize {
        self.render_distance.iter()
            .map(|n| (2 * n + 1) as usize)
            .product()
    }

    pub fn is_chunk_loc_in_render_distance(center: &ChunkLocation, render_distance: &IVec3, chunk: &ChunkLocation) -> bool {
        let location = chunk.0;
        let center = center.0;

        let min = center - render_distance;
        let max = center + render_distance;

        location.iter()
            .enumerate()
            .all(|(a, n)|
                (min[a]..=max[a]).contains(n)
            )
    }

    pub fn get_index_from_chunk_loc(&self, location: &ChunkLocation) -> usize {
        self.get_index_from_offset(&(location.0 - self.center.0))
    }

    pub fn get_index_from_offset(&self, offset: &IVec3) -> usize {
        let norm_offset = offset + self.render_distance;

        assert!(norm_offset.iter().all(|n| !n.is_negative()));

        into_1d_coordinate(&norm_offset, &self.render_size()) as usize
    }

    pub fn get_index_from_chunk_location_checked(&self, location: &ChunkLocation) -> Option<usize> {
        let offset = location.0 - self.center.0;

        let norm_offset = offset + self.render_distance;

        if norm_offset.iter()
            .enumerate()
            .any(|(i, n)| *n > 2 * self.render_distance[i] || n.is_negative())
        {
            return None;
        }

        let index = into_1d_coordinate(&norm_offset, &self.render_size()) as usize;

        Some(index)
    }

    pub fn get_location_from_index(&self, index: usize) -> ChunkLocation {
        let norm_offset = into_3d_coordinate(index as _, &self.render_size());

        let offset = norm_offset - self.render_distance;

        let chunk_loc = offset + self.center.0;

        ChunkLocation(chunk_loc)
    }

    pub fn get_chunk_ref_from_offset(&self, offset: &IVec3) -> &ChunkData {
        let offset = self.get_index_from_offset(offset);

        self.loaded_chunks.get(offset)
            .expect("TODO: error handling")
            .as_ref()
            .expect("TODO: error handling")
    }

    pub fn get_offset_from_chunk_loc(&self, loc: &ChunkLocation) -> IVec3 {
        loc.0 - self.center.0
    }

    pub fn render_size(&self) -> IVec3 {
        self.render_distance.map(|n| 2 * n + 1)
    }

    pub fn drop_all_recently_requested(&mut self) {
        self.recently_requested.clear();
    }

    // TODO: clones twice if doesn't exist, none if exists; if it were to take in an owned loc, then if it exists you'd lose that
    // returns whether that chunk has already been requested
    pub fn request_chunk(&mut self, loc: &ChunkLocation) -> bool {
        self.recently_requested.try_insert(loc.clone(), REQ_TIMEOUT).is_err()

        // TODO: request server for chunk
    }

    pub fn update(&mut self, curr_chunk: ChunkLocation, delta_time: Duration, received_chunks: Vec<ChunkData>) -> Vec<ChunkLocation> {
        self.update_and_resize(curr_chunk, delta_time, received_chunks, None)
    }

    pub fn update_and_resize(&mut self, curr_chunk: ChunkLocation, delta_time: Duration, received_chunks: Vec<ChunkData>, new_render_distance: Option<&U16Vec3>) -> Vec<ChunkLocation> {
        if let Some(render_distance) = new_render_distance {
            self.render_distance = render_distance.cast();
        }

        let delta_time_sec = delta_time.as_secs_f32();

        self.recently_requested.retain(|_, t| {
            *t -= delta_time_sec;
            *t > 0.0
        });

        self.center = curr_chunk;

        let mut new_loaded = Vec::new();
        new_loaded.resize_with(self.chunk_capacity(), || None);

        // TODO: we know old center and new center, so calculate new vec positions
        for chunk_option in std::mem::take(&mut self.loaded_chunks) {
            if let Some(chunk) = chunk_option {
                if let Some(index) = self.get_index_from_chunk_location_checked(&chunk.location) {
                    *new_loaded.get_mut(index).expect("index to exist") = Some(chunk);
                }
            }
        }

        self.loaded_chunks = new_loaded;

        for chunk in received_chunks {
            self.recently_requested.remove(&chunk.location);

            let Some(index) = self.get_index_from_chunk_location_checked(&chunk.location) else {
                continue;
            };

            self.loaded_chunks
                .get_mut(index)
                .expect("index in bounds")
                .get_or_insert(chunk);

            // TODO: create GPU data
        }

        self.loaded_chunks.iter()
            .enumerate()
            .filter_map(|(i, c)| c.is_none().then_some(i))
            .map(|i| self.get_location_from_index(i))
            .filter(|loc| !self.recently_requested.contains_key(loc))
            .collect()
    }
}

fn into_1d_coordinate(coord: &IVec3, size: &IVec3) -> i32 {
    coord.x + coord.y * size.x + coord.z * size.x * size.y
}

fn into_3d_coordinate(coord: i32, size: &IVec3) -> IVec3 {
    let x = coord % size.x;
    let y = (coord / size.x) % size.y;
    let z = coord / (size.x * size.y);

    IVec3::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use glm::IVec3;
    use super::*;

    #[test]
    fn test_chunk_offset_into_chunk_vec() {
        let render = IVec3::new(5, 3, 2);

        let offset = IVec3::new(3, 9, -1) - IVec3::new(6, 7, 0);

        let norm_offset = offset + render;

        assert!(norm_offset.iter().all(|n| !n.is_negative()));

        assert_eq!(norm_offset, IVec3::new(2, 5, 1))

        // into_1d_coordinate(&norm_offset, &self.render_distance) as usize
    }
}
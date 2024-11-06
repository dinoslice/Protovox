use std::time::Duration;
use glm::IVec3;
use hashbrown::HashMap;
use shipyard::{EntitiesViewMut, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut};
use wgpu::util::DeviceExt;
use game::block::Block;
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use game::location::{BlockLocation, WorldLocation};
use crate::application::delta_time::LastDeltaTime;
use crate::chunks::client_chunk::ClientChunk;
use crate::components::{LocalPlayer, Transform};
use crate::rendering::chunk_mesh::ChunkMesh;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::sized_buffer::SizedBuffer;
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent};
use crate::render_distance::RenderDistance;

const REQ_TIMEOUT: f32 = 5.0;

#[derive(Unique)]
pub struct ChunkManager {
    // TODO: add handle to gpu buffer?
    loaded_chunks: Vec<Option<ClientChunk>>,

    render_distance: RenderDistance,
    center: ChunkLocation,

    // TODO: one big buffer?
    bakery: HashMap<ChunkLocation, SizedBuffer>,

    recently_requested_gen: HashMap<ChunkLocation, f32>,
}

impl ChunkManager {
    pub fn new(render_distance: RenderDistance, center: ChunkLocation) -> Self {
        let size = render_distance.0.iter()
            .map(|n| (2 * n + 1) as usize)
            .product();

        tracing::debug!("attempting to allocate size for {size} chunks.");

        let mut loaded_chunks = Vec::new();
        loaded_chunks.resize_with(size, || None);

        Self {
            loaded_chunks,
            render_distance,
            center,
            recently_requested_gen: HashMap::default(),
            bakery: HashMap::default(),
        }
    }

    pub fn chunk_capacity(&self) -> usize {
        self.render_distance.0.iter()
            .map(|n| (2 * n + 1) as usize)
            .product()
    }

    pub fn is_in_render_distance(&self, chunk_loc: &ChunkLocation) -> bool {
        self.get_index_checked(chunk_loc).is_some()
    }
    
    pub fn drop_all_recently_requested(&mut self) {
        self.recently_requested_gen.clear();
    }

    pub fn update_and_resize(&mut self, new_center: ChunkLocation, delta_time: Duration, received_chunks: impl IntoIterator<Item = ChunkGenEvent>, new_render_distance: Option<RenderDistance>, g_ctx: &GraphicsContext) -> Vec<ChunkGenRequestEvent> {
        // TODO: skip if no chunks changed
        if let Some(render_distance) = new_render_distance {
            self.render_distance = render_distance;
        }

        let delta_time_sec = delta_time.as_secs_f32();

        self.recently_requested_gen.retain(|_, t| {
            *t -= delta_time_sec;
            *t > 0.0
        });

        self.center = new_center;

        let mut new_loaded = Vec::new();
        new_loaded.resize_with(self.chunk_capacity(), || None);

        // TODO: we know old center and new center, so calculate new vec positions
        for chunk in std::mem::take(&mut self.loaded_chunks).into_iter().flatten() {
            match self.get_index_checked(&chunk.data.location) {
                Some(index) => *new_loaded.get_mut(index).expect("index to exist") = Some(chunk),
                None => {
                    let loc = &chunk.data.location;
                    tracing::trace!("Deleting chunk buffer at {loc:?}");
                    self.bakery.remove(loc);
                }
            }
        }

        self.loaded_chunks = new_loaded;

        for chunk in received_chunks {
            let data = chunk.0;

            self.recently_requested_gen.remove(&data.location);

            let Some(index) = self.get_index_checked(&data.location) else {
                continue;
            };

            tracing::trace!("About to insert chunk {:?} at {}", data.location, index);

            self.loaded_chunks
                .get_mut(index)
                .expect("index in bounds")
                .get_or_insert(ClientChunk::new_dirty(data));
            // TODO: create GPU data
        }

        for chunk in self.loaded_chunks.iter_mut()
            .filter_map(|cc|
                cc.as_mut().and_then(|cc|
                    cc.dirty.then_some(cc)
                )
            )
        {
            let baked = ChunkMesh::from_chunk(&chunk.data).faces;

            tracing::trace!("Finished baking chunk at {:?}", &chunk.data.location);

            let buffer = g_ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("chunk buffer"),
                    contents: bytemuck::cast_slice(&baked),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // only needed in vertex buffer,
                }
            );

            let buffer = SizedBuffer {
                buffer,
                size: baked.len() as u32,
            };

            self.bakery.insert(chunk.data.location.clone(), buffer);
            chunk.dirty = false;
        }

        let requests = self.loaded_chunks.iter()
            .enumerate()
            .filter_map(|(i, c)| c.is_none().then_some(i))
            .map(|i| self.get_location_from_index(i))
            .filter(|loc| !self.recently_requested_gen.contains_key(loc))
            .map(ChunkGenRequestEvent)
            .collect::<Vec<_>>();

        for req in &requests {
            self.recently_requested_gen.insert(req.0.clone(), REQ_TIMEOUT);
        }

        requests
    }
    
    pub fn get_index_checked(&self, location: &ChunkLocation) -> Option<usize> {
        let offset = location.0 - self.center.0;

        let render_dist_i32 = self.render_distance.0.cast();

        let norm_offset = offset + render_dist_i32;

        if norm_offset.iter()
            .enumerate()
            .any(|(i, n)| *n > 2 * render_dist_i32[i] || n.is_negative())
        {
            return None;
        }

        let index = into_1d_coordinate(&norm_offset, &self.render_distance.render_size().cast()) as usize;

        Some(index)
    }

    pub fn get_location_from_index(&self, index: usize) -> ChunkLocation {
        let norm_offset = into_3d_coordinate(index as _, &self.render_distance.render_size().cast());

        let offset = norm_offset - self.render_distance.0.cast();

        let chunk_loc = offset + self.center.0;

        ChunkLocation(chunk_loc)
    }

    // TODO: error differentiating between invalid loc & not loaded chunk
    pub fn get_chunk_ref(&self, location: &ChunkLocation) -> Option<&ClientChunk> {
        let offset = self.get_index_checked(location)?;

        self.loaded_chunks.get(offset)?.as_ref()
    }

    pub fn get_chunk_mut(&mut self, location: &ChunkLocation) -> Option<&mut ClientChunk> {
        let offset = self.get_index_checked(location)?;

        self.loaded_chunks.get_mut(offset)?.as_mut()
    }

    pub fn get_block_ref(&self, block_loc: &BlockLocation) -> Option<&Block> {
        let chunk = self.get_chunk_ref(&block_loc.into())?;

        let chunk_pos = ChunkPos::from(block_loc);

        chunk
            .data
            .blocks
            .get(chunk_pos.0 as usize)
    }

    pub fn get_block_mut(&mut self, block_loc: &BlockLocation) -> Option<&mut Block> {
        let chunk = self.get_chunk_mut(&block_loc.into())?;

        let chunk_pos = ChunkPos::from(block_loc);

        chunk
            .data
            .blocks
            .get_mut(chunk_pos.0 as usize)
    }
    
    pub fn modify_block(&mut self, block_loc: &BlockLocation, new: Block) -> Option<Block> {
        let block_mut = self.get_block_mut(block_loc)?;
        
        let prev = *block_mut;
        
        if prev != new {
            *block_mut = new;

            self.get_chunk_mut(&block_loc.into())?.dirty = true;
        }
        
        Some(prev)
    }

    pub fn loaded_locations(&self) -> Vec<ChunkLocation> {
        self.loaded_chunks.iter()
            // TODO is there a better way to do this
            .filter_map(|n|
                n.as_ref().map(|n| n.data.location.clone()) // TODO: remove clone
            )
            .collect()
    }

    pub fn baked_chunks(&self) -> &HashMap<ChunkLocation, SizedBuffer> {
        &self.bakery
    }
}

pub fn chunk_manager_update_and_request(
    mut entities: EntitiesViewMut,
    mut vm_chunk_gen_req_evt: ViewMut<ChunkGenRequestEvent>,

    delta_time: UniqueView<LastDeltaTime>,
    mut chunk_mgr: UniqueViewMut<ChunkManager>,
    vm_transform: View<Transform>,
    vm_local_player: View<LocalPlayer>,
    g_ctx: UniqueView<GraphicsContext>,
    mut chunk_gen_event: ViewMut<ChunkGenEvent>,
) {
    let (_, transform) = (&vm_local_player, &vm_transform)
        .iter()
        .next()
        .expect("TODO: local player with transform didn't exist");

    let current_chunk = WorldLocation(transform.position).into();

    let recv = chunk_gen_event.drain();

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, recv, None, &g_ctx);
    
    if !reqs.is_empty() {
        entities.bulk_add_entity(&mut vm_chunk_gen_req_evt, reqs);
    }
}

// TODO: make this not use i32
fn into_1d_coordinate(coord: &IVec3, size: &IVec3) -> i32 {
    coord.x + coord.y * size.x + coord.z * size.x * size.y
}

// TODO: make this not use i32
fn into_3d_coordinate(coord: i32, size: &IVec3) -> IVec3 {
    let x = coord % size.x;
    let y = (coord / size.x) % size.y;
    let z = coord / (size.x * size.y);

    IVec3::new(x, y, z)
}

pub fn chunk_index_in_render_distance(location: &ChunkLocation, center: &ChunkLocation, render_distance: &RenderDistance) -> Option<usize> {
    let offset = location.0 - center.0;

    let render_dist_i32 = render_distance.0.cast();

    let norm_offset = offset + render_dist_i32;

    if norm_offset.iter()
        .enumerate()
        .any(|(i, n)| *n > 2 * render_dist_i32[i] || n.is_negative())
    {
        return None;
    }

    let index = into_1d_coordinate(&norm_offset, &render_distance.render_size().cast()) as usize;

    Some(index)
}

#[cfg(test)]
mod tests {
    use glm::IVec3;

    #[test]
    fn test_chunk_offset_into_chunk_vec() {
        let render = IVec3::new(5, 3, 2);

        let offset = IVec3::new(3, 9, -1) - IVec3::new(6, 7, 0);

        let norm_offset = offset + render;

        assert!(norm_offset.iter().all(|n| !n.is_negative()));

        assert_eq!(norm_offset, IVec3::new(2, 5, 1))

        // super::into_1d_coordinate(&norm_offset, &self.render_distance) as usize
    }
}
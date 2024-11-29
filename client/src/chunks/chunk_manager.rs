use std::fmt;
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
    loaded: HashMap<ChunkLocation, ClientChunk>, // TODO: check for more optimized hashmaps

    // TODO: remove player specific state from chunk manager (+ bakery)
    render_distance: RenderDistance,
    center: ChunkLocation,

    // TODO: one big buffer?
    bakery: HashMap<ChunkLocation, SizedBuffer>,

    recently_requested_gen: HashMap<ChunkLocation, f32>,
    max_bakes_per_frame: usize,
}

impl ChunkManager {
    pub fn new(render_distance: RenderDistance, center: ChunkLocation, max_bakes_per_frame: usize) -> Self {
        let size = render_distance.0.iter()
            .map(|n| (2 * n + 1) as usize)
            .product();

        tracing::info!("Initializing ChunkManager with space for {size} chunks");

        Self {
            loaded: HashMap::with_capacity(size),
            render_distance,
            center,
            recently_requested_gen: HashMap::default(),
            bakery: HashMap::default(),
            max_bakes_per_frame,
        }
    }

    pub fn reset(&mut self) {
        self.loaded.clear();

        self.bakery.clear();
        self.recently_requested_gen.clear();
    }

    pub fn chunk_capacity(&self) -> usize {
        self.render_distance.0.iter()
            .map(|n| (2 * n + 1) as usize)
            .product()
    }
    
    pub fn render_distance(&self) -> &RenderDistance {
        &self.render_distance
    }

    #[inline(always)]
    pub fn in_render_distance(&self, chunk_loc: &ChunkLocation) -> bool {
        Self::in_render_distance_with(chunk_loc, &self.center, &self.render_distance)
    }

    // TODO: possibly optimize this method?
    pub fn in_render_distance_with(chunk_loc: &ChunkLocation, center: &ChunkLocation, render_distance: &RenderDistance) -> bool {
        let offset = chunk_loc.0 - center.0;

        let render_dist_i32 = render_distance.0.cast();

        let norm_offset = offset + render_dist_i32;

        !norm_offset.iter()
            .enumerate()
            .any(|(i, n)| *n > 2 * render_dist_i32[i] || n.is_negative())
    }
    
    pub fn clear_recently_requested(&mut self) {
        self.recently_requested_gen.clear();
    }

    pub fn update_and_resize(&mut self, new_center: ChunkLocation, delta_time: Duration, received_chunks: impl IntoIterator<Item = ChunkGenEvent>, new_render_distance: Option<RenderDistance>, g_ctx: &GraphicsContext) -> Vec<ChunkGenRequestEvent> {
        let delta_time_sec = delta_time.as_secs_f32();

        self.recently_requested_gen.retain(|_, t| {
            *t -= delta_time_sec;
            *t > 0.0
        });

        if let Some(render_distance) = new_render_distance {
            self.render_distance = render_distance;
        }

        self.center = new_center;

        for chunk in received_chunks {
            let data = chunk.0;

            self.recently_requested_gen.remove(&data.location);

            // TODO: don't bake or render chunks that aren't in render distance

            tracing::trace!("Adding {:?} to chunk manager", data.location);

            let _ = self.loaded.try_insert(data.location.clone(), ClientChunk::new_dirty(data));
        }

        for (_, chunk) in self.loaded.iter_mut()
            .filter(|(_, cc)| cc.dirty)
            .take(self.max_bakes_per_frame)
        {
            let baked = ChunkMesh::from_chunk(&chunk.data).faces;

            tracing::trace!("Finished baking chunk at {:?}", &chunk.data.location);

            let buffer = g_ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("ChunkManger chunk buffer"),
                    contents: bytemuck::try_cast_slice(&baked).expect("compatible data"),
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

        let requests = self.renderable_locations()
            .filter(|loc| !self.loaded.contains_key(loc) && !self.recently_requested_gen.contains_key(loc))
            .map(ChunkGenRequestEvent)
            .collect::<Vec<_>>();

        for req in &requests {
            self.recently_requested_gen.insert(req.0.clone(), REQ_TIMEOUT);
        }

        requests
    }

    // TODO: ideally the iterator would be &ChunkLocation instead of Transform, but this is much easier to get working
    pub fn unload_chunks<'a>(&mut self, players_info: impl IntoIterator<Item = (&'a Transform, &'a RenderDistance), IntoIter: Clone>) {
        let players_info = players_info.into_iter();

        self.loaded.retain(|loc, _| {
            let ret = players_info.clone().any(|(transform, rend)| Self::in_render_distance_with(loc, &transform.get_loc(), rend));

            if !ret {
                tracing::trace!("Deleting chunk buffer at {loc:?}");
                self.bakery.remove(loc);
            }

            ret
        });
    }

    // TODO: error differentiating between invalid loc & not loaded chunk
    pub fn get_chunk_ref(&self, location: &ChunkLocation) -> Option<&ClientChunk> {
        self.loaded.get(location)
    }

    pub fn get_chunk_mut(&mut self, location: &ChunkLocation) -> Option<&mut ClientChunk> {
        self.loaded.get_mut(location)
    }

    pub fn get_block_ref(&self, block_loc: &BlockLocation) -> Option<&Block> {
        let (loc, pos) = block_loc.as_chunk_parts();

        self
            .get_chunk_ref(&loc)?
            .data
            .blocks
            .get(pos.0 as usize)
    }

    pub fn get_block_mut(&mut self, block_loc: &BlockLocation) -> Option<&mut Block> {
        let (loc, pos) = block_loc.as_chunk_parts();

        self
            .get_chunk_mut(&loc)?
            .data
            .blocks
            .get_mut(pos.0 as usize)
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

    pub fn loaded_locations(&self) -> Vec<&ChunkLocation> {
        self.loaded.keys().collect()
    }

    pub fn renderable_locations(&self) -> impl Iterator<Item = ChunkLocation> + Clone + fmt::Debug {
        let rend_dist = self.render_distance.0.cast();

        let min = self.center.0 - rend_dist;
        let max = self.center.0 + rend_dist;

        // possible fix to prioritize xz order
        itertools::iproduct!(min.x..=max.x, min.z..=max.z, min.y..=max.y)
            .map(|(x, z, y)| ChunkLocation(IVec3::new(x, y, z)))
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

    v_render_dist: View<RenderDistance>,
    v_transform: View<Transform>
) {
    let (_, transform) = (&vm_local_player, &vm_transform)
        .iter()
        .next()
        .expect("TODO: local player with transform didn't exist");

    let recv = chunk_gen_event.drain();

    let reqs = chunk_mgr.update_and_resize(transform.get_loc(), delta_time.0, recv, None, &g_ctx);
    
    if !reqs.is_empty() {
        entities.bulk_add_entity(&mut vm_chunk_gen_req_evt, reqs);
    }

    // TODO: is it possible to eliminate the intermediate vec?
    let player_info_vec = (&v_transform, &v_render_dist).iter()
        .collect::<Vec<_>>();

    chunk_mgr.unload_chunks(player_info_vec);
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
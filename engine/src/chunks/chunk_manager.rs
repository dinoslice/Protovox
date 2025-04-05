use std::fmt;
use std::time::Duration;
use glm::IVec3;
use hashbrown::HashMap;
use shipyard::{EntitiesViewMut, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut};
use wgpu::util::DeviceExt;
use game::block::Block;
use game::block::face_type::FaceType;
use game::chunk::location::ChunkLocation;
use game::location::BlockLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::chunks::client_chunk::{BakeState, ClientChunk};
use crate::components::{LocalPlayer, Transform};
use crate::rendering::chunk_mesh::ChunkMeshContext;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::sized_buffer::SizedBuffer;
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent};
use crate::render_distance::RenderDistance;
use crate::save::WorldSaver;

const REQ_TIMEOUT: f32 = 5.0;

#[derive(Unique)]
pub struct ChunkManager {
    loaded: HashMap<ChunkLocation, ClientChunk>, // TODO: check for more optimized hashmaps

    // TODO: one big buffer?, maybe remove bakery from chunk mgr?
    bakery: HashMap<ChunkLocation, SizedBuffer>,

    recently_requested_gen: HashMap<ChunkLocation, f32>,
    max_bakes_per_frame: usize,
}

impl ChunkManager {
    pub fn new(max_bakes_per_frame: usize, expected_render_dist: Option<&RenderDistance>) -> Self {
        let size = expected_render_dist.map(RenderDistance::total_chunks).unwrap_or(0);

        Self {
            loaded: HashMap::with_capacity(size),
            recently_requested_gen: HashMap::default(),
            bakery: HashMap::with_capacity(size),
            max_bakes_per_frame,
        }
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

    pub fn update(&mut self, center: &ChunkLocation, delta_time: Duration, received_chunks: impl IntoIterator<Item = ChunkGenEvent>, render_dist: &RenderDistance, g_ctx: &GraphicsContext) -> Vec<ChunkGenRequestEvent> {
        let delta_time_sec = delta_time.as_secs_f32();

        self.recently_requested_gen.retain(|_, t| {
            *t -= delta_time_sec;
            *t > 0.0
        });

        // 3. insert any received chunks
        for chunk in received_chunks {
            let data = chunk.0;

            self.recently_requested_gen.remove(&data.location);

            tracing::trace!("Adding {:?} to chunk manager", data.location);

            // not calculating bake state here anymore, since it's done in the next section

            for ft in FaceType::ALL {
                let neighbor_loc = ChunkLocation(data.location.0 + ft.as_vector());

                self.loaded.get_mut(&neighbor_loc).map(ClientChunk::set_dirty);
            }

            let _ = self.loaded.try_insert(data.location.clone(), ClientChunk { data, bake: BakeState::DontBake });
        }

        // 2. un-bake any chunks not in OUR render distance
        // TODO: is it expensive to iterate over the hashmap again each frame? maybe only unload & update bake every few frames?
        for (loc, cc) in &mut self.loaded {
            let in_rend = Self::in_render_distance_with(loc, center, render_dist);

            match (cc.bake, in_rend) {
                (BakeState::DontBake, true) => cc.bake = BakeState::NeedsBaking,
                (BakeState::NeedsBaking, false) => cc.bake = BakeState::DontBake,
                (BakeState::Baked, false) => {
                    let had_entry = self.bakery.remove(&cc.data.location).is_some();

                    debug_assert!(had_entry, "if it was baked, it should've been in the bakery");

                    cc.bake = BakeState::DontBake;
                }

                (BakeState::NeedsBaking, true) => {}
                (BakeState::Baked, true) => {}
                (BakeState::DontBake, false) => {}
            }
        }

        // TODO: don't collect this
        let mut baked = Vec::new();

        for (_, chunk) in self.loaded.iter()
            .filter(|(_, cc)| cc.bake == BakeState::NeedsBaking)
            .take(self.max_bakes_per_frame)
        {
            // TODO change this iterator to a collected iterator iterating over location or a immutable iterator?
            let mesher = ChunkMeshContext::from_manager(self, &chunk.data);

            let faces = mesher.faces();

            tracing::trace!("Finished baking chunk at {:?}", &chunk.data.location);

            let buffer = g_ctx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("ChunkManger chunk buffer"),
                    contents: bytemuck::try_cast_slice(&faces).expect("compatible data"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // only needed in vertex buffer,
                }
            );

            let buffer = SizedBuffer {
                buffer,
                size: faces.len() as _,
            };

            self.bakery.insert(chunk.data.location.clone(), buffer);
            // chunk.bake = BakeState::Baked;
            baked.push(chunk.data.location.clone());
        }

        baked.into_iter().for_each(|loc|
            self.loaded
                .get_mut(&loc)
                .expect("should exist")
                .bake = BakeState::Baked
        );

        let requests = Self::renderable_locations_with(center, render_dist)
            .filter(|loc| !self.loaded.contains_key(loc) && !self.recently_requested_gen.contains_key(loc))
            .map(ChunkGenRequestEvent)
            .collect::<Vec<_>>();

        for req in &requests {
            self.recently_requested_gen.insert(req.0.clone(), REQ_TIMEOUT);
        }

        requests
    }

    // TODO: ideally the iterator would be &ChunkLocation instead of Transform, but this is much easier to get working
    pub fn unload_chunks<'a>(&mut self, players_info: impl IntoIterator<Item = (&'a Transform, &'a RenderDistance), IntoIter: Clone>, world_saver: &mut WorldSaver) {
        let players_info = players_info.into_iter();

        for (loc, chunk_data) in self.loaded
            .extract_if(|loc, _| !players_info.clone().any(|(transform, rend)|
                Self::in_render_distance_with(loc, &transform.get_loc(), rend)
            ))
        {
            let _had_key = self.bakery.remove(&loc);

            // TODO: this debug assert has been failing from the start, but logically it shouldn't- figure it out eventually
            // if nobody is loading this chunk, that means that when ChunkManager::update was called,
            // then earlier when we checked if WE were loading this chunk, we should've gotten false and deleted it then
            // debug_assert!(_had_key.is_none(), "chunk should've been deleted earlier!");
            
            world_saver.cache(loc, chunk_data.data);
        }
    }

    pub fn get_chunk_ref(&self, location: &ChunkLocation) -> Option<&ClientChunk> {
        self.loaded.get(location)
    }

    pub fn get_chunk_mut(&mut self, location: &ChunkLocation) -> Option<&mut ClientChunk> {
        self.loaded.get_mut(location)
    }

    pub fn get_block(&self, block_loc: &BlockLocation) -> Option<Block> {
        let (loc, pos) = block_loc.as_chunk_parts();

        self
            .get_chunk_ref(&loc)?
            .data
            .blocks
            .get(pos.0 as usize)
            .copied()
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

            self.get_chunk_mut(&block_loc.into()).map(ClientChunk::set_dirty);

            // TODO: work with chunk parts instead?
            for ft in FaceType::ALL {
                let original = BlockLocation(block_loc.0 + ft.as_vector());

                let (new_chunk_loc, new_chunk_pos) = original.as_chunk_parts();

                if let Some(chunk) = self.get_chunk_mut(&new_chunk_loc) {
                    if chunk.data.get_block(new_chunk_pos) != Block::Air {
                        chunk.set_dirty();
                    }
                }
            }
        }
        
        Some(prev)
    }

    pub fn loaded_locations(&self) -> Vec<&ChunkLocation> {
        self.loaded.keys().collect()
    }

    pub fn renderable_locations_with(center: &ChunkLocation, render_distance: &RenderDistance) -> impl Iterator<Item = ChunkLocation> + Clone + fmt::Debug {
        let rend_dist = render_distance.0.cast();

        let min = center.0 - rend_dist;
        let max = center.0 + rend_dist;

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
    vm_local_player: View<LocalPlayer>,
    g_ctx: UniqueView<GraphicsContext>,
    mut chunk_gen_event: ViewMut<ChunkGenEvent>,
    
    mut world_saver: UniqueViewMut<WorldSaver>,

    v_render_dist: View<RenderDistance>,
    v_transform: View<Transform>
) {
    let (transform, render_dist, ..) = (&v_transform, &v_render_dist, &vm_local_player)
        .iter()
        .next()
        .expect("TODO: local player with transform didn't exist");

    let recv = chunk_gen_event.drain();

    let reqs = chunk_mgr.update(&transform.get_loc(), delta_time.0, recv, render_dist, &g_ctx);
    
    if !reqs.is_empty() {
        entities.bulk_add_entity(&mut vm_chunk_gen_req_evt, reqs);
    }

    // TODO: is it possible to eliminate the intermediate vec?
    let player_info_vec = (&v_transform, &v_render_dist).iter()
        .collect::<Vec<_>>();

    chunk_mgr.unload_chunks(player_info_vec, &mut world_saver);
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
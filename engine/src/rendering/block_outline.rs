use std::array;
use shipyard::{AllStoragesView, IntoIter, Unique, UniqueView, UniqueViewMut, View};
use resources::Registry;
use crate::base_types::SELECTION_OUTLINE;
use crate::components::LocalPlayer;
use crate::game::chunk::pos::ChunkPos;
use crate::game::face_type::{FaceType, TextureId};
use crate::looking_at_block::LookingAtBlock;
use crate::rendering::face_data::FaceData;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::sized_buffer::SizedBuffer;

#[derive(Unique)]
pub struct BlockOutlineRenderState {
    pub buffer: SizedBuffer,
}

pub fn update_block_outline_buffer(
    g_ctx: UniqueView<GraphicsContext>,
    mut outline_rend_state: UniqueViewMut<BlockOutlineRenderState>,
    v_local_player: View<LocalPlayer>,
    v_looking_at_block: View<LookingAtBlock>,
    registry: UniqueView<Registry>
) {
    let (_, looking_at_block) = (&v_local_player, &v_looking_at_block)
        .iter()
        .next()
        .expect("local player should have looking at block");

    let Some(raycast) = &looking_at_block.0 else {
        return
    };

    let chunk_pos = ChunkPos::from(&raycast.hit_block);

    let block_outline_selection = registry.get(&SELECTION_OUTLINE).expect("selection texture not registered");
    let block_outline_selection = block_outline_selection.atlas_id;

    let faces: [_; 6] = array::from_fn(|ty| FaceData::new(chunk_pos, FaceType::ALL[ty], block_outline_selection));

    outline_rend_state.buffer.size = 6;

    g_ctx.queue.write_buffer(&outline_rend_state.buffer.buffer, 0, bytemuck::cast_slice(&faces));
}

pub fn initialize_block_outline_render_state(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let buffer = g_ctx.device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("block_outline_buffer"),
            size: 6 * size_of::<FaceData>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    storages.add_unique(BlockOutlineRenderState {
        buffer: SizedBuffer { buffer, size: 0 },
    })
}
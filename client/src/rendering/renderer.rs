use shipyard::{AllStoragesView, IntoWorkload, UniqueView, Workload};
use crate::rendering;
use rendering::camera_uniform_buffer::CameraUniformBuffer;
use rendering::graphics_context::GraphicsContext;
use rendering::depth_texture;
use rendering::texture_atlas;
use crate::rendering::block_outline::initialize_block_outline_render_state;
use crate::rendering::{gizmos, world};
pub fn initialize_renderer() -> Workload {
    (
        (
            texture_atlas::initialize_texture_atlas,
            depth_texture::initialize_depth_texture,
            initialize_camera_uniform_buffer,
        ).into_workload(),
        world::initialize_world_render_state,
        initialize_block_outline_render_state,
        gizmos::initialize,
    ).into_sequential_workload()
}

pub fn initialize_camera_uniform_buffer(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    storages.add_unique(CameraUniformBuffer::new(&g_ctx));
}


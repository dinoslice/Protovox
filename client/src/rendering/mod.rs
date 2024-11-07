use shipyard::{IntoWorkload, Workload};
use crate::rendering::block_outline::initialize_block_outline_render_state;
use crate::rendering::camera_uniform_buffer::initialize_camera_uniform_buffer;

pub mod graphics_context;
mod render;

pub mod texture; // TODO: fix visibility
mod face_data;

pub mod chunk_mesh;

mod camera_uniform_buffer;
pub mod depth_texture; // TODO: fix visibility
mod texture_atlas;

pub mod sized_buffer;
pub mod gizmos;
mod block_outline;
mod world;
mod egui;

pub use egui::EguiRenderer;

pub use render::render;

pub fn initialize() -> Workload {
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

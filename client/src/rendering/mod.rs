use shipyard::{IntoWorkload, Workload};
use crate::rendering::block_outline::initialize_block_outline_render_state;
use crate::rendering::camera_uniform_buffer::initialize_camera_uniform_buffer;

pub mod graphics_context;
pub mod render; // TODO: fix visibility

pub mod texture; // TODO: fix visibility
mod face_data;

pub mod chunk_mesh;

pub mod camera_uniform_buffer;
pub mod depth_texture; // TODO: fix visibility
mod texture_atlas;

pub mod sized_buffer;
pub mod gizmos;
pub mod block_outline;
pub mod world;
pub mod egui;

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
        egui::initialize_egui_renderer,
        gizmos::initialize,
    ).into_sequential_workload()
}

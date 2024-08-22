use glm::Vec3;
use na::Perspective3;
use rand::prelude::SliceRandom;
use rand::Rng;
use shipyard::{AllStoragesView, IntoWorkload, UniqueView, Workload};
use game::block::Block;
use game::chunk::data::ChunkData;
use crate::camera::Camera;
use crate::application::CaptureState;
use crate::application::delta_time::LastDeltaTime;
use crate::input::InputManager;
use crate::rendering::chunk_mesh::ChunkMesh;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::renderer;

pub fn startup() -> Workload {
    (
        renderer::initialize_renderer,
        init_chunk_faces,
        initialize_camera,
        initialize_application_systems,
    ).into_workload()
}

fn init_chunk_faces(storages: AllStoragesView) {
    let mut chunk = ChunkData::default();

    for i in 0..65536 {
        if rand::thread_rng().gen_bool(0.1) {
            chunk.blocks[i] = *[
                Block::Grass,
                Block::Dirt,
                Block::Cobblestone,
            ].choose(&mut rand::thread_rng())
                .expect("blocks exist");
        }
    }

    // TODO: move this elsewhere
    let baked = ChunkMesh::from_chunk(&chunk);
    storages.add_unique(baked);
}

pub fn initialize_camera(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    storages.add_unique(Camera {
        position: Vec3::new(0.0, 0.0, 0.0),
        yaw: 90.0f32.to_radians(),
        pitch: -20.0f32.to_radians(),
        speed: 8.0,
        perspective: Perspective3::new(
            g_ctx.aspect(),
            45.0f32.to_radians(),
            0.01,
            1000.0
        )
    })
}

pub fn initialize_application_systems(storages: AllStoragesView) {
    storages.add_unique(InputManager::with_mouse_sensitivity(0.75));
    storages.add_unique(CaptureState::default());
    storages.add_unique(LastDeltaTime::default())
}


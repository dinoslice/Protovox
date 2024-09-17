use glm::{U16Vec3, Vec3};
use na::Perspective3;
use rand::prelude::SliceRandom;
use rand::Rng;
use shipyard::{AllStoragesView, IntoWorkload, SystemModificator, UniqueView, Workload};
use game::block::Block;
use game::chunk::data::ChunkData;
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::camera::Camera;
use crate::application::CaptureState;
use crate::application::delta_time::LastDeltaTime;
use crate::args;
use crate::chunks::chunk_manager::ChunkManager;
use crate::environment::{Environment, is_hosted, is_multiplayer_client};
use crate::input::InputManager;
use crate::multiplayer::server_connection::ServerConnection;
use crate::networking::server_socket::ServerHandler;
use crate::render_distance::RenderDistance;
use crate::rendering::chunk_mesh::ChunkMesh;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::renderer;
use crate::world_gen::WorldGenerator;

pub fn startup() -> Workload {
    (
        args::parse_env,
        renderer::initialize_renderer,
        init_chunk_faces,
        initialize_camera,
        initialize_gameplay_systems.after_all(initialize_camera),
        initialize_application_systems,
        initialize_networking.after_all(args::parse_env),
    ).into_sequential_workload()
}

fn init_chunk_faces(storages: AllStoragesView) {
    let mut chunk = ChunkData::empty(ChunkLocation::default());

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
    });
}

pub fn initialize_gameplay_systems(storages: AllStoragesView, camera: UniqueView<Camera>) {
    storages.add_unique(ChunkManager::new(
        RenderDistance(U16Vec3::new(3,0,3)),
        ChunkLocation::from(WorldLocation(camera.position))
    ));
}

pub fn initialize_application_systems(storages: AllStoragesView) {
    storages.add_unique(InputManager::with_mouse_sensitivity(0.75));
    storages.add_unique(CaptureState::default());
    storages.add_unique(LastDeltaTime::default());
    storages.add_unique(WorldGenerator::new(50));
}

fn initialize_networking(env: UniqueView<Environment>, storages: AllStoragesView) {
    if storages.run(is_hosted) {
        storages.add_unique(ServerHandler::new());
    } else if storages.run(is_multiplayer_client) {
        let Environment::MultiplayerClient(addr) = *env else {
            unreachable!();
        };

        storages.add_unique(ServerConnection::bind(addr))
    }
}


use glm::{U16Vec3, Vec3};
use na::Perspective3;
use shipyard::{AllStoragesView, AllStoragesViewMut, EntitiesViewMut, IntoWorkload, SystemModificator, UniqueView, ViewMut, Workload};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::camera::Camera;
use crate::application::CaptureState;
use crate::application::delta_time::LastDeltaTime;
use crate::args;
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{Entity, GravityAffected, Hitbox, LocalPlayer, Player, PlayerSpeed, Transform, Velocity};
use crate::environment::{Environment, is_hosted, is_multiplayer_client};
use crate::input::InputManager;
use crate::multiplayer::server_connection::ServerConnection;
use crate::networking::server_socket::ServerHandler;
use crate::render_distance::RenderDistance;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::renderer;
use crate::world_gen::WorldGenerator;

pub fn startup() -> Workload {
    (
        args::parse_env,
        renderer::initialize_renderer,
        initialize_local_player,
        initialize_gameplay_systems.after_all(initialize_local_player),
        initialize_application_systems,
        initialize_networking.after_all(args::parse_env),
    ).into_sequential_workload()
}

fn initialize_local_player(mut storages: AllStoragesViewMut) {
    let aspect = storages
        .borrow::<UniqueView<GraphicsContext>>()
        .expect("unable to borrow graphics context")
        .aspect();

    storages.add_entity((
        LocalPlayer,
        Player,
        Entity,
        GravityAffected,
        Transform {
            position: Vec3::new(0.5, 20.0, 0.5),
            .. Default::default()
        },
        Velocity::default(),
        PlayerSpeed {
            max_vel: 4.32,
            jump_vel: 4.95,
            accel: 0.098 * 20.0,
            friction: 0.546 * 20.0,
        },
        Camera {
            offset: Vec3::new(0.0, 0.5, 0.0),
            perspective: Perspective3::new(
                aspect,
                45.0f32.to_radians(),
                0.01,
                1000.0
            ),
        },
        Hitbox(Vec3::new(0.6, 2.0, 0.6))
    ));
}

pub fn initialize_gameplay_systems(storages: AllStoragesView) {
    let iter = &mut storages.iter::<(&LocalPlayer, &Transform)>();

    let transform = iter.iter()
        .next()
        .expect("TODO: local player with transform should exist")
        .1;

    storages.add_unique(ChunkManager::new(
        RenderDistance(U16Vec3::new(3,0,3)),
        ChunkLocation::from(WorldLocation(transform.position))
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


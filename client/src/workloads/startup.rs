use std::time::Instant;
use glm::{U16Vec3, Vec3};
use na::Perspective3;
use shipyard::{AllStoragesView, AllStoragesViewMut, IntoWorkload, SystemModificator, UniqueView, Workload};
use game::block::Block;
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::camera::Camera;
use crate::{args, rendering};
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{Entity, GravityAffected, HeldBlock, Hitbox, IsOnGround, LocalPlayer, Player, PlayerSpeed, SpectatorSpeed, Transform, Velocity};
use crate::environment::{Environment, is_hosted, is_multiplayer_client};
use crate::gamemode::Gamemode;
use crate::input::InputManager;
use crate::input::mouse_manager::MouseManager;
use crate::last_world_interaction::LastWorldInteraction;
use crate::looking_at_block::LookingAtBlock;
use crate::networking::server_connection::ServerConnection;
use crate::networking::keep_alive::init_keep_alive;
use crate::networking::server_handler::ServerHandler;
use crate::render_distance::RenderDistance;
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;
use crate::world_gen_debugger::spline_editor::SplineEditor;

pub fn startup() -> Workload {
    (
        args::parse_env,
        rendering::initialize,
        initialize_local_player,
        initialize_gameplay_systems.after_all(initialize_local_player),
        initialize_networking.after_all(args::parse_env),
        init_keep_alive,//.run_if(is_hosted),
        set_window_title,
    ).into_sequential_workload()
}

fn initialize_local_player(mut storages: AllStoragesViewMut) {
    let aspect = storages
        .borrow::<UniqueView<GraphicsContext>>()
        .expect("unable to borrow graphics context")
        .aspect();

    let id = storages.add_entity((
        LocalPlayer,
        Player,
        Entity,
        GravityAffected,
        IsOnGround::default(),
        Transform {
            position: Vec3::new(0.5, 20.0, 0.5),
            .. Default::default()
        },
        Velocity::default(),
        PlayerSpeed::default(),
        Camera {
            offset: Vec3::new(0.0, 0.5, 0.0),
            perspective: Perspective3::new(
                aspect,
                70.0f32.to_radians(),
                0.01,
                1000.0
            ),
        },
        Hitbox::default_player(),
    ));
    
    storages.add_component(id, LookingAtBlock(None)); // TODO: fix a better way for >10 components
    storages.add_component(id, HeldBlock(Block::Cobblestone));
    storages.add_component(id, Gamemode::Survival);
    storages.add_component(id, SpectatorSpeed::default()); // TODO: should this always be on the player or only added when switching gamemodes?
}

pub fn initialize_gameplay_systems(storages: AllStoragesView) {
    let iter = &mut storages.iter::<(&LocalPlayer, &Transform)>();

    let transform = iter.iter()
        .next()
        .expect("TODO: local player with transform should exist")
        .1;

    storages.add_unique(ChunkManager::new(
        RenderDistance(U16Vec3::new(3,1,3)),
        ChunkLocation::from(WorldLocation(transform.position))
    ));
    storages.add_unique(WorldGenerator::new(50));
    storages.add_unique(LastWorldInteraction(Instant::now()));
    storages.add_unique(SplineEditor::default());
}

fn initialize_networking(env: UniqueView<Environment>, storages: AllStoragesView) {
    if storages.run(is_hosted) {
        storages.add_unique(ServerHandler::new(None));
    } else if storages.run(is_multiplayer_client) {
        let Environment::MultiplayerClient(addr) = *env else {
            unreachable!();
        };

        storages.add_unique(ServerConnection::bind(addr))
    }
}

fn set_window_title(g_ctx: UniqueView<GraphicsContext>, env: UniqueView<Environment>) {
    g_ctx.window.set_title(&format!("voxel game: {}", *env))
}

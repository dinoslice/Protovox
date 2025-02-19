use glm::{U16Vec3, Vec3};
use na::Perspective3;
use shipyard::{AllStoragesView, AllStoragesViewMut, UniqueOrDefaultViewMut, UniqueView};
use game::block::Block;
use networking::PacketRegistry;
use crate::camera::Camera;
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{Entity, GravityAffected, HeldBlock, Hitbox, IsOnGround, LocalPlayer, Player, PlayerSpeed, SpectatorSpeed, Transform, Velocity};
use crate::environment::{Environment, is_hosted, is_multiplayer_client};
use crate::gamemode::Gamemode;
use crate::looking_at_block::LookingAtBlock;
use crate::networking::server_connection::ServerConnection;
use crate::networking::server_handler::ServerHandler;
use crate::render_distance::RenderDistance;
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;

pub fn initialize_local_player(mut storages: AllStoragesViewMut) {
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
            offset: Vec3::new(0.0, 0.5, 0.0), // TODO: spawning in in free space causes gravity to not work
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
    storages.add_component(id, RenderDistance(U16Vec3::new(3,1,3)));
}

pub fn initialize_gameplay_systems(storages: AllStoragesView) {
    let iter = &mut storages.iter::<(&RenderDistance, &LocalPlayer)>();

    let (render_dist, ..) = iter.iter()
        .next()
        .expect("TODO: local player with transform should exist");

    storages.add_unique(ChunkManager::new(6, Some(render_dist)));
    storages.add_unique(WorldGenerator::new(50));
}

pub fn initialize_networking(env: UniqueView<Environment>, registry: UniqueView<PacketRegistry>, storages: AllStoragesView) {
    if storages.run(is_hosted) {
        storages.add_unique(ServerHandler::new(None));
    } else if storages.run(is_multiplayer_client) {
        let Environment::MultiplayerClient(addr) = *env else {
            unreachable!();
        };

        let connection_request_ser_id = registry
            .identifier_of()
            .expect("should be registered");

        storages.add_unique(ServerConnection::bind(addr, connection_request_ser_id));
    }
}

pub fn register_packets(mut registry: UniqueOrDefaultViewMut<PacketRegistry>) {
    use crate::events::{*, render_distance::*};

    registry.register::<ChunkGenRequestEvent, false, false>();
    registry.register::<ChunkGenEvent, true, false>();
    registry.register::<BlockUpdateEvent, false, true>();
    registry.register::<ClientInformationRequestEvent, false, false>();
    registry.register::<ClientInformationUpdateEvent, false, false>();
    registry.register::<ClientSettingsRequestEvent, false, false>();
    registry.register::<ClientSettingsUpdateEvent, false, false>();
    registry.register::<ConnectionRequest, false, false>();
    registry.register::<ConnectionSuccess, false, false>();
    registry.register::<ClientTransformUpdate, false, false>();
    registry.register::<ClientChunkRequest, false, true>();
    registry.register::<KeepAlive, false, false>();
    registry.register::<KickedByServer, false, false>();
    registry.register::<RenderDistanceRequestEvent, false, false>();
    registry.register::<RenderDistanceUpdateEvent, false, false>();
}

pub fn set_window_title(g_ctx: UniqueView<GraphicsContext>, env: UniqueView<Environment>) {
    g_ctx.window.set_title(&format!("voxel game: {}", *env))
}

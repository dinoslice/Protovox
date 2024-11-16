use glm::{U16Vec3, Vec3};
use na::Perspective3;
use shipyard::{AllStoragesView, AllStoragesViewMut, EntitiesViewMut, IntoIter, IntoWorkload, UniqueView, UniqueViewMut, View, ViewMut, Workload};
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::chunks::chunk_manager::ChunkManager;
use crate::components::{Entity, IsOnGround, LocalPlayer, Player, PlayerSpeed, Transform, Velocity};
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent};
use crate::input::reset_mouse_manager_state;
use crate::looking_at_block::LookingAtBlock;
use crate::physics::{process_input, process_physics};
use crate::render_distance::RenderDistance;
use crate::rendering;
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;
use crate::world_gen_debugger::spline_editor::SplineEditor;

pub mod spline_editor;
pub mod render;

pub fn startup() -> Workload {
    (
        rendering::initialize,
        init_debug_player,
        initialize_game_systems,
    ).into_sequential_workload()
}

pub fn update() -> Workload {
    (
        process_input,
        process_physics,
        reset_mouse_manager_state,
        get_generated_chunks,
        chunk_manager_update_and_request,
        generate_chunks,
    ).into_sequential_workload()
}

pub fn shutdown() -> Workload {
    (
        || (),
    ).into_sequential_workload()
}

fn init_debug_player(mut storages: AllStoragesViewMut) {
    let aspect = storages
        .borrow::<UniqueView<GraphicsContext>>()
        .expect("unable to borrow graphics context")
        .aspect();

    let _id = storages.add_entity((
        LocalPlayer,
        Player,
        Entity,
        Transform {
            position: Vec3::new(0.5, 20.0, 0.5),
            .. Default::default()
        },
        IsOnGround::default(),
        Velocity::default(),
        PlayerSpeed::from_observed(
            4.32,
            1.25,
            9.8,
            0.2,
            0.18
        ),
        Camera {
            offset: Vec3::new(0.0, 0.5, 0.0),
            perspective: Perspective3::new(
                aspect,
                70.0f32.to_radians(),
                0.01,
                1000.0
            ),
        },
        LookingAtBlock(None),
    ));
}

fn initialize_game_systems(storages: AllStoragesView) {
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
    storages.add_unique(SplineEditor::default());
}

pub fn chunk_manager_update_and_request(
    mut entities: EntitiesViewMut,
    mut vm_chunk_gen_req_evt: ViewMut<ChunkGenRequestEvent>,

    delta_time: UniqueView<LastDeltaTime>,
    mut chunk_mgr: UniqueViewMut<ChunkManager>,
    vm_transform: View<Transform>,
    vm_local_player: View<LocalPlayer>,
    g_ctx: UniqueView<GraphicsContext>,
    mut chunk_gen_event: ViewMut<ChunkGenEvent>,
) {
    let (_, transform) = (&vm_local_player, &vm_transform)
        .iter()
        .next()
        .expect("TODO: local player with transform didn't exist");

    let current_chunk = WorldLocation(transform.position).into();

    let recv = chunk_gen_event.drain();

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, recv, None, &g_ctx);

    if !reqs.is_empty() {
        tracing::debug!("bulk requesting: {}", reqs.len());
        entities.bulk_add_entity(&mut vm_chunk_gen_req_evt, reqs);
    }
}

fn get_generated_chunks(world_gen: UniqueView<WorldGenerator>, mut vm_entities: EntitiesViewMut, vm_chunk_gen_evt: ViewMut<ChunkGenEvent>) {
    let chunks = world_gen.receive_chunks();

    drop(world_gen);

    if !chunks.is_empty() {
        vm_entities.bulk_add_entity(vm_chunk_gen_evt, chunks);
    }
}

fn generate_chunks(mut reqs: ViewMut<ChunkGenRequestEvent>, world_generator: UniqueView<WorldGenerator>) {
    for req in reqs.drain() {
        world_generator.spawn_generate_task(req.0);
    }
}
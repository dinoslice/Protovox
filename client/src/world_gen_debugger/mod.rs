use glm::{U16Vec3, Vec3};
use na::Perspective3;
use shipyard::{AllStoragesView, AllStoragesViewMut, EntitiesViewMut, IntoIter, IntoWorkload, SystemModificator, WorkloadModificator, UniqueView, UniqueViewMut, View, ViewMut, Workload};
use game::chunk::CHUNK_SIZE;
use game::chunk::location::ChunkLocation;
use game::chunk::pos::ChunkPos;
use game::location::{BlockLocation, WorldLocation};
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
use crate::world_gen_debugger::params::WorldGenVisualizerParams;
use crate::world_gen_debugger::spline_editor::SplineEditor;

pub mod spline_editor;
pub mod render;
mod params;

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
        process_physics.skip_if(locked_position),
        reset_mouse_manager_state,
        get_generated_chunks,
        chunk_manager_update_and_request,
        generate_chunks,
        set_locked_position.run_if(locked_position)
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

    let center = ChunkLocation::from(WorldLocation(transform.position));

    storages.add_unique(ChunkManager::new(RenderDistance(U16Vec3::new(3,1,3)), center.clone()));
    storages.add_unique(WorldGenerator::new(50));
    storages.add_unique(SplineEditor::default());
    storages.add_unique(WorldGenVisualizerParams {
        generate_center: center,
        render_distance: RenderDistance(U16Vec3::new(3, 1, 3)),
        cam_offset: Default::default(),
        lock_position: false,
    });
}

fn set_locked_position(v_local_player: View<LocalPlayer>, mut vm_transform: ViewMut<Transform>, mut vm_velocity: ViewMut<Velocity>, vis_params: UniqueView<WorldGenVisualizerParams>) {
    let (transform, velocity, ..) = (&mut vm_transform, &mut vm_velocity, &v_local_player).iter()
        .next()
        .expect("local player should have transform");

    let mut target_chunk = vis_params.generate_center.clone();

    target_chunk.0.y += vis_params.render_distance.0.y as i32;

    let center = BlockLocation::from_chunk_parts(&target_chunk, &ChunkPos::center());

    transform.position = WorldLocation::from(&center).0 + vis_params.cam_offset.0;
    velocity.0 = Vec3::zeros();
}

pub fn chunk_manager_update_and_request(
    mut entities: EntitiesViewMut,
    mut vm_chunk_gen_req_evt: ViewMut<ChunkGenRequestEvent>,

    delta_time: UniqueView<LastDeltaTime>,
    mut chunk_mgr: UniqueViewMut<ChunkManager>,
    g_ctx: UniqueView<GraphicsContext>,
    mut chunk_gen_event: ViewMut<ChunkGenEvent>,
    vis_params: UniqueView<WorldGenVisualizerParams>,
) {
    let recv = chunk_gen_event.drain();

    let reqs = chunk_mgr.update_and_resize(
        vis_params.generate_center.clone(),
        delta_time.0,
        recv,
        Some(vis_params.render_distance.clone()),
        &g_ctx
    );

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

fn locked_position(vis_params: UniqueView<WorldGenVisualizerParams>) -> bool {
    vis_params.lock_position
}
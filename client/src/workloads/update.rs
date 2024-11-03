use glm::Vec3;
use crate::chunks::chunk_manager::ChunkManager;
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator, AllStoragesViewMut, ViewMut, IntoIter, View, EntitiesViewMut};
use game::block::Block;
use game::chunk::location::ChunkLocation;
use game::location::WorldLocation;
use crate::application::delta_time::LastDeltaTime;
use crate::camera::Camera;
use crate::chunks::raycast::RaycastResult;
use crate::components::{Entity, GravityAffected, Hitbox, IsOnGround, LocalPlayer, Player, PlayerSpeed, Transform, Velocity};
use crate::environment::is_hosted;
use crate::events::{ChunkGenEvent, ChunkGenRequestEvent, ClientInformationRequestEvent};
use crate::input::action_map::Action;
use crate::input::InputManager;
use crate::looking_at_block::LookingAtBlock;
use crate::networking;
use crate::physics::movement::{adjust_fly_speed, apply_camera_input, process_movement};
use crate::physics::process_physics;
use crate::rendering::gizmos;
use crate::rendering::gizmos::{BoxGizmo, GizmoLifetime, GizmoStyle};
use crate::rendering::graphics_context::GraphicsContext;
use crate::world_gen::WorldGenerator;

pub fn update() -> Workload {
    (
        process_input,
        process_physics,
        reset_mouse_manager_state,
        networking::update_networking,
        get_generated_chunks.run_if(is_hosted),
        chunk_manager_update_and_request,
        generate_chunks.run_if(is_hosted),
        debug_draw_hitbox_gizmos,
        spawn_multiplayer_player,
        raycast,
        place_break_blocks,
        gizmos::update,
    ).into_sequential_workload()
}

pub fn process_input() -> Workload {
    (
        apply_camera_input,
        process_movement,
        adjust_fly_speed,
    ).into_workload()
}

fn raycast(chunk_mgr: UniqueView<ChunkManager>, v_local_player: View<LocalPlayer>, v_transform: View<Transform>, v_camera: View<Camera>, mut vm_looking_at_block: ViewMut<LookingAtBlock>) {
    let (_, transform, camera, looking_at_block) = (&v_local_player, &v_transform, &v_camera, &mut vm_looking_at_block)
        .iter()
        .next()
        .expect("TODO: local player with transform & camera didn't exist");

    let raycast_origin = transform.position + camera.offset;

    // TODO: get this direction vector in a better way
    let direction = Vec3::new(
        transform.yaw.cos() * transform.pitch.cos(),
        transform.pitch.sin(),
        transform.yaw.sin() * transform.pitch.cos(),
    );

    looking_at_block.0 = chunk_mgr.raycast(&raycast_origin, &direction, 15.0, 0.1);
}

fn place_break_blocks(mut chunk_mgr: UniqueViewMut<ChunkManager>, v_local_player: View<LocalPlayer>, v_looking_at_block: View<LookingAtBlock>, input: UniqueView<InputManager>) {
    let Some(RaycastResult { prev_air, hit_position, .. }) = (&v_local_player, &v_looking_at_block)
        .iter()
        .next()
        .and_then(|(_, look_at)| look_at.0.as_ref())
    else {
        return
    };

    if input.action_map.get_action(Action::PlaceBlock) {
        if let Some(prev_air) = prev_air {
            if let Some(b) = chunk_mgr.get_block_ref_from_world_loc_mut(prev_air) {
                *b = Block::Cobblestone;
                chunk_mgr.set_dirty_if_exists(prev_air);
            }
        }
    } else if input.action_map.get_action(Action::BreakBlock) {
        if let Some(b) = chunk_mgr.get_block_ref_from_world_loc_mut(hit_position) {
            *b = Block::Air;
            chunk_mgr.set_dirty_if_exists(hit_position);
        }
    }
}

fn spawn_multiplayer_player(
    // TODO: for now using this event to spawn the player's components
    mut vm_info_req_evt: ViewMut<ClientInformationRequestEvent>,

    // TODO: better way to keep component list in sync
    entities: EntitiesViewMut,
    mut vm_player: ViewMut<Player>,
    mut vm_entity: ViewMut<Entity>,
    mut vm_gravity_affected: ViewMut<GravityAffected>,
    mut vm_is_on_ground: ViewMut<IsOnGround>,
    mut vm_transform: ViewMut<Transform>,
    mut vm_velocity: ViewMut<Velocity>,
    mut vm_player_speed: ViewMut<PlayerSpeed>,
    mut vm_hitbox: ViewMut<Hitbox>,
) {
    for (id, _) in vm_info_req_evt.drain().with_id() {
        entities.add_component(id,
            (
                &mut vm_player,
                &mut vm_entity,
                &mut vm_gravity_affected,
                &mut vm_is_on_ground,
                &mut vm_transform,
                &mut vm_velocity,
                &mut vm_player_speed,
                &mut vm_hitbox
            ),
            (
                Player,
                Entity,
                GravityAffected,
                IsOnGround::default(),
                Transform {
                    position: Vec3::new(0.5, 20.0, 0.5),
                    .. Default::default()
                },
                Velocity::default(),
                PlayerSpeed::from_observed(
                    4.32,
                    1.25,
                    9.8,
                    0.2,
                    0.18
                ),
                Hitbox(Vec3::new(0.6, 2.0, 0.6))
            )
        );
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

// TODO: fix borrowing of storages
fn chunk_manager_update_and_request(mut all_storages: AllStoragesViewMut) {
    if let Some(reqs) = all_storages.run(chunk_manager_update) {
        all_storages.bulk_add_entity(reqs.into_iter());
    }
}

fn chunk_manager_update(delta_time: UniqueView<LastDeltaTime>, mut chunk_mgr: UniqueViewMut<ChunkManager>, vm_transform: View<Transform>, vm_local_player: View<LocalPlayer>, g_ctx: UniqueView<GraphicsContext>, mut chunk_gen_event: ViewMut<ChunkGenEvent>) -> Option<Vec<ChunkGenRequestEvent>> {
    let transform = (&vm_local_player, &vm_transform)
        .iter()
        .next()
        .expect("TODO: local player with transform didn't exist")
        .1;

    let current_chunk = ChunkLocation::from(WorldLocation(transform.position));

    let recv = chunk_gen_event.drain().collect();

    let reqs = chunk_mgr.update_and_resize(current_chunk, delta_time.0, recv, None, &g_ctx);

    (!reqs.is_empty()).then_some(reqs)
}

fn reset_mouse_manager_state(mut input_manager: UniqueViewMut<InputManager>) {
    input_manager.mouse_manager.reset_scroll_rotate();
}

fn debug_draw_hitbox_gizmos(
    v_hitbox: View<Hitbox>,
    v_transform: View<Transform>,

    mut entities: EntitiesViewMut,
    mut vm_box_gizmos: ViewMut<BoxGizmo>,
) {
    for (transform, hitbox) in (&v_transform, &v_hitbox).iter() {
        let half_hitbox = hitbox.0 * 0.5;

        let min_extent = transform.position - half_hitbox;
        let max_extent = transform.position + half_hitbox;

        entities.add_entity(&mut vm_box_gizmos, BoxGizmo::from_corners(
            min_extent,
            max_extent,
            GizmoStyle::stroke(rgb::Rgb { r: 1.0, g: 0.0, b: 0.0 }),
            GizmoLifetime::SingleFrame,
        ));
    }
}
use glm::Vec3;
use crate::chunks::chunk_manager::{ChunkManager, chunk_manager_update_and_request};
use shipyard::{IntoWorkload, UniqueView, UniqueViewMut, Workload, SystemModificator, ViewMut, IntoIter, View, EntitiesViewMut, WorkloadModificator};
use game::block::Block;
use game::location::BlockLocation;
use crate::camera::Camera;
use crate::chunks::raycast::BlockRaycastResult;
use crate::components::{Entity, GravityAffected, Hitbox, IsOnGround, LocalPlayer, Player, PlayerSpeed, Transform, Velocity};
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::events::{BlockUpdateEvent, ChunkGenEvent, ChunkGenRequestEvent, ClientInformationRequestEvent};
use crate::input::action_map::Action;
use crate::input::InputManager;
use crate::last_world_interaction::LastWorldInteraction;
use crate::looking_at_block::LookingAtBlock;
use crate::networking;
use crate::physics::movement::{adjust_fly_speed, apply_camera_input, process_movement};
use crate::physics::{collision, process_physics};
use crate::rendering::gizmos;
use crate::rendering::gizmos::{BoxGizmo, GizmoLifetime, GizmoStyle};
use crate::world_gen::WorldGenerator;

pub fn update() -> Workload {
    (
        update_input_manager,
        process_input,
        process_physics,
        reset_mouse_manager_state,
        networking::update_networking_server.run_if(is_hosted),
        networking::update_networking_client.run_if(is_multiplayer_client),
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

fn update_input_manager(mut input: UniqueViewMut<InputManager>) {
    input.process();
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

    looking_at_block.0 = chunk_mgr.raycast(&raycast_origin, &direction, 4.5, 0.1);
}

fn place_break_blocks(
    mut chunk_mgr: UniqueViewMut<ChunkManager>,
    v_local_player: View<LocalPlayer>,
    v_looking_at_block: View<LookingAtBlock>,
    input: UniqueView<InputManager>,
    mut last_world_interaction: UniqueViewMut<LastWorldInteraction>,

    // to ensure we're placing at a valid spot
    v_entity: View<Entity>,
    v_transform: View<Transform>,
    v_hitbox: View<Hitbox>,
    
    mut entities: EntitiesViewMut,
    mut vm_block_update_evts: ViewMut<BlockUpdateEvent>,
) {
    let Some(BlockRaycastResult { prev_air, hit_block, .. }) = (&v_local_player, &v_looking_at_block)
        .iter()
        .next()
        .and_then(|(_, look_at)| look_at.0.as_ref())
    else {
        return
    };

    let mut should_place = input.just_pressed().get_action(Action::PlaceBlock);
    let mut should_break = input.just_pressed().get_action(Action::BreakBlock);

    if last_world_interaction.cooldown_passed() {
        should_place |= input.pressed().get_action(Action::PlaceBlock);
        should_break |= input.pressed().get_action(Action::BreakBlock);
    }

    let mut update_block = |pos: BlockLocation, block: Block| {
        chunk_mgr.modify_block(&pos, block); // TODO: only create event now, modify world later?
        last_world_interaction.reset_cooldown();

        entities.add_entity(&mut vm_block_update_evts, BlockUpdateEvent(pos.clone(), block));
    };

    if should_place && should_break {
        update_block(hit_block.clone(), Block::Cobblestone);
    } else if should_break {
        update_block(hit_block.clone(), Block::Air);
    } else if should_place {
        if let Some(prev_air) = prev_air {
            let (min, max) = prev_air.get_aabb_bounds();

            if collision::collides_with_any_entity(min, max, v_entity, v_transform, v_hitbox).is_none() {
                update_block(prev_air.clone(), Block::Cobblestone);
            }
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
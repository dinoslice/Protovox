use glm::Vec3;
use crate::chunks::chunk_manager::ChunkManager;
use shipyard::{UniqueView, UniqueViewMut, ViewMut, IntoIter, View, EntitiesViewMut, EntitiesView, IntoWithId, Remove, UniqueOrDefaultViewMut};
use strum::EnumCount;
use game::block::{Block, BlockTy};
use game::inventory::Inventory;
use game::item::{ItemStack, ItemType};
use game::location::BlockLocation;
use crate::camera::Camera;
use crate::chunks::raycast::{RaycastHit, RaycastResult};
use crate::components::{Entity, GravityAffected, HeldBlock, Hitbox, IsOnGround, LocalPlayer, Player, PlayerSpeed, SpectatorSpeed, Transform, Velocity};
use crate::events::{BlockUpdateEvent, ChunkGenEvent, ChunkGenRequestEvent, ClientInformationRequestEvent};
use crate::events::event_bus::EventBus;
use crate::gamemode::Gamemode;
use crate::input::action_map::Action;
use crate::input::InputManager;
use crate::inventory::PlayerInventory;
use crate::last_world_interaction::LastWorldInteraction;
use crate::looking_at_block::LookingAtBlock;
use crate::physics::{collision};
use crate::save::WorldSaver;
use crate::world_gen::WorldGenerator;

pub fn toggle_gamemode(
    input: UniqueView<InputManager>,
    v_local_player: View<LocalPlayer>,
    mut vm_looking_at_block: ViewMut<LookingAtBlock>,
    mut vm_gamemode: ViewMut<Gamemode>,
    mut vm_spec_speed: ViewMut<SpectatorSpeed>,
    mut vm_velocity: ViewMut<Velocity>,
    mut vm_hitbox: ViewMut<Hitbox>,
    mut vm_gravity_affected: ViewMut<GravityAffected>,
    entities: EntitiesView,
) {
    if !input.just_pressed().get_action(Action::ToggleGamemode) {
        return;
    }
    
    let (id, (_, gamemode, velocity, look_at, spec_speed)) = (&v_local_player, &mut vm_gamemode, &mut vm_velocity, &mut vm_looking_at_block, &mut vm_spec_speed).iter().with_id()
        .next()
        .expect("local player should have gamemode and velocity");
    
    match gamemode {
        Gamemode::Survival => {
            *gamemode = Gamemode::Spectator;
            spec_speed.curr_speed = SpectatorSpeed::default().curr_speed;
            look_at.0 = None;
            
            vm_gravity_affected.remove(id);
            vm_hitbox.remove(id);
        },
        Gamemode::Spectator => {
            *gamemode = Gamemode::Survival;
            
            entities.add_component(id, &mut vm_hitbox, Hitbox::default_player());
            entities.add_component(id, &mut vm_gravity_affected, GravityAffected);
        },
    };

    *velocity = Velocity::default();
}

pub fn update_world_saver(mut world_saver: UniqueViewMut<WorldSaver>) {
    world_saver.process();
}

pub fn server_apply_block_updates(mut world: UniqueViewMut<ChunkManager>, mut vm_block_update_evt_bus: ViewMut<EventBus<BlockUpdateEvent>>, mut vm_block_update_evt: ViewMut<BlockUpdateEvent>) {
    for mut bus in vm_block_update_evt_bus.drain() {
        for BlockUpdateEvent(loc, new_block) in bus.0.drain(..) {
            if world.modify_block(&loc, new_block).is_err() {
                tracing::error!("Location from block update wasn't loaded");
            }
        }
    }
    
    vm_block_update_evt.drain();
}

pub fn client_apply_block_updates(mut world: UniqueViewMut<ChunkManager>, mut vm_block_update_evt_bus: ViewMut<BlockUpdateEvent>) {
    for BlockUpdateEvent(loc, new_block) in vm_block_update_evt_bus.drain() {
        if world.modify_block(&loc, new_block).is_err() {
            tracing::error!("Location from block update wasn't loaded");
        }
    }
}

pub fn raycast(chunk_mgr: UniqueView<ChunkManager>, v_local_player: View<LocalPlayer>, v_transform: View<Transform>, v_camera: View<Camera>, mut vm_looking_at_block: ViewMut<LookingAtBlock>) {
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

    looking_at_block.0 = chunk_mgr.raycast(raycast_origin, direction, 4.5);
}

pub fn place_break_blocks(
    mut chunk_mgr: UniqueViewMut<ChunkManager>,
    v_local_player: View<LocalPlayer>,
    v_looking_at_block: View<LookingAtBlock>,
    held: UniqueView<HeldBlock>,
    input: UniqueView<InputManager>,
    mut last_world_interaction: UniqueOrDefaultViewMut<LastWorldInteraction>,

    // to ensure we're placing at a valid spot
    (v_entity, v_transform): (View<Entity>, View<Transform>),
    v_hitbox: View<Hitbox>,
    mut vm_inventory: ViewMut<PlayerInventory>,

    (mut entities, mut vm_block_update_evts): (EntitiesViewMut, ViewMut<BlockUpdateEvent>)
) {
    let (_, look_at, inventory) = (&v_local_player, &v_looking_at_block, &mut vm_inventory).iter()
        .next()
        .expect("local player didn't have LookingAtBlock & HeldBlock");
    
    let Some(RaycastResult {
        hit: RaycastHit::Block {
            location,
            face,
        },
        ..
    }) = &look_at.0 else {
        return;
    };

    let mut should_place = input.just_pressed().get_action(Action::PlaceBlock);
    let mut should_break = input.just_pressed().get_action(Action::BreakBlock);

    if last_world_interaction.cooldown_passed() {
        should_place |= input.pressed().get_action(Action::PlaceBlock);
        should_break |= input.pressed().get_action(Action::BreakBlock);
    }

    let mut update_block = |world: &mut ChunkManager, pos: BlockLocation, block: Block, inventory: &mut PlayerInventory| {
        last_world_interaction.reset_cooldown();

        entities.add_entity(&mut vm_block_update_evts, BlockUpdateEvent(pos.clone(), block.clone()));
        
        let old = world.modify_block(&pos, block).expect("chunk shouldn't have unloaded so quickly");

        let drops = old.on_break();

        // TODO: deal with overflow
        let _residual = inventory.try_insert_many(drops);
    };

    if should_place && should_break {
        if let Some(ft) = face {
            let block = inventory.try_get_place_at(held.0, location.clone(), *ft).unwrap_or(Block::Air);

            update_block(&mut chunk_mgr, location.clone(), block, inventory);
        }
    } else if should_break {
        update_block(&mut chunk_mgr, location.clone(), Block::Air, inventory);
    } else if should_place {
        if let Some(ft) = face {
            // TODO: impl Add<IVec3> for BlockLocation
            let adj = BlockLocation(location.0 + ft.as_vector());

            if chunk_mgr.get_block_ref(&adj) == Some(&Block::Air) {
                let (min, max) = adj.get_aabb_bounds();

                if collision::collides_with_any_entity(min, max, v_entity, v_transform, v_hitbox).is_none() {
                    if let Some(block) = inventory.try_get_place_at(held.0, adj.clone(), *ft) {
                        update_block(&mut chunk_mgr, adj, block, inventory);
                    }
                }
            }
        }
    }
}

pub fn spawn_multiplayer_player(
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
                    position: Vec3::new(0.5, 60.0, 0.5),
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

pub fn get_generated_chunks(world_gen: UniqueView<WorldGenerator>, mut vm_entities: EntitiesViewMut, vm_chunk_gen_evt: ViewMut<ChunkGenEvent>) {
    let chunks = world_gen.receive_chunks();

    drop(world_gen);

    if !chunks.is_empty() {
        vm_entities.bulk_add_entity(vm_chunk_gen_evt, chunks);
    }
}


pub fn generate_chunks(mut reqs: ViewMut<ChunkGenRequestEvent>, world_generator: UniqueView<WorldGenerator>, mut world_saver: UniqueViewMut<WorldSaver>) {
    for req in reqs.drain() {
        if let Some(cache) = world_saver.try_get(&req.0) {
            world_generator.send(cache.data);
        } else {
            world_generator.spawn_generate_task(req.0, world_generator.splines.clone(), world_generator.params.clone());
        }
    }
}
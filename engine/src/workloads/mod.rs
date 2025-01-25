use shipyard::{IntoWorkload, IntoWorkloadTrySystem, SystemModificator, Workload};
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase, EnginePluginMetadata};
use strck::IntoCk;
use dino_plugins::{path, Identifiable};
use crate::{args, rendering};
use crate::chunks::chunk_manager::chunk_manager_update_and_request;
use crate::environment::{is_hosted, is_multiplayer_client};
use crate::gamemode::local_player_is_gamemode_spectator;
use crate::input::reset_mouse_manager_state;
use crate::networking::{client_acknowledge_connection_success, client_handle_kicked_by_server, client_request_chunks_from_server, client_send_block_updates, client_send_settings, client_update_position, server_broadcast_block_updates, server_broadcast_chunks, server_handle_client_chunk_reqs, server_process_client_connection_req, server_process_render_dist_update, server_request_client_settings, server_update_client_transform};
use crate::networking::keep_alive::server_send_keep_alive;
use crate::physics::movement::{adjust_spectator_fly_speed, apply_camera_input, process_movement};
use crate::physics::process_physics;
use crate::rendering::block_outline::update_block_outline_buffer;
use crate::rendering::camera_uniform_buffer::update_camera_uniform_buffer;
use crate::rendering::render;
use crate::rendering::render::{block_outline, submit_rendered_frame, world};
use crate::workloads::shutdown::disconnect_connected_players;
use crate::workloads::startup::{initialize_gameplay_systems, initialize_local_player, initialize_networking, set_window_title};
use crate::workloads::update::{client_apply_block_updates, generate_chunks, get_generated_chunks, place_break_blocks, raycast, scroll_hotbar, server_apply_block_updates, spawn_multiplayer_player, toggle_gamemode};

mod startup;
mod update;
mod shutdown;

// TODO: fix visibility with all systems for this plugin

pub struct VoxelEngine;

impl DinoEnginePlugin for VoxelEngine {
    fn early_startup(&self) -> Option<Workload> {
        (
            args::parse_env,
            rendering::initialize,
            initialize_local_player,
        )
            .into_sequential_workload()
            .into()
    }

    fn late_startup(&self) -> Option<Workload> {
        (
            initialize_gameplay_systems,
            initialize_networking,
            set_window_title,
        )
            .into_sequential_workload()
            .into()
    }

    fn input(&self) -> Option<Workload> {
        (
            apply_camera_input,
            process_movement,
            toggle_gamemode,
            adjust_spectator_fly_speed.run_if(local_player_is_gamemode_spectator),
            scroll_hotbar.skip_if(local_player_is_gamemode_spectator),
        )
            .into_sequential_workload()
            .into()
    }

    fn early_update(&self) -> Option<Workload> {
        (
            process_physics,
            reset_mouse_manager_state,
            get_generated_chunks.run_if(is_hosted),
        )
            .into_sequential_workload()
            .into()
    }

    fn networking_client_pre_recv(&self) -> Option<Workload> {
        (
            client_send_block_updates,
        ).into_workload()
            .into()
    }

    fn networking_client_post_recv(&self) -> Option<Workload> {
        (
            client_handle_kicked_by_server,
            client_acknowledge_connection_success,
            client_update_position,
            client_request_chunks_from_server,
            client_send_settings,
        ).into_workload()
            .into()
    }

    fn networking_server_pre_recv(&self) -> Option<Workload> {
        let path = path!({self}::{ EnginePhase::NetworkingServerPreRecv });

        (
            move || tracing::trace!("{path}"),
        )
            .into_workload()
            .into()
    }

    fn networking_server_post_recv(&self) -> Option<Workload> {
        (
            server_broadcast_chunks,
            server_broadcast_block_updates,
            server_process_client_connection_req,
            server_update_client_transform,
            server_request_client_settings,
            server_process_render_dist_update,
            server_handle_client_chunk_reqs,
            server_send_keep_alive,
        ).into_workload()
            .into()
    }

    fn late_update(&self) -> Option<Workload> {
        (
            chunk_manager_update_and_request,
            generate_chunks.run_if(is_hosted),
            server_apply_block_updates.run_if(is_hosted),
            client_apply_block_updates.run_if(is_multiplayer_client),
            spawn_multiplayer_player,
            raycast.skip_if(local_player_is_gamemode_spectator),
            place_break_blocks.skip_if(local_player_is_gamemode_spectator),
        ).into_sequential_workload()
            .into()
    }

    fn pre_render(&self) -> Option<Workload> {
        (
            // -- PRE RENDER -- //
            update_block_outline_buffer,
            update_camera_uniform_buffer,
            render::create_new_render_context
                .into_workload_try_system()
                .expect("failed to convert to try_system?"),
        ).into_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        let plugin = self.identifier();

        (
            // -- RENDER -- //
            world::render_world.tag(path!({plugin}::{EnginePhase::Render}::render_world)),
            block_outline::render_block_outline,
            render::egui::render_egui.tag(path!({plugin}::{EnginePhase::Render}::render_egui)), // -- RENDER UI -- //
            submit_rendered_frame.tag(path!({plugin}::{EnginePhase::Render}::submit_rendered_frame)),
        ).into_sequential_workload()
            .into()
    }

    fn post_render(&self) -> Option<Workload> {
        let path = path!({self}::{ EnginePhase::PostRender });

        (
            move || tracing::trace!("{path}"),
        )
            .into_workload()
            .into()
    }

    fn shutdown(&self) -> Option<Workload> {
        (
            // -- SHUTDOWN -- //
            disconnect_connected_players.run_if(is_hosted),
        ).into_sequential_workload()
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "voxel_engine".ck().expect("valid name"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[],
        }
    }
}
use egui::{Color32, LayerId};
use shipyard::{AllStoragesView, IntoWorkload, UniqueView, Workload, WorkloadModificator};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase, EnginePluginMetadata};
use dino_plugins::path;
use egui_systems::{CurrentEguiFrame, EguiSystemsPlugin};
use egui_systems::DuringEgui;
use block_bar::block_bar;
use engine::gamemode::local_player_is_gamemode_spectator;
use engine::VoxelEngine;
use crate::block_bar::{create_block_bar_display, scroll_block_bar};
use crate::bottom_bar::bottom_bar;
use crate::egui_views::initialize_texture_atlas_views;
use shipyard::SystemModificator;
use crate::debug::debug_ui;
use crate::inventory::{inventory, toggle_inv_block_bar, InventoryOpen};
use crate::inventory::hand::{render_hand, InventoryHand};

extern crate nalgebra_glm as glm;

mod bottom_bar;
mod egui_views;
mod block_bar;
mod inventory;
pub mod gui_bundle;
pub(crate) mod item_stack;
mod debug;

pub struct GameUiPlugin;

impl DinoEnginePlugin for GameUiPlugin {
    fn early_startup(&self) -> Option<Workload> {
        (
            initialize_texture_atlas_views
                .after_all(path!({EguiSystemsPlugin}::{EnginePhase::EarlyStartup}::initialize_renderer)),
            inventory::initialize,
        )
            .into_workload()
            .into()
    }

    fn late_startup(&self) -> Option<Workload> {
        create_block_bar_display
            .into_workload()
            .after_all(path!({VoxelEngine}::{EnginePhase::LateStartup}::initialize_gameplay_systems))
            .into()
    }

    fn input(&self) -> Option<Workload> {
        (
            scroll_block_bar.skip_if(local_player_is_gamemode_spectator),
            toggle_inv_block_bar,
        )
            .into_sequential_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        (
            reticle,
            bottom_bar,
            block_bar,
            inventory,
            debug_ui,
            render_hand,
        )
            .into_workload()
            .order_egui()
            .into()
    }
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "game_ui".ck().expect("valid identifier"),
            version: "0.1.0",
            dependencies: &[
                &VoxelEngine,
                &EguiSystemsPlugin,
            ],
        }
    }
}

fn reticle(egui_frame: UniqueView<CurrentEguiFrame>) {
    let ctx = egui_frame.ctx();

    ctx.layer_painter(LayerId::background())
        .circle_filled(
            ctx.screen_rect().center(),
            2.5,
            Color32::from_rgba_premultiplied(192, 192, 192, 128),
        );
}
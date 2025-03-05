use egui::Color32;
use shipyard::{IntoIter, IntoWorkload, UniqueView, Workload, WorkloadModificator};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePhase, EnginePluginMetadata};
use dino_plugins::path;
use egui_systems::{CurrentEguiFrame, EguiSystemsPlugin};
use egui_systems::DuringEgui;
use block_bar::block_bar;
use engine::gamemode::local_player_is_gamemode_spectator;
use engine::VoxelEngine;
use crate::block_bar::{create_block_bar_display, process_block_bar, scroll_block_bar};
use crate::bottom_bar::bottom_bar;
use crate::egui_views::initialize_texture_atlas_views;
use shipyard::SystemModificator;

extern crate nalgebra_glm as glm;

mod bottom_bar;
mod egui_views;
mod block_bar;

pub struct GameUiPlugin;

impl DinoEnginePlugin for GameUiPlugin {
    fn early_startup(&self) -> Option<Workload> {
        initialize_texture_atlas_views
            .into_workload()
            .after_all(path!({EguiSystemsPlugin}::{EnginePhase::EarlyStartup}::initialize_renderer))
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
            process_block_bar,
        )
            .into_sequential_workload()
            .into()
    }

    fn render(&self) -> Option<Workload> {
        (
            reticle,
            bottom_bar,
            block_bar,
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
                &engine::VoxelEngine,
                &egui_systems::EguiSystemsPlugin,
            ],
        }
    }
}

fn reticle(egui_frame: UniqueView<CurrentEguiFrame>) {
    let ctx = egui_frame.ctx();

    ctx.layer_painter(egui::LayerId::background())
        .circle_filled(
            ctx.screen_rect().center(),
            2.5,
            Color32::from_rgba_premultiplied(192, 192, 192, 128),
        );
}
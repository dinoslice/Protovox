use egui::Color32;
use shipyard::{IntoIter, IntoWorkload, UniqueView, Workload};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use egui_systems::CurrentEguiFrame;
use egui_systems::DuringEgui;
use game::block::Block;

extern crate nalgebra_glm as glm;

pub struct GameUiPlugin;

impl DinoEnginePlugin for GameUiPlugin {
    fn render(&self) -> Option<Workload> {
        (
            reticle,
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
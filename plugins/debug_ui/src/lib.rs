use shipyard::{IntoWorkload, UniqueView, Workload};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use egui_systems::{CurrentEguiFrame, DuringEgui, EguiSystemsPlugin};
use engine::networking::server_handler::ServerHandler;
use engine::VoxelEngine;

pub struct DebugUiPlugin;

impl DinoEnginePlugin for DebugUiPlugin {
    fn render(&self) -> Option<Workload> {
        egui
            .into_workload()
            .order_egui()
            .into()
    }
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "debug_ui".ck().expect("valid identifier"),
            version: "0.1.0",
            dependencies: &[&VoxelEngine, &EguiSystemsPlugin],
        }
    }
}

fn egui(
    egui_frame: UniqueView<CurrentEguiFrame>,

    opt_server_handler: Option<UniqueView<ServerHandler>>,
) {
    if let Some(server_handler) = opt_server_handler {
        egui::Window::new("Debug")
            .show(egui_frame.ctx(), |ui| {
                ui.add(server_handler.as_ref())
            });
    }
}
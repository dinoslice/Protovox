use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use egui_systems::EguiSystemsPlugin;
use engine::VoxelEngine;

pub struct DebugUiPlugin;

impl DinoEnginePlugin for DebugUiPlugin {
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "debug_ui".ck().expect("valid identifier"),
            version: "0.1.0",
            dependencies: &[&VoxelEngine, &EguiSystemsPlugin],
        }
    }
}
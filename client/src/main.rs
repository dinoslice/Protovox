use debug_ui::DebugUiPlugin;
use egui_systems::EguiSystemsPlugin;
use engine::application::plugin_manager::PluginManager;
use engine::VoxelEngine;
use game_ui::GameUiPlugin;
use gizmos::GizmosPlugin;
use visual_debug::VisualDebugPlugin;

fn main() {
    client::init_tracing().expect("tracing initialized");

    client::run(
        PluginManager::new()
            .with(&VoxelEngine)
            .with(&GizmosPlugin)
            .with(&VisualDebugPlugin)
            .with(&EguiSystemsPlugin)
            .with(&GameUiPlugin)
            .with(&DebugUiPlugin)
    );
}
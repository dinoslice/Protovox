use engine::application::plugin_manager::PluginManager;
use engine::VoxelEngine;
use gizmos::GizmosPlugin;
use visual_debug::VisualDebugPlugin;

fn main() {
    client::init_tracing().expect("tracing initialized");

    client::run(
        PluginManager::new()
            .with(&VoxelEngine)
            .with(&GizmosPlugin)
            .with(&VisualDebugPlugin)
    );
}
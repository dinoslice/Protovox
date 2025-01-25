use engine::application::plugin_manager::PluginManager;
use engine::VoxelEngine;

fn main() {
    client::init_tracing().expect("tracing initialized");

    client::run(
        PluginManager::new()
            .with(&VoxelEngine)
    );
}
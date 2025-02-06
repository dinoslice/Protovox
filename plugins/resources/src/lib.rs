mod resource_type;
mod key;
mod registry;

use shipyard::{AllStoragesView, IntoWorkload, Workload};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
pub use registry::Registry;
pub use key::ResourceKey;
pub use registry::test;

#[cfg(feature = "custom_types")]
pub use resource_type::ResourceType;

fn create_registry(storages: AllStoragesView) {
    storages.add_unique(Registry::default());
}

pub struct ResourcePlugin;

impl DinoEnginePlugin for ResourcePlugin {
    fn early_startup(&self) -> Option<Workload> {
        create_registry
            .into_sequential_workload()
            .into()
    }

    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "resources".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[]
        }
    }
}
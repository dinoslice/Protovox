mod r#type;
mod key;

#[cfg(feature = "custom_types")]
mod registry;

use shipyard::{AllStoragesView, IntoWorkload, Workload, WorkloadModificator};
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
pub use registry::Registry;
pub use key::ResourceKey;

#[cfg(feature = "custom_types")]
pub use r#type::ResourceType;

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
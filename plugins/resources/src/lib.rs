mod resource_type;
mod key;
mod registry;

use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
pub use key::ResourceKey;
pub use registry::Registry;
use strck::IntoCk;

#[cfg(feature = "custom_types")]
pub use resource_type::ResourceType;

pub struct ResourcePlugin;

impl DinoEnginePlugin for ResourcePlugin {
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "resources".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[]
        }
    }
}
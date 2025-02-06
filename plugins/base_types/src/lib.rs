mod block;
mod texture;
mod item;

use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};
use resources::ResourcePlugin;

pub use texture::*;
pub use proc_types;

pub struct BaseTypesPlugins;

impl DinoEnginePlugin for BaseTypesPlugins {
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "base_types".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
            dependencies: &[&ResourcePlugin]
        }
    }
}
use strck::IntoCk;
use dino_plugins::engine::{DinoEnginePlugin, EnginePluginMetadata};

pub struct GizmosPlugin;

impl DinoEnginePlugin for GizmosPlugin {
    fn plugin_metadata(&self) -> EnginePluginMetadata {
        EnginePluginMetadata {
            name: "gizmos".ck().expect("valid identifier"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
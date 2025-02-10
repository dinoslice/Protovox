use std::sync::LazyLock;
use resources::ResourceKey;
use crate::base_types::block::Block;
use crate::base_types::texture::Texture;

pub mod block;
pub mod texture;

macro_rules! generate_keys {
    ($($resource:ident $name:ident = $gen:expr),* $(,)?) => {
        $(
            pub const $name: LazyLock<ResourceKey<$resource>> = LazyLock::new(|| $gen);
        )*
    };
}

generate_keys!(
    Block COBBLESTONE = ResourceKey::new("engine", "cobblestone"),
    Texture COBBLESTONE_T = ResourceKey::new("engine", "cobblestone"),
    Block DIRT = ResourceKey::new("engine", "dirt"),
    Texture DIRT_T = ResourceKey::new("engine", "dirt"),
);
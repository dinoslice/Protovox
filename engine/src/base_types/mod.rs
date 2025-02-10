use std::sync::LazyLock;
use resources::ResourceKey;
use crate::base_types::block::Block;
use crate::base_types::texture::Texture;
use resources::Registry;
use crate::model;

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

pub fn test() {
    let block = model!("../../assets/model/cobblestone.json");
    let mut registry = Registry::default();

    registry.register(COBBLESTONE.clone(), block.clone());
    registry.register(ResourceKey::new("engine", "cube"), model!("../../assets/model/cube.json"));
    registry.register(ResourceKey::new("engine", "cube_all"), model!("../../assets/model/cube_all.json"));

    let tex = block.get_texture(game::block::face_type::FaceType::Front, &mut registry);
    println!("{tex}");
}
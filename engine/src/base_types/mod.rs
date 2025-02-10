use std::sync::LazyLock;
use shipyard::UniqueViewMut;
use resources::ResourceKey;
use crate::base_types::block::Block;
use crate::base_types::texture::Texture;
use resources::Registry;
use crate::{model, texture};
use crate::game::face_type::FaceType;

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

    Block AIR = ResourceKey::new("engine", "air"),
    Texture SELECTION_OUTLINE = ResourceKey::new("engine", "selection"),

    Block DEBUG = ResourceKey::new("engine", "debug"),
    Texture DEBUG_BLUE = ResourceKey::new("engine", "debug_blue"),
    Texture DEBUG_RED = ResourceKey::new("engine", "debug_red"),
    Texture DEBUG_GREEN = ResourceKey::new("engine", "debug_green"),

    Block STONE = ResourceKey::new("engine", "stone"),
    Texture STONE_T = ResourceKey::new("engine", "stone"),

    Block GRASS = ResourceKey::new("engine", "grass"),
    Texture GRASS_SIDE = ResourceKey::new("engine", "grass_side"),
    Texture GRASS_T = ResourceKey::new("engine", "grass"),
);

pub fn register_engine_resources(mut registry: UniqueViewMut<Registry>) {
    registry.register(COBBLESTONE.clone(), model!("../../assets/model/cobblestone.json"));
    registry.register(ResourceKey::new("engine", "cube"), model!("../../assets/model/cube.json"));
    registry.register(ResourceKey::new("engine", "cube_all"), model!("../../assets/model/cube_all.json"));
    registry.register(SELECTION_OUTLINE.clone(), texture!("../../assets/texture/selection.png"));

    registry.register(DEBUG.clone(), model!("../../assets/model/debug.json"));
    registry.register(DEBUG_BLUE.clone(), texture!("../../assets/texture/debug_blue.png"));
    registry.register(DEBUG_RED.clone(), texture!("../../assets/texture/debug_red.png"));
    registry.register(DEBUG_GREEN.clone(), texture!("../../assets/texture/debug_green.png"));

    registry.register(STONE.clone(), model!("../../assets/model/stone.json"));
    registry.register(STONE_T.clone(), texture!("../../assets/texture/stone.png"));

    registry.register(GRASS.clone(), model!("../../assets/model/grass.json"));
    registry.register(GRASS_SIDE.clone(), texture!("../../assets/texture/grass_side.png"));
    registry.register(GRASS_T.clone(), texture!("../../assets/texture/grass.png"));
}

pub fn test() {
    let block = model!("../../assets/model/cobblestone.json");
    let mut registry = Registry::default();

    registry.register(COBBLESTONE.clone(), block.clone());
    registry.register(ResourceKey::new("engine", "cube"), model!("../../assets/model/cube.json"));
    registry.register(ResourceKey::new("engine", "cube_all"), model!("../../assets/model/cube_all.json"));

    let tex = block.get_texture(FaceType::Front, &mut registry);
    println!("{tex}");
}
use proc_types::block;
use resources::{Registry, ResourceKey};
use crate::block::Block;

mod block;
mod item;
mod texture;

pub use texture::Texture;

pub fn main() {
    let mut registry = Registry::default();
    registry.register(ResourceKey::new("base", "test"), block!("assets/blocks/test.json"));
    registry.register(ResourceKey::new("base", "test"), texture!("../assets/textures/test.png"));

    for (key, block) in registry.iter::<Block>() {
        println!("{} {}", key, block.texture);
        let texture = registry.get(&block.texture).unwrap();
        println!("{:?}", texture.image);
    }
}
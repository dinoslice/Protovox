use serde::Deserialize;
use proc_types::block_parse;
use resources::{ResourceKey, ResourceType};
use crate::Texture;

pub struct Block {
    texture: ResourceKey<Texture>,
}

impl ResourceType for Block {
    fn resource_name() -> &'static str {
        "block"
    }
}

const Test: Block = block!("assets/blocks/test.json");
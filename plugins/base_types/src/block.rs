use serde::Deserialize;
use proc_types::{block};
use resources::{ResourceKey, ResourceType};
use crate::{texture, Texture};

pub struct Block {
    pub texture: ResourceKey<Texture>,
}

impl ResourceType for Block {
    fn resource_name() -> &'static str {
        "block"
    }
}
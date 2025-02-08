use std::collections::HashMap;
use serde::Deserialize;
use shipyard::UniqueViewMut;
use resources::{Registry, ResourceKey, ResourceType};
use crate::base_types::texture::Texture;

#[derive(Debug)]
pub enum ModelTextureType {
    ChildDefined(String),
    TextureResource(ResourceKey<Texture>),
}

impl Into<ModelTextureType> for String {
    fn into(self) -> ModelTextureType {
        let key = ResourceKey::try_from(&self);

        if let Ok(key) = key {
            ModelTextureType::TextureResource(key)
        } else {
            if !self.starts_with("#") {
                panic!("ChildDefined texture name must start with a #")
            }
            ModelTextureType::ChildDefined(self.replace("#", ""))
        }
    }
}

#[derive(Debug)]
pub struct Block {
    parent: Option<ResourceKey<Block>>,
    elements: Option<Vec<ModelElement>>,
    textures: Option<HashMap<String, ModelTextureType>>,
}

impl Block {
    pub fn can_be_used(&self) -> bool {
        if let Some(textures) = &self.textures {
            for (_, texture) in textures.iter() {
                if let ModelTextureType::ChildDefined(_) = texture {
                    return false;
                }
            }
        }

        if let Some(elements) = &self.elements {
            for element in elements {
                if !element.can_be_used() {
                    return false;
                }
            }
        }

        true
    }
}

impl ResourceType for Block {
    fn resource_name() -> &'static str {
        "model"
    }

    fn is_valid(&self, registry: &mut Registry) -> bool {
        // TODO: check the parent thing if each of the keys for the textures match to a part of the parent/parent's parent/so on
        true
    }
}

#[derive(Debug)]
pub struct ModelElement {
    from: [f32; 3],
    to: [f32; 3],
    front: Option<ModelElementTexture>,
    back: Option<ModelElementTexture>,
    left: Option<ModelElementTexture>,
    right: Option<ModelElementTexture>,
    top: Option<ModelElementTexture>,
    bottom: Option<ModelElementTexture>,
}

impl ModelElement {
    fn can_be_used(&self) -> bool {
        let checks = [&self.front, &self.back, &self.left, &self.right, &self.top, &self.bottom];

        for check in checks {
            if let Some(texture) = check {
                if !texture.can_be_used() {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct ModelElementTexture {
    uv: Option<[u32; 4]>,
    texture: ModelTextureType,
}

impl ModelElementTexture {
    fn can_be_used(&self) -> bool {
        if let ModelTextureType::ChildDefined(_) = self.texture {
            false
        } else {
            true
        }
    }
}

#[derive(Deserialize)]
struct ModelJson {
    parent: Option<String>,
    elements: Option<Vec<ModelElementJson>>,
    textures: Option<HashMap<String, String>>
}

impl Into<Block> for ModelJson {
    fn into(self) -> Block {
        Block {
            parent: self.parent.and_then(|parent| Some(ResourceKey::try_from(&parent).expect("Failed to parse parent ResourceKey for model!"))),
            elements: self.elements.and_then(|elements| Some(elements.into_iter().map(|element| element.into()).collect())),
            textures: self.textures.and_then(|map| Some(map.into_iter().map(|(key, value)| (key, value.into())).collect()))
        }
    }
}

#[derive(Deserialize)]
struct ModelElementJson {
    from: [f32; 3],
    to: [f32; 3],
    front: Option<ModelElementTextureJson>,
    back: Option<ModelElementTextureJson>,
    left: Option<ModelElementTextureJson>,
    right: Option<ModelElementTextureJson>,
    top: Option<ModelElementTextureJson>,
    bottom: Option<ModelElementTextureJson>,
}

impl Into<ModelElement> for ModelElementJson {
    fn into(self) -> ModelElement {
        ModelElement {
            from: self.from,
            to: self.to,
            front: self.front.and_then(|front| Some(front.into())),
            back: self.back.and_then(|back| Some(back.into())),
            left: self.left.and_then(|left| Some(left.into())),
            right: self.right.and_then(|right| Some(right.into())),
            top: self.top.and_then(|top| Some(top.into())),
            bottom: self.bottom.and_then(|bottom| Some(bottom.into())),
        }
    }
}

#[derive(Deserialize)]
struct ModelElementTextureJson {
    uv: Option<[u32; 4]>,
    texture: String,
}

impl Into<ModelElementTexture> for ModelElementTextureJson {
    fn into(self) -> ModelElementTexture {
        ModelElementTexture {
            uv: self.uv,
            texture: self.texture.into()
        }
    }
}

macro_rules! model {
    ($path:expr) => {
        {
            use $crate::base_types::block::*;

            let str = include_str!($path);
            let model = serde_json::from_str::<ModelJson>(str).expect("Failed to parse json");

            <ModelJson as Into<Block>>::into(model)
        }
    };
}

pub fn model(mut registry: UniqueViewMut<Registry>) {
    registry.register(ResourceKey::new("engine", "cobblestone"), model!("../../assets/model/cobblestone.json"));
    registry.register(ResourceKey::new("engine", "cube"), model!("../../assets/model/cube.json"));
    registry.register(ResourceKey::new("engine", "cube_all"), model!("../../assets/model/cube_all.json"));
}
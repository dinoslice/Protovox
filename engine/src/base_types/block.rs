use std::collections::HashMap;
use serde::Deserialize;
use shipyard::UniqueViewMut;
use resources::{Registry, ResourceKey, ResourceType};
use crate::base_types::{COBBLESTONE, COBBLESTONE_T, DIRT, DIRT_T};
use crate::base_types::texture::Texture;
use crate::game::face_type::FaceType;
use crate::texture;

#[derive(Debug, Clone)]
pub enum ModelTextureType {
    ChildDefined(String),
    TextureResource(ResourceKey<Texture>),
}

impl Default for ModelTextureType {
    fn default() -> Self {
        ModelTextureType::TextureResource(ResourceKey::default())
    }
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

#[derive(Debug, Clone, Default)]
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

    pub fn get_texture(&self, face: FaceType, registry: &Registry) -> ResourceKey<Texture> {
        if !self.can_be_used() {
            return ResourceKey::new("engine", "cobblestone");
        }

        if let Some(parent) = &self.parent {
            let parent = registry.get(parent).expect("Parent was not registered!");

            parent.get_texture_inner(face, self.textures.as_ref().unwrap(), registry)
        } else {
            ResourceKey::new("engine", "cobblestone")
        }
    }

    fn get_texture_inner(&self, face: FaceType, prev_textures: &HashMap<String, ModelTextureType>, registry: &Registry) -> ResourceKey<Texture> {
        if let Some(textures) = &self.textures {
            let mut new_textures = HashMap::new();
            for (key, value) in textures {
                match value {
                    ModelTextureType::ChildDefined(k) => {
                        new_textures.insert(key.clone(), prev_textures.get(k).expect("Failed to get the texture for model key!").clone());
                    },
                    ModelTextureType::TextureResource(tex) => {
                        new_textures.insert(key.clone(), ModelTextureType::TextureResource(tex.clone()));
                    }
                }
            }
            let parent = self.parent.as_ref().expect("Failed to find parent for model");
            registry.get_unchecked(parent).get_texture_inner(face, &new_textures, registry)
        } else if let Some(elements) = &self.elements {
            let tex = match face {
                FaceType::Front => elements[0].front.as_ref().expect("failed to get front texture"),
                FaceType::Back => elements[0].back.as_ref().expect("failed to get back texture"),
                FaceType::Left => elements[0].left.as_ref().expect("failed to get left texture"),
                FaceType::Right => elements[0].front.as_ref().expect("failed to get right texture"),
                FaceType::Top => elements[0].top.as_ref().expect("failed to get top texture"),
                FaceType::Bottom => elements[0].front.as_ref().expect("failed to get bottom texture"),
            };

            if let ModelTextureType::ChildDefined(key) = &tex.texture {
                let tex = prev_textures.get(key).expect("failed to get key");
                if let ModelTextureType::TextureResource(key) = tex {
                    key.clone()
                } else {
                    panic!("failed to find texture")
                }
            } else if let ModelTextureType::TextureResource(tex) = &tex.texture {
                tex.clone()
            } else {
                ResourceKey::new("engine", "cobblestone")
            }
        } else {
            panic!("failed to find texture")
        }
    }
}

impl ResourceType for Block {
    fn resource_name() -> &'static str {
        "model"
    }

    fn is_valid(&self, _registry: &mut Registry) -> bool {
        // TODO: check the parent thing if each of the keys for the textures match to a part of the parent/parent's parent/so on
        true
    }

    fn default_resource() -> ResourceKey<Self>
    where
        Self: Sized,
    {
        ResourceKey::new("engine", "cobblestone")
    }
}

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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
pub struct ModelJson {
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
pub struct ModelElementJson {
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
pub struct ModelElementTextureJson {
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

#[macro_export]
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
    registry.register(COBBLESTONE_T.clone(), texture!("../../assets/texture/cobblestone.png"));
    registry.register(COBBLESTONE.clone(), model!("../../assets/model/cobblestone.json"));

    registry.register(DIRT_T.clone(), texture!("../../assets/texture/dirt.png"));
    registry.register(DIRT.clone(), model!("../../assets/model/cobblestone.json"));

    registry.register(ResourceKey::new("engine", "cube"), model!("../../assets/model/cube.json"));
    registry.register(ResourceKey::new("engine", "cube_all"), model!("../../assets/model/cube_all.json"));
}
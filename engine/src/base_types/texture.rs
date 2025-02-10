use image::{load_from_memory, DynamicImage};
use resources::ResourceType;

#[derive(Clone)]
pub struct Texture {
    pub atlas_id: usize,
    pub image: DynamicImage,
}

impl ResourceType for Texture {
    fn resource_name() -> &'static str {
        "texture"
    }
}

#[macro_export]
macro_rules! texture {
    ($asset:expr) => {
        {
            use $crate::base_types::texture::*;
            use image::{DynamicImage, load_from_memory};

            Texture {
                atlas_id: 0,
                image: load_from_memory(include_bytes!($asset)).expect(format!("Failed to load asset {}", stringify!($asset)).as_str())
            }
        }
    };
}
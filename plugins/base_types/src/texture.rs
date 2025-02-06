use image::{load_from_memory, DynamicImage, RgbImage};
use resources::ResourceType;

#[macro_export]
macro_rules! texture {
    ($path:expr) => {
        $crate::texture::Texture {
            atlas_id: 0,
            image: image::load_from_memory(include_bytes!($path)).unwrap(),
        }
    };
}

pub struct Texture {
    pub atlas_id: usize,
    pub image: DynamicImage,
}

impl ResourceType for Texture {
    fn resource_name() -> &'static str {
        "texture"
    }
}
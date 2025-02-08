use image::DynamicImage;
use resources::ResourceType;

pub struct Texture {
    pub atlas_id: usize,
    pub image: DynamicImage,
}

impl ResourceType for Texture {
    fn resource_name() -> &'static str {
        "texture"
    }
}
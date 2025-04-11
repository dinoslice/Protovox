use std::num::NonZeroU32;
use hashbrown::HashMap;
use shipyard::{AllStoragesView, Unique, UniqueView};
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension};
use crate::entity::model::Model;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::texture::Texture;

#[derive(Unique)]
pub struct ModelMap {
    pub map: HashMap<String, (Model, usize)>,

    pub tex: Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

const MODELS: &[(&'static str, &'static str)] = &[
    ("toy_car", "engine/assets/entity/toy_car/config.gltf"),
];

pub fn initialize(storages: AllStoragesView, g_ctx: UniqueView<GraphicsContext>) {
    let mut model_map = HashMap::new();
    let mut textures = Vec::new();
    for (name, path) in MODELS.iter() {
        let mut model = Model::open(path);
        let offset = textures.len();

        textures.append(&mut model.texture);

        model_map.insert(name.to_string(), (model, offset));
    }

    let texture = Texture::from_images_2d(&g_ctx.device, &g_ctx.queue, &textures, None)
        .expect("failed to build texture");

    let bind_group_layout = g_ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: NonZeroU32::new(textures.len() as u32),
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            }
        ],
    });

    let bind_group = g_ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture.view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&texture.sampler),
            }
        ],
    });

    storages.add_unique(ModelMap { map: model_map, tex: texture, bind_group, bind_group_layout });
}
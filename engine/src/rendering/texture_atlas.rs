use std::num::NonZeroU32;
use shipyard::{AllStoragesView, Unique, UniqueView, UniqueViewMut};
use resources::Registry;
use crate::rendering::texture::Texture;
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Unique)]
pub struct TextureAtlas {
    #[allow(dead_code)]
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub fn initialize_texture_atlas(mut registry: UniqueViewMut<Registry>, g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let mut images = Vec::new();
    let mut id = 0;

    for (_, texture) in registry.iter_mut::<crate::base_types::texture::Texture>() {
        images.push(&texture.image);
        texture.atlas_id = id;
        id += 1;
    }

    let texture = Texture::from_images(&g_ctx.device, &g_ctx.queue, &images, Some("texture_atlas"));

    // bind group -> data constant through one draw call
    let bind_group_layout =
        g_ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, // corresponds to @binding(n) in the shader
                    visibility: wgpu::ShaderStages::FRAGMENT, // use this bind group in the fragment shader
                    ty: wgpu::BindingType::Texture { // it's a texture, texture_2d<f32> in shader
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array, // _2d
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: Some(NonZeroU32::new(images.len().max(1) as u32).unwrap()),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // sampler in shader
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // sample type for texture must be filterable
                    count: None, // not an array
                },
            ],
            label: Some("Texture Atlas Bind Group Layout"),
        });

    let bind_group = g_ctx.device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &bind_group_layout, // layout defined above
            entries: &[ // matches the entries defined above
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view), // assign the data into the bind group
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                }
            ],
            label: Some("Texture Atlas Bind Group"),
        }
    );

    storages.add_unique(TextureAtlas { texture, bind_group, bind_group_layout });
}
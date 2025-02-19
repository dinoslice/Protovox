use std::fs;
use std::num::NonZeroU32;
use shipyard::{AllStoragesView, Unique, UniqueView};
use crate::rendering::texture::Texture;
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Unique)]
pub struct TextureAtlas {
    pub texture_atlas: Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub num_textures: usize,
}

pub fn initialize_texture_atlas(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    // 4. load textures into bind group

    let textures = [
        "cobblestone",
        "dirt",
        "grass",
        "grass_side",
        "selection",
        "debug_red",
        "debug_green",
        "debug_blue",
        "log",
    ];

    let loaded_textures = textures.map(|key| {
        // TODO: pack textures into binary or better loading?
        let path = format!("engine/assets/blocks/{key}.png");

        let bytes = fs::read(path).expect("TODO: better error; file to exist");

        image::load_from_memory(&bytes).expect("TODO: better error; valid image")
    });

    let texture_atlas = Texture::from_images_2d(&g_ctx.device, &g_ctx.queue, &loaded_textures, Some("texture_atlas"))
        .expect("TODO: better errors; valid textures");

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
                    count: Some(NonZeroU32::new(loaded_textures.len() as _).expect("nonzero amount"))
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
                    resource: wgpu::BindingResource::TextureView(&texture_atlas.view), // assign the data into the bind group
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_atlas.sampler),
                }
            ],
            label: Some("Texture Atlas Bind Group"),
        }
    );

    storages.add_unique(TextureAtlas { texture_atlas, bind_group, bind_group_layout, num_textures: loaded_textures.len() });
}
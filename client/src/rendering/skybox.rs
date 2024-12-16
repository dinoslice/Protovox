use std::fs;
use shipyard::{AllStoragesView, Unique, UniqueView};
use wgpu::{include_wgsl, BindGroupDescriptor, RenderPipeline};
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::texture::Texture;

#[derive(Unique)]
pub struct SkyboxRenderBundle {
    pub texture: Texture,
    pub render_pipeline: RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub fn initialize_skybox(g_ctx: UniqueView<GraphicsContext>, camera_uniform_buffer: UniqueView<CameraUniformBuffer>, storages: AllStoragesView) {
    let shader = g_ctx.device.create_shader_module(include_wgsl!("shaders/skybox.wgsl"));

    let cube_faces = ["px", "nx", "py", "ny", "pz", "nz"];

    let loaded_textures = cube_faces.map(|face| { // TODO: abstract this into method?
        // TODO: pack textures into binary or better loading?
        let path = format!("client/assets/skybox/sky_{face}.png");

        let bytes = fs::read(path).expect("TODO: better error; file to exist");

        image::load_from_memory(&bytes).expect("TODO: better error; valid image")
    });

    let texture = Texture::from_cubemap(&g_ctx.device, &g_ctx.queue, &loaded_textures, Some("skybox texture"));

    let bind_group_layout = g_ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("skybox bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true }
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        ],
    });

    let bind_group = g_ctx.device.create_bind_group(&BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler)
            }
        ],
        label: Some("skybox bind group"),
    });

    let render_pipeline = g_ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("skybox pipeline"),
        layout: Some(&g_ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skybox pipeline layout"),
            bind_group_layouts: &[&bind_group_layout, &camera_uniform_buffer.bind_group_layout],
            push_constant_ranges: &[]
        })),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: g_ctx.config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    storages.add_unique(SkyboxRenderBundle { texture, bind_group, bind_group_layout, render_pipeline });
}
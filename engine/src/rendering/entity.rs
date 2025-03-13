use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::math::Vertex;
use crate::rendering::texture::Texture;
use shipyard::{AllStoragesView, Unique, UniqueView};
use crate::rendering::math::initialize_base_face;
use crate::rendering::sized_buffer::SizedBuffer;

#[derive(Unique)]
pub struct EntityRenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub base_face: SizedBuffer,
}

pub fn initialize_entity_render_state(
    g_ctx: UniqueView<GraphicsContext>,
    camera_uniform_buffer: UniqueView<CameraUniformBuffer>,
    storages: AllStoragesView,
) {
    let shader = g_ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/entity.wgsl"));

    let push_constant_range = wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::VERTEX,
        range: 0..36,
    };

    // pipeline describes the GPU's actions on a set of data, like a shader program
    let render_pipeline_layout = g_ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&camera_uniform_buffer.bind_group_layout], // layouts of the bind groups, matches @group(n) in shader
        push_constant_ranges: &[push_constant_range],
    });

    let pipeline = g_ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[ // format of the vertex buffers used, indices correspond to slot when setting the buffer before rendering
                Vertex::buffer_desc()
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: g_ctx.config.format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), // blending, if set to replace this overwrites the contents
                write_mask: wgpu::ColorWrites::ALL, // write to all channels (rgba)
            })],
        }),
        primitive: wgpu::PrimitiveState { // how to interpret vertices when converting to triangles
            topology: wgpu::PrimitiveTopology::TriangleList, // 3 vert per triangle
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // counter-clockwise ordered faces are front
            cull_mode: Some(wgpu::Face::Back), // backface culling
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual, // draw pixels front to back based on the depth texture
            stencil: wgpu::StencilState::default(), // usually stored in same texture as depth texture
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1, // multisampling
            mask: !0, // use all samples
            alpha_to_coverage_enabled: false, // for anti-aliasing
        },
        multiview: None, // for rendering to array textures
    });

    storages.add_unique(EntityRenderState { pipeline, base_face: initialize_base_face(&g_ctx) });
}
use shipyard::{AllStoragesView, IntoWorkload, Unique, UniqueView, Workload};
use crate::rendering;
use rendering::camera_uniform_buffer::CameraUniformBuffer;
use rendering::face_data::FaceData;
use rendering::graphics_context::GraphicsContext;
use rendering::texture::Texture;
use rendering::{base_face, depth_texture, face_buffer};
use rendering::texture_atlas;
use rendering::texture_atlas::TextureAtlas;
use rendering::vertex::Vertex;
use crate::rendering::gizmos;
use crate::rendering::render::BlockOutlineRenderState;
use crate::rendering::sized_buffer::SizedBuffer;

#[derive(Unique)]
pub struct RenderPipeline(pub wgpu::RenderPipeline);

pub fn initialize_renderer() -> Workload {
    (
        (
            base_face::initialize_base_face,
            face_buffer::init_face_buffer,
            texture_atlas::initialize_texture_atlas,
            depth_texture::initialize_depth_texture,
            initialize_camera_uniform_buffer,
        ).into_workload(),
        create_pipeline,
        initialize_block_outline_render_state,
        gizmos::initialize,
    ).into_sequential_workload()
}

pub fn initialize_block_outline_render_state(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let buffer = g_ctx.device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("block_outline_buffer"),
            size: 6 * size_of::<FaceData>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );
    
    storages.add_unique(BlockOutlineRenderState {
        buffer: SizedBuffer { buffer, size: 0 },
    })
}

pub fn initialize_camera_uniform_buffer(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    storages.add_unique(CameraUniformBuffer::new(&g_ctx));
}

pub fn create_pipeline(g_ctx: UniqueView<GraphicsContext>, camera_uniform_buffer: UniqueView<CameraUniformBuffer>, texture_atlas: UniqueView<TextureAtlas>, storages: AllStoragesView) {
    // 5. pipeline / instructions for GPU

    // loads a shader and returns a handle to the compiled shader
    let shader = g_ctx.device.create_shader_module(wgpu::include_wgsl!("../rendering/shaders/shader.wgsl"));

    let push_constant_range = wgpu::PushConstantRange {
        stages: wgpu::ShaderStages::VERTEX,
        range: 0..12,
    };

    // pipeline describes the GPU's actions on a set of data, like a shader program
    let render_pipeline_layout = g_ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&texture_atlas.bind_group_layout, &camera_uniform_buffer.bind_group_layout], // layouts of the bind groups, matches @group(n) in shader
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
                Vertex::buffer_desc(), FaceData::buffer_desc()
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: g_ctx.config.format,
                blend: Some(wgpu::BlendState::REPLACE), // blending, if set to replace this overwrites the contents
                write_mask: wgpu::ColorWrites::ALL, // write to all channels (rgba)
            })],
        }),
        primitive: wgpu::PrimitiveState { // how to interpret vertices when converting to triangles
            topology: wgpu::PrimitiveTopology::TriangleStrip, // 3 vert per triangle
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

    storages.add_unique(RenderPipeline(pipeline));
}
use shipyard::{AllStoragesView, Unique, UniqueView};
use wgpu::util::DeviceExt;
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::texture::Texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GizmoVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Unique)]
pub struct LineGizmosBuffer {
    pub buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

#[derive(Unique)]
pub struct GizmosPipeline(pub wgpu::RenderPipeline);

impl GizmoVertex {
    pub fn buffer_desc() -> wgpu::VertexBufferLayout<'static> {
        // corresponds to using @location(x) in shader, how to read the buffer, what types and offsets
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

pub fn init_test_gizmos(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let temp_line = [
        GizmoVertex { position: [1.0, 12.0, 2.0], color: [1.0, 0.0, 0.0] },
        GizmoVertex { position: [-1.0, 12.0, 2.0], color: [1.0, 0.0, 0.0] },
    ];

    let buffer = g_ctx.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),
            contents: bytemuck::cast_slice(&temp_line),
            // use the buffer in a uniform in a bind group, copy_dst -> it can be written to in bind group
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }
    );

    storages.add_unique(LineGizmosBuffer {
        buffer,
        num_vertices: temp_line.len() as _,
    })
}

pub fn create_line_gizmos_pipeline(g_ctx: UniqueView<GraphicsContext>, camera_uniform_buffer: UniqueView<CameraUniformBuffer>, storages: AllStoragesView) {
    // 5. pipeline / instructions for GPU

    // loads a shader and returns a handle to the compiled shader
    let shader = g_ctx.device.create_shader_module(wgpu::include_wgsl!("../rendering/shaders/gizmos_lines.wgsl"));

    // pipeline describes the GPU's actions on a set of data, like a shader program
    let render_pipeline_layout = g_ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Line Gizmos Pipeline Layout"),
        bind_group_layouts: &[&camera_uniform_buffer.bind_group_layout], // layouts of the bind groups, matches @group(n) in shader
        .. Default::default()
    });

    let pipeline = g_ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Line Gizmos Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[ // format of the vertex buffers used, indices correspond to slot when setting the buffer before rendering
                GizmoVertex::buffer_desc()
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
            topology: wgpu::PrimitiveTopology::LineList, // 3 vert per triangle
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // counter-clockwise ordered faces are front
            cull_mode: None, // backface culling
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // draw pixels front to back based on the depth texture
            stencil: wgpu::StencilState::default(), // usually stored in same texture as depth texture
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None, // for rendering to array textures
    });

    storages.add_unique(GizmosPipeline(pipeline));
}
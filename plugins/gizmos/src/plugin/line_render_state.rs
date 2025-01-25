use shipyard::{AllStoragesView, Unique, UniqueView};
use engine::rendering::camera_uniform_buffer::CameraUniformBuffer;
use engine::rendering::graphics_context::GraphicsContext;
use engine::rendering::sized_buffer::SizedBuffer;
use engine::rendering::texture::Texture;
use crate::plugin::settings::GizmoRenderingSettings;
use crate::plugin::vertex::GizmoVertex;

#[derive(Unique)]
pub struct GizmosLineRenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub sized_buffer: SizedBuffer,
}

impl GizmosLineRenderState {
    pub fn update_buffer(&mut self, g_ctx: &GraphicsContext, buffer: &[GizmoVertex]) {
        g_ctx.queue.write_buffer(&self.sized_buffer.buffer, 0, bytemuck::cast_slice(buffer));
        self.sized_buffer.size = buffer.len() as _;
    }
}

pub fn initialize_line_gizmos_render_state(g_ctx: UniqueView<GraphicsContext>, gizmo_settings: UniqueView<GizmoRenderingSettings>, camera_uniform_buffer: UniqueView<CameraUniformBuffer>, storages: AllStoragesView) {
    let num_line_gizmo_vertices = gizmo_settings.num_lines() * 2;

    let buffer = g_ctx.device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("line_gizmos_buffer"),
            size: num_line_gizmo_vertices as u64 * std::mem::size_of::<GizmoVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    let shader = g_ctx.device.create_shader_module(wgpu::include_wgsl!("../../rendering/shaders/gizmos_lines.wgsl"));

    let pipeline_layout = g_ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("line_gizmos_pipeline_layout"),
        bind_group_layouts: &[&camera_uniform_buffer.bind_group_layout],
        .. Default::default()
    });

    let pipeline = g_ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Line Gizmos Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GizmoVertex::buffer_desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: g_ctx.config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    storages.add_unique(GizmosLineRenderState {
        pipeline,
        sized_buffer: SizedBuffer { buffer, size: 0 },
    });
}
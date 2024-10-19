use shipyard::{AllStoragesView, IntoWorkload, Unique, UniqueView, Workload};
use wgpu::util::DeviceExt;
use crate::rendering::camera_uniform_buffer::CameraUniformBuffer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::sized_buffer::SizedBuffer;
use crate::rendering::texture::Texture;

pub fn initialize() -> Workload {
    (
        read_settings,
        initialize_line_gizmos_render_state,
    ).into_sequential_workload()
}

pub fn read_settings(storages: AllStoragesView) {
    // TODO: parse from file somewhere else
    storages.add_unique(GizmoRenderingSettings::default())
}

#[derive(Unique)]
pub struct GizmoRenderingSettings {
    pub max_line_gizmos: u16,
    pub max_box_gizmos: u16,
}

impl Default for GizmoRenderingSettings {
    fn default() -> Self {
        Self {
            max_line_gizmos: 512,
            max_box_gizmos: 512,
        }
    }
}

impl GizmoRenderingSettings {
    fn num_lines(&self) -> u32 {
        self.max_line_gizmos as u32 + self.max_box_gizmos as u32 * 12
    }
}

#[derive(Unique)]
pub struct GizmosLineRenderState {
    pub pipeline: wgpu::RenderPipeline,
    pub sized_buffer: SizedBuffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GizmoVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

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

fn initialize_line_gizmos_render_state(g_ctx: UniqueView<GraphicsContext>, gizmo_settings: UniqueView<GizmoRenderingSettings>, camera_uniform_buffer: UniqueView<CameraUniformBuffer>, storages: AllStoragesView) {
    let num_line_gizmo_vertices = gizmo_settings.num_lines() * 2;

    let buffer = g_ctx.device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("line_gizmos_buffer"),
            size: num_line_gizmo_vertices as u64 * std::mem::size_of::<GizmoVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    );

    let shader = g_ctx.device.create_shader_module(wgpu::include_wgsl!("../rendering/shaders/gizmos_lines.wgsl"));

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
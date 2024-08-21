use crate::rendering::vertex::Vertex;
use shipyard::{AllStoragesView, Unique, UniqueView};
use wgpu::util::DeviceExt;
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Unique)]
pub struct BaseFace {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

pub fn initialize_base_face(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    // TODO: each vertex can be compressed into 5 bits
    const BASE_FACE: [Vertex; 4] = [
        Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0] },
        Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [1.0, 0.0, 1.0], tex_coords: [1.0, 1.0] },
    ];

    // holds vertices, available in shader
    let vertex_buffer = g_ctx.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&BASE_FACE),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    storages.add_unique(BaseFace { vertex_buffer, num_vertices: BASE_FACE.len() as _ })
}
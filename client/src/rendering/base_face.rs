use wgpu::util::DeviceExt;
use crate::rendering::vertex::Vertex;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::sized_buffer::SizedBuffer;

pub fn initialize_base_face(g_ctx: &GraphicsContext) -> SizedBuffer {
    // TODO: each vertex can be compressed into 5 bits
    const BASE_FACE: [Vertex; 4] = [
        Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 0.0] },
        Vertex { position: [0.0, 0.0, 1.0], tex_coords: [0.0, 1.0] },
        Vertex { position: [1.0, 0.0, 0.0], tex_coords: [1.0, 0.0] },
        Vertex { position: [1.0, 0.0, 1.0], tex_coords: [1.0, 1.0] },
    ];

    // holds vertices, available in shader
    let buffer = g_ctx.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&BASE_FACE),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    SizedBuffer { buffer, size: BASE_FACE.len() as _ }
}
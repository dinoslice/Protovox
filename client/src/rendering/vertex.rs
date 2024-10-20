#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex { // VertexInput in shader
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    #[allow(dead_code)]
    pub fn add_pos(&self, x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [
                self.position[0] + x,
                self.position[1] + y,
                self.position[2] + z,
            ],
            tex_coords: self.tex_coords,
        }
    }

    pub fn buffer_desc() -> wgpu::VertexBufferLayout<'static> {
        // corresponds to using @location(x) in shader, how to read the buffer, what types and offsets
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress, // how wide (bytes) each vertex is
            step_mode: wgpu::VertexStepMode::Vertex, // switch to the next item in the buffer after each vertex
            attributes: &ATTRIBUTES, // generally a 1:1 mapping with the struct fields
        }
    }
}

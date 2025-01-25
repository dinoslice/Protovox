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
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
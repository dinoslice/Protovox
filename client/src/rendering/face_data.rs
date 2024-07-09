#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FaceData {
    pub face: u32,
    pub _pad: u32,
    pub pos: [f32; 3],
}

impl FaceData {
    pub fn new(x: f32, y: f32, z: f32, face: u8) -> Self {
        Self {
            face: face as u32,
            _pad: 0,
            pos: [x, y, z],
        }
    }

    pub fn buffer_desc() -> wgpu::VertexBufferLayout<'static> {
        // corresponds to using @location(x) in shader, how to read the buffer, what types and offsets
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![2 => Uint32x2, 3 => Float32x3];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress, // how wide (bytes) each vertex is
            step_mode: wgpu::VertexStepMode::Instance, // switch to the next item in the buffer after each vertex
            attributes: &ATTRIBUTES, // generally a 1:1 mapping with the struct fields
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum FaceType {
    Bottom, // Y-
    Top, // Y+
    Front, // Z+
    Back, // Z-
    Left, // X-
    Right, // X+
}
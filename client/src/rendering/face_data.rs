use game::chunk::pos::ChunkPos;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FaceData(u32);

impl FaceData {
    pub fn new(pos: ChunkPos, face: FaceType) -> Self {
        let mut data = pos.0 as _;
        data |= (face as u8 as u32 & 0x7) << 16;

        Self(data)
    }

    pub fn buffer_desc() -> wgpu::VertexBufferLayout<'static> {
        // corresponds to using @location(x) in shader, how to read the buffer, what types and offsets
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] =
            wgpu::vertex_attr_array![2 => Uint32];

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
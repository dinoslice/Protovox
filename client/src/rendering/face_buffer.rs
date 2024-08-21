use shipyard::{AllStoragesView, Unique, UniqueView};
use crate::rendering::face_data::FaceData;
use crate::rendering::graphics_context::GraphicsContext;

// TODO: this actually represents the max chunks if all faces were present, real max is much higher
const MAX_CHUNKS: u64 = 100;
const FACES_PER_VOXEL: u64 = 6;
const VOXELS_PER_CHUNK: u64 = 32 * 64 * 32;

#[derive(Unique)]
pub struct FaceBuffer(pub wgpu::Buffer);

pub fn init_face_buffer(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    // TODO: this can be reduced once culling is implemented
    const FACE_BUFFER_MAX_SIZE: u64 = std::mem::size_of::<FaceData>() as u64 * FACES_PER_VOXEL * VOXELS_PER_CHUNK * MAX_CHUNKS;

    let face_buffer = g_ctx.device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Face Buffer"),
            size: FACE_BUFFER_MAX_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, // only needed in vertex buffer,
            mapped_at_creation: false,
        }
    );

    storages.add_unique(FaceBuffer(face_buffer));
}
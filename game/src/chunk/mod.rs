pub mod pos;
pub mod data;
pub mod location;
mod adjacent;

pub const CHUNK_SIZE: glm::TVec3<u8> = glm::TVec3::new(32, 64, 32);
pub const BLOCKS_PER_CHUNK: usize = (32 * 64 * 32) as _;
use std::sync::OnceLock;

use game::{block::Block, chunk::{data::ChunkData, location::ChunkLocation, pos::ChunkPos, CHUNK_SIZE}};
use noise::{NoiseFn, Perlin};

static PERLIN_NOISE: OnceLock<Perlin> = OnceLock::new();

pub fn generate_chunk(chunk: ChunkLocation) -> ChunkData {
    let mut out = ChunkData::empty(chunk);

    let perlin = PERLIN_NOISE.get_or_init(|| Perlin::new(0));

    for x in 0..CHUNK_SIZE.x {
        for z in 0..CHUNK_SIZE.z {
            let world_x = (out.location.0.x as i32 * CHUNK_SIZE.x as i32) + x as i32;
            let world_z = (out.location.0.z as i32 * CHUNK_SIZE.z as i32) + z as i32;

            let height = perlin.get([world_x as f64, world_z as f64]) * 10.0;

            out.set_block(ChunkPos::new_unchecked(x, height as u8, z), Block::Grass);
            for y in 0..(height as u8) {
                if y >= (height as u8 - 3) {
                    out.set_block(ChunkPos::new_unchecked(x, y, z), Block::Dirt);
                } else {
                    out.set_block(ChunkPos::new_unchecked(x, y, z), Block::Cobblestone);
                }
            }
        }
    }

    out
}
mod spline;
pub mod biomes;
pub mod fractal_noise;

use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender};
use game::{block::Block, chunk::{data::ChunkData, location::ChunkLocation, pos::ChunkPos, CHUNK_SIZE}};
use noise::{NoiseFn, Perlin};
use rayon::{ThreadPool, ThreadPoolBuilder};
use shipyard::Unique;

use crate::events::ChunkGenEvent;

#[derive(Unique)]
pub struct WorldGenerator {
    thread_pool: ThreadPool,
    perlin_noise: Arc<Perlin>,
    chunk_output: (Sender<ChunkGenEvent>, Receiver<ChunkGenEvent>),
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .expect("thread pool did not build successfully");

        let chunk_output = crossbeam::channel::unbounded::<ChunkGenEvent>();
        let perlin_noise = Arc::new(Perlin::new(seed));

        Self {
            thread_pool,
            perlin_noise,
            chunk_output,
        }
    }

    pub fn receive_chunks(&self) -> Vec<ChunkGenEvent> {
        let mut out = vec![];
        while let Ok(data) = self.chunk_output.1.try_recv() {
            out.push(data);
        }
        
        out
    }

    pub fn spawn_generate_task(&self, chunk: ChunkLocation) {
        let sender = self.chunk_output.0.clone();
        let perlin = self.perlin_noise.clone();

        self.thread_pool.spawn(move ||
            sender.send(Self::generate_chunk(perlin, chunk))
                .expect("channel should not have disconnected")
        );
    }


    fn generate_chunk(perlin: Arc<Perlin>, chunk: ChunkLocation) -> ChunkGenEvent {
        let mut out = ChunkData::empty(chunk);

        for x in 0..CHUNK_SIZE.x {
            for z in 0..CHUNK_SIZE.z {
                let world_x = (out.location.0.x * CHUNK_SIZE.x as i32) + x as i32;
                let world_z = (out.location.0.z * CHUNK_SIZE.z as i32) + z as i32;

                let height = perlin.get([world_x as f64 / 10.0, world_z as f64 / 10.0]) * 10.0;

                let height = height.abs() as u8;

                out.set_block(ChunkPos::new_unchecked(x, height, z), Block::Grass);
                for y in 0..height {
                    if y + 3 >= (height) {
                        out.set_block(ChunkPos::new_unchecked(x, y, z), Block::Dirt);
                    } else {
                        out.set_block(ChunkPos::new_unchecked(x, y, z), Block::Cobblestone);
                    }
                }
            }
        }

        ChunkGenEvent(out)
    }
}
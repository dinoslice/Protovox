mod spline;
pub mod biomes;
pub mod fractal_noise;
pub mod debug;

use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender};
use game::{block::Block, chunk::{data::ChunkData, location::ChunkLocation, pos::ChunkPos, CHUNK_SIZE}};
use noise::{Perlin, NoiseFn};
use rand::Rng;
use rayon::{ThreadPool, ThreadPoolBuilder};
use shipyard::Unique;
use game::location::WorldLocation;
use crate::events::ChunkGenEvent;
use crate::world_gen::biomes::BiomeGenerator;

pub const PERLIN_SCALE: f64 = 100.0;

#[derive(Unique)]
pub struct WorldGenerator {
    thread_pool: ThreadPool,
    perlin_noise: Arc<Perlin>,
    chunk_output: (Sender<ChunkGenEvent>, Receiver<ChunkGenEvent>),
    pub biome_generator: Arc<BiomeGenerator>
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .expect("thread pool did not build successfully");

        let chunk_output = crossbeam::channel::unbounded::<ChunkGenEvent>();
        let perlin_noise = Arc::new(Perlin::new(seed));
        let biome_generator = Arc::new(BiomeGenerator::overworld(seed));

        Self {
            thread_pool,
            perlin_noise,
            chunk_output,
            biome_generator
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

        let biome_generator = self.biome_generator.clone();

        self.thread_pool.spawn(move ||
            sender.send(Self::generate_chunk(perlin, biome_generator, chunk))
                .expect("channel should not have disconnected")
        );
    }

    fn generate_chunk(perlin: Arc<Perlin>, biome_generator: Arc<BiomeGenerator>, chunk: ChunkLocation) -> ChunkGenEvent {
        let mut out = ChunkData::empty(chunk.clone());

        for x in 0..CHUNK_SIZE.x {
            for z in 0..CHUNK_SIZE.z {
                let world_x = (out.location.0.x * CHUNK_SIZE.x as i32) + x as i32;
                let world_z = (out.location.0.z * CHUNK_SIZE.z as i32) + z as i32;
                
                //TODO: go thru each block and squish 3d perlin based on erosion, middle of squish is at altitude
                let world = WorldLocation::from(&chunk) + ChunkPos::new_unchecked(x, 0, z);

                let parameters = biome_generator.generate_block_data(&world);
                let altitude = parameters.altitude as f64;
                let erosion = parameters.erosion as f64;

                for y in 0..CHUNK_SIZE.y {
                    let world_y = ((out.location.0.y * CHUNK_SIZE.y as i32) + y as i32) as f64;

                    let v = perlin.get([world_x as f64 / PERLIN_SCALE, world_y / PERLIN_SCALE, world_z as f64 / PERLIN_SCALE]) -
                        ((0.25 * (world_y - (20.0 + (altitude * (10.0 / ((erosion * erosion) + 1.0)))))).tanh());
                    if v >= 0.0 {
                        out.set_block(ChunkPos::new_unchecked(x, y, z), if rand::thread_rng().gen() { Block::Cobblestone } else { Block::Dirt });
                    }
                }
            }
        }

        ChunkGenEvent(out)
    }
}
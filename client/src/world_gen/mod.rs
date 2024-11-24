pub mod params;

use std::cmp::Ordering;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender};
use game::{block::Block, chunk::{data::ChunkData, location::ChunkLocation, pos::ChunkPos, CHUNK_SIZE}};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use rayon::{ThreadPool, ThreadPoolBuilder};
use shipyard::Unique;
use game::location::BlockLocation;
use splines::easings::InOutSine;
use splines::Spline;
use crate::events::ChunkGenEvent;
use crate::world_gen::params::WorldGenParams;

pub type SineSpline = Spline<InOutSine>;

pub const PERLIN_SCALE: f64 = 100.0;

#[derive(Unique)]
pub struct WorldGenerator {
    thread_pool: ThreadPool,
    perlin_noise: Arc<Perlin>,
    chunk_output: (Sender<ChunkGenEvent>, Receiver<ChunkGenEvent>),
}

#[derive(Unique, Default, Clone)]
pub struct WorldGenSplines {
    pub continentalness: SineSpline,
    pub erosion: SineSpline,
    pub peaks_valleys: SineSpline,
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

    pub fn spawn_generate_task(&self, chunk: ChunkLocation, splines: &WorldGenSplines, params: &WorldGenParams) {
        let sender = self.chunk_output.0.clone();
        let perlin = self.perlin_noise.clone();

        let splines = splines.clone();
        let params = params.clone();

        self.thread_pool.spawn(move ||
            sender.send(Self::generate_chunk(perlin, splines, chunk, params))
                .expect("channel should not have disconnected")
        );
    }

    fn generate_chunk(perlin: Arc<Perlin>, splines: WorldGenSplines, chunk: ChunkLocation, params: WorldGenParams) -> ChunkGenEvent {
        let mut out = ChunkData::empty(chunk.clone());

        let chunk_start = BlockLocation::from(&chunk);

        let noise_range = -1.0..=1.0;

        let water_level = remap(noise_range.clone(), params.c_start..=params.c_end, -0.175) as i32;

        for x in 0..CHUNK_SIZE.x {
            for z in 0..CHUNK_SIZE.z {
                let xf = (x as i32 + chunk_start.0.x) as f64;
                let zf = (z as i32 + chunk_start.0.z) as f64;

                // Sample the Perlin noise at world coordinates
                let continentalness_noise = perlin.get([xf * params.continentalness_scale, zf * params.continentalness_scale]) as f32;
                let erosion_noise = perlin.get([xf * params.erosion_scale, zf * params.erosion_scale]) as f32;
                let peaks_and_valleys_noise = perlin.get([xf * params.peaks_valleys_scale, zf * params.peaks_valleys_scale]) as f32;

                let continentalness = splines.continentalness.sample(continentalness_noise);
                let erosion = splines.erosion.sample(erosion_noise);
                let peaks_and_valleys = splines.peaks_valleys.sample(peaks_and_valleys_noise);

                let height = remap(noise_range.clone(), params.c_start..=params.c_end, continentalness) + remap(noise_range.clone(), params.e_start..=params.e_end, erosion) * remap(noise_range.clone(), params.pv_start..=params.pv_end, peaks_and_valleys);

                for y in 0..CHUNK_SIZE.y {
                    let pos = ChunkPos::new(x, y, z).expect("valid");

                    let block_y = chunk_start.0.y + y as i32; // could use BlockLocation::from_chunk_parts, but this is faster

                    match height as i32 - block_y {
                        0 => match block_y.cmp(&water_level) {
                            Ordering::Greater | Ordering::Equal => out.set_block(pos, Block::Grass),
                            Ordering::Less => out.set_block(pos, Block::Dirt),
                        }
                        1..4 => out.set_block(pos, Block::Dirt),
                        4.. => out.set_block(pos, Block::Cobblestone),
                        _ if block_y <= water_level => out.set_block(pos, Block::Debug), // TODO: make water block
                        _ => {}, // AIR
                    }
                }
            }
        }

        ChunkGenEvent(out)
    }
}

fn remap(input: RangeInclusive<f32>, output: RangeInclusive<f32>, v: f32) -> f32 {
    let input_start = *input.start();
    let input_end = *input.end();
    let output_start = *output.start();
    let output_end = *output.end();

    output_start + (v - input_start) * (output_end - output_start) / (input_end - input_start)
}
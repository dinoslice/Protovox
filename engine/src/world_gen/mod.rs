pub mod params;

use std::cmp::Ordering;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender};
use game::{block::Block, chunk::{data::ChunkData, location::ChunkLocation, pos::ChunkPos, CHUNK_SIZE}};
use noise::{NoiseFn, Perlin};
use rayon::{ThreadPool, ThreadPoolBuilder};
use shipyard::Unique;
use game::location::BlockLocation;
use splines::easings::InOutSine;
use splines::Spline;
use crate::events::ChunkGenEvent;
use crate::world_gen::params::WorldGenParams;

pub type SineSpline = Spline<InOutSine>;

#[derive(Unique)]
pub struct WorldGenerator {
    thread_pool: ThreadPool,
    perlin_noise: Arc<Perlin>,
    pub params: Arc<WorldGenParams>,
    pub splines: Arc<WorldGenSplines>,
    chunk_output: (Sender<ChunkGenEvent>, Receiver<ChunkGenEvent>),
}

#[derive(Default, Clone)]
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

        let params = WorldGenParams {
            continentalness_scale: 0.00125,
            erosion_scale: 0.002,
            peaks_valleys_scale: 0.0125,
            c_start: -10.0,
            c_end: 175.0,
            e_start: -0.5,
            e_end: 1.0,
            pv_start: 0.0,
            pv_end: 35.0,
        };

        let splines = WorldGenSplines {
            continentalness: Spline::new([[-1.0, -1.0], [-0.9279977, -0.90286434], [-0.26820922, -0.8263215], [-0.044113815, -0.14479148], [0.763953, -0.08767879], [0.95565224, 0.9540222], [1.0, 1.0]]),
            erosion: Spline::new([[-1.0, 1.0], [-0.83050734, 0.4721343], [-0.5038637, 0.26844186], [-0.3988908, 0.43217272], [-0.2064119, -0.816993], [0.5861441, -0.90852606], [0.636498, -0.43075633], [0.7577101, -0.44334638], [0.798712, -0.89013314], [1.0, -1.0]]),
            peaks_valleys: Spline::new([[-1.0, -1.0], [-0.9223045, -0.8987539], [-0.5608352, -0.8535681], [-0.3662839, -0.24826753], [0.23613429, -0.102552295], [0.767043, 0.8733756], [1.0, 1.0]]),
        };

        Self {
            thread_pool,
            perlin_noise,
            params: Arc::new(params),
            splines: Arc::new(splines),
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

    pub fn spawn_generate_task(&self, chunk: ChunkLocation, splines: Arc<WorldGenSplines>, params: Arc<WorldGenParams>) {
        let sender = self.chunk_output.0.clone();
        let perlin = self.perlin_noise.clone();

        self.thread_pool.spawn(move ||
            sender.send(Self::generate_chunk(perlin, splines, chunk, params))
                .expect("channel should not have disconnected")
        );
    }
    
    pub fn send(&self, chunk_data: ChunkData) {
        let sender = self.chunk_output.0.clone();
        
        self.thread_pool.spawn(move ||
            sender.send(ChunkGenEvent(chunk_data))
                .expect("channel should not have disconnected")
        );
    }

    fn generate_chunk(perlin: Arc<Perlin>, splines: Arc<WorldGenSplines>, chunk: ChunkLocation, params: Arc<WorldGenParams>) -> ChunkGenEvent {
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

                    let stone_scale = 0.15;
                    let cobble_threshold = -0.5;

                    match height as i32 - block_y {
                        0 => match block_y.cmp(&water_level) {
                            Ordering::Greater | Ordering::Equal => *out.block_mut(pos) = Block::Grass,
                            Ordering::Less => *out.block_mut(pos) = Block::Dirt,
                        }
                        1..4 => *out.block_mut(pos) = Block::Dirt,
                        y @ 4.. => *out.block_mut(pos) = if perlin.get([xf, y as f64, zf].map(|n| n * stone_scale)) < cobble_threshold {
                            Block::Cobblestone
                        } else {
                            Block::Stone
                        },
                        _ if block_y <= water_level => *out.block_mut(pos) = Block::Debug, // TODO: make water block
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
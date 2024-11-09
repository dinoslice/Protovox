use std::cell::LazyCell;
use noise::NoiseFn;
use rand::{thread_rng, Rng};
use game::location::WorldLocation;
use crate::spline;
use settings::BiomeParameters;
use crate::world_gen::fractal_noise::FractalNoise;
use crate::world_gen::PERLIN_SCALE;
use crate::world_gen::spline::Spline5D;

pub mod settings;

pub struct BiomeGenerator {
    biome_list: Vec<Biome>,
    temperature: FractalNoise,
    humidity: FractalNoise,
    erosion: FractalNoise,
    rainfall: FractalNoise,
    altitude: FractalNoise
}

impl BiomeGenerator {
    pub fn new(biome_list: Vec<Biome>, seed: u32) -> Self {
        Self {
            biome_list,
            temperature: FractalNoise::new(seed, 10, &[1.5, 0.0, 1.0, 0.0, 0.0, 0.0]),
            humidity: FractalNoise::new(seed, 8, &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0]),
            erosion: FractalNoise::new(seed, 9, &[1.0, 1.0, 0.0, 1.0, 1.0]),
            rainfall: FractalNoise::new(seed, 2, &[1.0]),
            altitude: FractalNoise::new(seed, 9, &[1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0])
        }
    }

    pub fn overworld(seed: u32) -> Self {
        BiomeGenerator::new(vec![Biome::Null, Biome::Plains, Biome::Mountain], seed)
    }

    pub fn generate_biome(&self, location: &WorldLocation) -> &Biome {
        let mut chances = vec![];
        for biome in &self.biome_list {
            chances.push((biome, biome.get_spline().interpolate(self.generate_block_data(location).into())))
        }

        let max_chance = chances.iter().map(|(_, chance)| *chance).reduce(f32::max).expect("There isn't a maximum value in the chance vector.");
        let chance = thread_rng().gen_range(0.0..max_chance);

        let mut chances = chances.into_iter().filter(|(_, a)| a < &chance).collect::<Vec<_>>();
        chances.sort_by(|(_, a), (_, b)| a.total_cmp(b));
        let default = (&self.biome_list[0], 0.0);
        let b = chances.last();
        let b = *b.unwrap_or(&default);
        
        b.0
    }

    pub fn generate_block_data(&self, location: &WorldLocation) -> BiomeParameters {
        let point = [location.0.x as f64 / PERLIN_SCALE, location.0.y as f64 / PERLIN_SCALE, location.0.z as f64 / PERLIN_SCALE];

        BiomeParameters { // TODO: add function to build from point & BiomeGenerator
            temperature: self.temperature.get(point) as _,
            humidity: self.humidity.get(point) as _,
            altitude: self.altitude.get(point) as _,
            rainfall: self.rainfall.get(point) as _,
            erosion: self.erosion.get(point) as _,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Biome {
    Null, // TODO: get rid of null
    Plains,
    Mountain
}

impl Default for Biome {
    fn default() -> Self {
        Self::Plains
    }
}

impl Biome {
    #[allow(clippy::declare_interior_mutable_const)]
    const PLAINS_SPLINE: LazyCell<Spline5D> = LazyCell::new(|| spline!(
            0.2, 0.4, 0.1, 0.3, 0.8 => 0.0,
            0.35, 0.45, 0.25, 0.45, 1.0 => 1.0,
            0.5, 0.7, 0.4, 0.7, 1.0 => 0.0
        ));
    #[allow(clippy::declare_interior_mutable_const)]
    const MOUNTAIN_SPLINE: LazyCell<Spline5D> = LazyCell::new(|| spline!(
            -1.0, 0.5, 0.8, 0.9, 1.0 => 0.0,
            0.0, 0.55, 1.0, 0.95, 1.0 => 1.0,
            1.0, 0.6, 1.0, 1.0, 1.0 => 0.0
        ));
    #[allow(clippy::declare_interior_mutable_const)]
    const NULL_SPLINE: LazyCell<Spline5D> = LazyCell::new(|| spline!(
            0.0, 0.0, 0.0, 0.0, 0.0 => 0.0
        ));
    
    pub fn get_spline(&self) -> LazyCell<Spline5D> {
        match self {
            Biome::Null => Self::NULL_SPLINE,
            Biome::Plains => Self::PLAINS_SPLINE,
            Biome::Mountain => Self::MOUNTAIN_SPLINE
        }
    }
}
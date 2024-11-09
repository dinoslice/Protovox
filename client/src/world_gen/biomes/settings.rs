use crate::world_gen::spline::Vec5;
#[derive(Debug)]
pub struct BiomeParameters {
    pub temperature: f32,
    pub humidity: f32,
    pub altitude: f32,
    pub rainfall: f32,
    pub erosion: f32,
}

impl Default for BiomeParameters {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            humidity: 0.0,
            altitude: 0.0,
            rainfall: 0.0,
            erosion: 0.0
        }
    }
}

impl From<BiomeParameters> for Vec5 {
    fn from(value: BiomeParameters) -> Self {
        Vec5::new(value.temperature, value.humidity, value.altitude, value.rainfall, value.erosion)
    }
}

pub struct BiomeModificationParameters {
    soil_fertility: f32,
    rock_porousness: f32,
    water_concentration: f32,
}
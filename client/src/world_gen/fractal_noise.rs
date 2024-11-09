use noise::{NoiseFn, Perlin};

pub struct FractalNoise {
    noise: Perlin,
    octaves: u8,
    amplitudes: Vec<f32>
}

impl NoiseFn<f64, 3> for FractalNoise {
    fn get(&self, point: [f64; 3]) -> f64 {
        let mut noise_value = 0.0;
        let mut amplitude = 1.0;

        for octave_index in 0..self.amplitudes.len() {
            let octave = -(self.octaves as i32) * octave_index as i32;
            let frequency = 2.0f64.powi(octave);

            noise_value += self.noise.get(point.map(|v| v * frequency)) * amplitude;

            amplitude *= self.amplitudes[octave_index] as f64;
        }

        noise_value
    }
}

impl FractalNoise {
    pub fn new(seed: u32, octaves: u8, amplitudes: &[f32]) -> Self {
        Self {
            noise: Perlin::new(seed),
            octaves,
            amplitudes: amplitudes.to_vec()
        }
    }
}
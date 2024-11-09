use glm::TVec;

pub type Vec5 = TVec<f32, 5>;

pub struct Spline5D {
    points: Vec<(Vec5, f32)>
}

impl Spline5D {
    pub fn new<const N: usize>(points: [(Vec5, f32); N]) -> Self {
        Self { points: points.into() }
    }

    pub fn interpolate(&self, query: Vec5) -> f32 {
        let mut result = 0.0;
        let mut total_weight = 0.0;

        for (point, output) in &self.points {
            let weight = Self::weight(&query, &point);
            result += output * weight;
            total_weight += weight;
        }

        result / total_weight
    }

    fn weight(query: &Vec5, point: &Vec5) -> f32 {
        let distance = query.iter()
            .zip(point.iter())
            .map(|(q, p)| (q - p) * (q - p))
            .sum::<f32>()
            .sqrt();

        1.0 / (distance + 1e-10)
    }
}

#[macro_export]
macro_rules! spline {
    ($($a:expr, $b:expr, $c:expr, $d:expr, $e:expr => $o:expr),*) => {
        {
            use crate::world_gen::spline::Vec5;
            Spline5D::new([
                $(
                    (Vec5::new($a, $b, $c, $d, $e), $o),
                )*
            ])
        }
    };
    
    ($($a:expr => $o:expr),*) => {
        {
            Spline5D::new([
                $(
                    (Vec5::new($a.temperature, $a.humidity, $a.altitude, $a.rainfall, $a.erosion), $o),
                )*
            ])
        }
    };
}
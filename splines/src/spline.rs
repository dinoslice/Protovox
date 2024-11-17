use std::marker::PhantomData;
use glm::Vec2;

pub trait Easing {
    fn ease(x: f32) -> f32;
}

#[derive(Debug)]
pub struct Spline<E: Easing> {
    points: Vec<Vec2>,
    _easing: PhantomData<E>
}

impl<E: Easing> Spline<E> {
    fn new_unchecked(points: Vec<Vec2>) -> Self {
        Self { points, _easing: PhantomData }
    }

    pub fn new(points: impl IntoIterator<Item = Vec2>) -> Option<Self> {
        let mut points = points.into_iter().collect::<Vec<_>>();

        points.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).expect("no NAN values allowed"));

        points // TODO: instead of not allowing duplicate x values, maybe choose the center?
            .windows(2)
            .all(|w| w[0].x != w[1].x)
            .then_some(Self::new_unchecked(points))
    }

    pub fn sample(&self, x: f32) -> f32 {
        if self.points.is_empty() {
            return x;
        }

        let first = self.points.first().expect("can't be empty");

        if x <= first.x {
            return first.y;
        }

        let last = self.points.last().expect("can't be empty");

        if x >= last.x {
            return last.y;
        }

        for pts in self.points.windows(2) {
            let p0 = pts[0];
            let p1 = pts[1];

            if (p0.x..=p1.x).contains(&x) {
                let t = (x - p0.x) / (p1.x - p0.x);
                return glm::lerp_scalar(p0.y, p1.y, E::ease(t));
            }
        }

        unreachable!()
    }

    pub fn with_easing<T: Easing>(self) -> Spline<T> {
        Spline::new_unchecked(self.points)
    }
}
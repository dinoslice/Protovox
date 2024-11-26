use std::fmt;
use std::marker::PhantomData;
use glm::Vec2;

pub trait Easing {
    fn ease(x: f32) -> f32;
}

#[derive(Debug, Clone)]
pub struct Spline<E: Easing> {
    points: Vec<Vec2>,
    _easing: PhantomData<E>,
}

impl<E: Easing> Default for Spline<E> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<E: Easing> Spline<E> {
    pub fn empty() -> Self {
        Self::new_unchecked(Vec::<Vec2>::default())
    }

    fn new_unchecked(points: impl IntoIterator<Item = impl Into<Vec2>>) -> Self {
        Self { points: points.into_iter().map(Into::into).collect(), _easing: PhantomData }
    }

    pub fn new(points: impl IntoIterator<Item = impl Into<Vec2>>) -> Self {
        let mut spline = Self::new_unchecked(points);

        spline.sort().expect("no NAN values!");

        // TODO: instead of deleting duplicate x values, maybe choose center?
        spline.points.dedup_by(|a, b| a.x == b.x);

        spline
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

    pub fn points(&self) -> &Vec<Vec2> {
        &self.points
    }

    pub fn add(&mut self, point: Vec2) {
        self.mutate(|points| points.push(point));
    }

    pub fn mutate(&mut self, mut func: impl FnMut(&mut Vec<Vec2>)) {
        func(&mut self.points);
        self.sort().expect("no NAN values!");
    }

    pub fn remove_all(&mut self, predicate: impl FnMut(&Vec2) -> bool) -> Vec<Vec2> {
        let points = std::mem::take(&mut self.points);

        let (ret, keep) = points.into_iter().partition(predicate);

        self.points = keep;

        ret
    }

    pub fn remove_first(&mut self, predicate: impl FnMut(&Vec2) -> bool) -> Option<Vec2> {
        self.points.iter().position(predicate).map(|pos| self.points.remove(pos))
    }

    fn sort(&mut self) -> Option<()> {
        if self.points().iter().any(|p| p.x.is_nan()) {
            return None;
        }

        self.points.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).expect("no NAN values allowed"));

        Some(())
    }
}

impl<E: Easing> fmt::Display for Spline<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self.points()
            .iter()
            .map(|v| format!("[{}, {}]", v.x, v.y))
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "{str}")
    }
}
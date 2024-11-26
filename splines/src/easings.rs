use crate::Easing;

#[derive(Debug, Copy, Clone)]
pub struct InOutSine;

impl Easing for InOutSine {
    fn ease(x: f32) -> f32 {
        use std::f32::consts::PI;

        -(x * PI).cos() * 0.5 + 0.5
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InOutCubic;

impl Easing for InOutCubic {
    fn ease(x: f32) -> f32 {
        if x < 0.5 {
            4.0 * x * x * x
        } else {
            1.0 - f32::powi(-2.0 * x + 2.0, 3) * 0.5
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InOutQuint;

impl Easing for InOutQuint {
    fn ease(x: f32) -> f32 {
        if x < 0.5 {
            16.0 * x * x * x * x * x
        } else {
            1.0 - f32::powi(-2.0 * x + 2.0, 5) * 0.5
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InOutCirc;

impl Easing for InOutCirc {
    fn ease(x: f32) -> f32 {
        if x < 0.5 {
            (1.0 - f32::sqrt(1.0 - 4.0 * x * x)) * 0.5
        } else {
            (f32::sqrt(1.0 - f32::powi(-2.0 * x + 2.0, 2)) + 1.0) * 0.5
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InOutQuad;

impl Easing for InOutQuad {
    fn ease(x: f32) -> f32 {
        if x < 0.5 {
            2.0 * x * x
        } else {
            1.0 - f32::powi(-2.0 * x + 2.0, 2) * 0.5
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InOutQuart;

impl Easing for InOutQuart {
    fn ease(x: f32) -> f32 {
        if x < 0.5 {
            8.0 * x * x * x * x
        } else {
            1.0 - f32::powi(-2.0 * x + 2.0, 4) * 0.5
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InOutExpo;

impl Easing for InOutExpo {
    fn ease(x: f32) -> f32 {
        match x {
            0.0 => 0.0,
            1.0 => 1.0,
            _ if x < 0.5 => f32::powi(2.0, (20.0 * x - 10.0) as i32) * 0.5,
            _ => (2.0 - f32::powi(2.0, (-20.0 * x + 10.0) as i32)) * 0.5,
        }
    }
}
use std::time::Duration;
use glm::Vec3;
use rgb::Rgb;
use shipyard::Component;

#[derive(Clone, Component, Debug, PartialEq)]
pub struct LineGizmo {
    pub start: Vec3,
    pub end: Vec3,
    pub style: GizmoStyle,
    pub lifetime: GizmoLifetime,
}

#[derive(Clone, Component, Debug, PartialEq)]
pub struct SphereGizmo {
    pub center: Vec3,
    pub radius: f32,
    pub style: GizmoStyle,
    pub lifetime: GizmoLifetime,
}

#[derive(Clone, Component, Debug, PartialEq)]
pub struct BoxGizmo {
    pub min: Vec3,
    pub max: Vec3,
    pub style: GizmoStyle,
    pub lifetime: GizmoLifetime,
}

impl BoxGizmo {
    pub fn from_corners(a: Vec3, b: Vec3, style: GizmoStyle, lifetime: GizmoLifetime) -> Self {
        let min = glm::min2(&a, &b);
        let max = glm::max2(&a, &b);

        Self { min, max, style, lifetime }
    }
}

#[derive(Clone, Component, Debug, PartialEq)]
pub struct PointGizmo {
    pub center: Vec3,
    pub style: GizmoStyle,
    pub lifetime: GizmoLifetime,
}

pub type GizmoColor = Rgb<f32>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoStyle {
    pub stroke_color: GizmoColor,
    // pub stroke_width: f32, TODO: figure out how to draw line thickness

    pub fill_color: Option<GizmoColor>,
}

impl GizmoStyle {
    pub fn stroke(color: GizmoColor) -> Self {
        Self {
            stroke_color: color,
            fill_color: None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GizmoLifetime {
    SingleFrame,
    Persistent(Duration)
}
use glm::Vec2;
use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;

#[derive(Debug)]
pub struct MouseManager {
    pub pressed: bool,
    pub rotate: Vec2,
    pub scroll: f32,
    pub sensitivity: f32,
}

impl MouseManager {
    pub fn with_sensitivity(sensitivity: f32) -> Self {
        Self {
            pressed: false,
            rotate: Vec2::zeros(),
            scroll: 0.0,
            sensitivity,
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(
                PhysicalPosition { y: scroll, .. }
            ) => *scroll as f32,
        };
    }

    pub fn reset_scroll_rotate(&mut self) {
        self.rotate = Vec2::zeros();
        self.scroll = 0.0;
    }
}
use glm::Vec2;
use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;

#[derive(Debug)]
pub struct MouseManager {
    pub rotate: Vec2,
    pub scroll: f32,
    pub scroll_pixels_per_line: f32,
    pub sensitivity: f32,
}

impl MouseManager {
    pub fn new(sensitivity: f32, scroll_pixels_per_line: f32) -> Self {
        assert!(sensitivity >= 0.0);
        assert!(scroll_pixels_per_line >= 0.0);
        
        Self {
            rotate: Vec2::zeros(),
            scroll: 0.0,
            scroll_pixels_per_line,
            sensitivity,
        }
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll -= match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => *scroll,
            MouseScrollDelta::PixelDelta(
                PhysicalPosition { y: scroll, .. }
            ) => (*scroll as f32) / self.scroll_pixels_per_line,
        };
    }

    pub fn reset_scroll_rotate(&mut self) {
        self.rotate = Vec2::zeros();
        self.scroll = 0.0;
    }
}
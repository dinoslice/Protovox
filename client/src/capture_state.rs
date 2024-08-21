use shipyard::Unique;
use winit::window::{CursorGrabMode, Window};

#[derive(Unique)]
pub struct CaptureState(pub bool);

impl CaptureState {
    pub fn set(&mut self, window: &Window, captured: bool) -> Option<()> {
        self.0 = captured;

        let cursor_grab = match self.0 {
            true => CursorGrabMode::Confined,
            false => CursorGrabMode::None,
        };

        window.set_cursor_visible(!self.0);
        window.set_cursor_grab(cursor_grab).ok()
    }

    pub fn is_captured(&self) -> bool {
        self.0
    }

    pub fn toggle(&mut self, window: &Window) -> Option<bool> {
        self.set(window, !self.0).map(|_| self.0)
    }
}
use shipyard::{Borrow, BorrowInfo, UniqueViewMut};
use crate::application::CaptureState;
use crate::input::InputManager;
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Borrow, BorrowInfo)]
pub struct GuiBundle<'v> {
    pub g_ctx: UniqueViewMut<'v, GraphicsContext>,
    pub capture_state: UniqueViewMut<'v, CaptureState>,
    pub input: UniqueViewMut<'v, InputManager>,
}

impl GuiBundle<'_> {
    pub fn set_capture(&mut self, set: bool, reset_input: bool) {
        if self.capture_state.set(&self.g_ctx.window, set).is_none() {
            tracing::error!("Unable to set capture/release mouse cursor.")
        } else if !set && reset_input { // only reset action map if released cursor
            self.input.reset_all();
        }
    }
}
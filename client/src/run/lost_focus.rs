use shipyard::{Unique, UniqueOrDefaultViewMut, UniqueView, UniqueViewMut};
use tracing::error;
use engine::application::CaptureState;
use engine::input::InputManager;
use engine::rendering::graphics_context::GraphicsContext;

#[derive(Unique, Default)]
pub struct LostFocus {
    was_prev_captured: bool
}

pub fn on_window_focus_change(focused: bool, g_ctx: UniqueView<GraphicsContext>, mut lost_focus: UniqueOrDefaultViewMut<LostFocus>, mut capture_state: UniqueViewMut<CaptureState>, mut input_manager: UniqueViewMut<InputManager>) {
    if !focused || lost_focus.was_prev_captured {
        lost_focus.was_prev_captured = capture_state.is_captured();

        if capture_state.set(&g_ctx.window, focused).is_none() {
            error!("Unable to set capture/release mouse cursor.")
        } else if !focused { // only reset action map if released cursor
            input_manager.reset_all();
        }
    }
}
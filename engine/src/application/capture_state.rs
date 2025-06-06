use shipyard::{Unique, UniqueView, UniqueViewMut};
use tracing::error;
use winit::window::{CursorGrabMode, Window};
use crate::input::InputManager;
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Unique, Debug)]
pub struct CaptureState(bool);

impl CaptureState {
    // TODO: store Arc<Window>
    pub fn new_with(window: &Window, captured: bool) -> Option<Self> {
        let state = Self(captured);

        state.reconfigure_window(window);

        Some(state)
    }

    pub fn set(&mut self, window: &Window, captured: bool) -> Option<()> {
        self.0 = captured;

        self.reconfigure_window(window).map(|_| ())
    }

    pub fn is_captured(&self) -> bool {
        self.0
    }

    pub fn toggle(&mut self, window: &Window) -> Option<bool> {
        self.set(window, !self.0).map(|_| self.0)
    }

    pub fn reconfigure_window(&self, window: &Window) -> Option<bool> {
        let captured = self.is_captured();

        let cursor_grab = match self.0 {
            true => CursorGrabMode::Confined,
            false => CursorGrabMode::None,
        };

        window.set_cursor_visible(!self.0);
        window.set_cursor_grab(cursor_grab).ok()?;

        Some(captured)
    }
}

pub fn is_captured(capture: UniqueView<CaptureState>) -> bool {
    capture.is_captured()
}

pub fn is_not_captured(capture: UniqueView<CaptureState>) -> bool {
    !capture.is_captured()
}

pub fn toggle_captured(g_ctx: UniqueView<GraphicsContext>, mut capture_state: UniqueViewMut<CaptureState>, mut input: UniqueViewMut<InputManager>) {
    match capture_state.toggle(&g_ctx.window) {
        Some(false) => input.reset_all(),
        None => error!("Unable to set capture/release mouse cursor."),
        _ => {}
    }
}

pub fn set_from_focus(focused: bool, g_ctx: UniqueView<GraphicsContext>, mut capture_state: UniqueViewMut<CaptureState>, mut input_manager: UniqueViewMut<InputManager>) {
    if capture_state.set(&g_ctx.window, focused).is_none() {
        error!("Unable to set capture/release mouse cursor.")
    } else if !focused { // only reset action map if released cursor
        input_manager.reset_all();
    }
}
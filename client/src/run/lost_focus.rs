use shipyard::{AllStoragesView, Unique, UniqueOrDefaultViewMut, UniqueView, UniqueViewMut};
use tracing::error;
use engine::application::CaptureState;
use engine::application::pause::{IsPaused, ToggleGuiPressed};
use engine::input::InputManager;
use engine::rendering::graphics_context::GraphicsContext;
use engine::rendering::gui_bundle::GuiBundle;

pub fn on_window_focus_change(focused: bool, capture: UniqueView<CaptureState>, storages: AllStoragesView) {
    if capture.is_captured() && !focused {
        storages.add_unique(ToggleGuiPressed);
    }
}
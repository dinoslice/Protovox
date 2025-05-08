use shipyard::{AllStoragesView, Unique, UniqueView, UniqueViewMut};
use crate::input::action_map::Action;
use crate::input::InputManager;
use crate::rendering::gui_bundle::GuiBundle;

#[derive(Default, Unique)]
pub struct IsPaused(bool);

impl IsPaused {
    pub fn new(initial: bool) -> Self {
        Self(initial)
    }

    pub fn set_without_event(&mut self, set: bool) {
        self.0 = set;
    }

    pub fn is_paused(&self) -> bool {
        self.0
    }
}

pub fn is_paused(paused: UniqueView<IsPaused>) -> bool {
    paused.0
}

#[derive(Unique)]
pub struct ToggleGuiPressed;

pub fn listen_for_toggle_pause(input: UniqueView<InputManager>, storages: AllStoragesView) {
    if input.just_pressed().get_action(Action::ToggleGui) {
        storages.add_unique(ToggleGuiPressed);
    }
}

pub fn toggle_pause_menu(storages: AllStoragesView, mut pause: UniqueViewMut<IsPaused>, mut gui_bundle: GuiBundle) {
    let Ok(ToggleGuiPressed) = storages.remove_unique() else {
        return;
    };

    let prev = pause.is_paused();

    pause.set_without_event(!prev);
    gui_bundle.set_capture(!pause.is_paused(), true);
}
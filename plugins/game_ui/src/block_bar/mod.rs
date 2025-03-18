use shipyard::{AllStoragesView, UniqueView, UniqueViewMut};
use engine::input::action_map::Action;
use engine::input::InputManager;

mod render;
mod display;

pub use render::block_bar;
pub use display::BlockBarDisplay;
use crate::inventory::InventoryOpen;

pub fn process_block_bar(input: UniqueView<InputManager>, mut block_bar_display: UniqueViewMut<BlockBarDisplay>, inv_open: UniqueView<InventoryOpen>) {
    if !inv_open.0 && input.just_released().get_action(Action::ToggleBlockBar) {
        block_bar_display.toggle();
    }
}

pub fn scroll_block_bar(input: UniqueView<InputManager>, mut block_bar_display: UniqueViewMut<BlockBarDisplay>) {
    let scroll = input.mouse_manager.scroll.floor() as i32;

    block_bar_display.scroll(-scroll);
}

pub fn create_block_bar_display(storages: AllStoragesView) {
    storages.add_unique(BlockBarDisplay::Minimized { start: 0, selected: 0 });
}
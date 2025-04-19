use shipyard::{AllStoragesView, UniqueView, UniqueViewMut};
use engine::input::InputManager;

mod render;
mod display;

pub use render::block_bar;
pub use display::BlockBarDisplay;


pub fn scroll_block_bar(input: UniqueView<InputManager>, mut block_bar_display: UniqueViewMut<BlockBarDisplay>) {
    let scroll = input.mouse_manager.scroll.floor() as i32;

    block_bar_display.scroll(-scroll);
}

pub fn create_block_bar_display(storages: AllStoragesView) {
    storages.add_unique(BlockBarDisplay::Minimized { start: 0, selected: 0 });
}
mod render;

use std::time::{Duration, Instant};
use shipyard::{Unique, UniqueOrDefaultViewMut, UniqueViewMut};
use engine::input::action_map::Action;
pub use render::inventory;
use crate::block_bar::BlockBarDisplay;
use crate::gui_bundle::GuiBundle;

#[derive(Unique, Default)]
pub struct InventoryOpenTime(pub Option<Instant>);

#[derive(Unique, Default)]
pub struct InventoryOpen(pub bool);

#[derive(Unique, Default)]
pub struct PrevBlockBarState(pub bool);

pub fn open_inventory(
    mut open_time: UniqueOrDefaultViewMut<InventoryOpenTime>,
    mut open: UniqueViewMut<InventoryOpen>,
    mut block_bar_display: UniqueViewMut<BlockBarDisplay>,
    mut prev_block_bar_state: UniqueOrDefaultViewMut<PrevBlockBarState>,
    mut gui_bundle: GuiBundle,
) {
    let pressed = gui_bundle.input.pressed().get_action(Action::ToggleBlockBar);

    if !pressed {
        return;
    }

    let just_pressed = gui_bundle.input.just_pressed().get_action(Action::ToggleBlockBar);

    if open.0 {
        if just_pressed {
            open.0 = false;
            open_time.0 = None;

            if prev_block_bar_state.0 { // one ! for toggle, another ! since it tries to toggle block bar
                block_bar_display.toggle();
            }

            gui_bundle.set_capture(true, false);
        }
    } else {
        if just_pressed {
            open_time.0 = Some(Instant::now());
        }

        if matches!(open_time.0, Some(t) if pressed && t.elapsed() > Duration::from_secs_f32(0.5)) {
            open.0 = true;
            open_time.0 = None;

            prev_block_bar_state.0 = matches!(block_bar_display.as_ref(), BlockBarDisplay::Expanded { .. });

            if !prev_block_bar_state.0 {
                block_bar_display.toggle();
            }

            gui_bundle.set_capture(false, false);
        }
    }
}
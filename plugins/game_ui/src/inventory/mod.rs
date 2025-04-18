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

pub fn toggle_inv_block_bar(
    mut open_time: UniqueOrDefaultViewMut<InventoryOpenTime>,
    mut open: UniqueViewMut<InventoryOpen>,
    mut block_bar_display: UniqueViewMut<BlockBarDisplay>,
    mut prev_block_bar_state: UniqueOrDefaultViewMut<PrevBlockBarState>,
    mut gui_bundle: GuiBundle,
) {
    let just_rel = gui_bundle.input.just_released().get_action(Action::ToggleBlockBar);
    
    if !gui_bundle.input.pressed().get_action(Action::ToggleBlockBar) {
        if !open.0 && just_rel {
            block_bar_display.toggle();
        }
        
        return;
    }

    let just_pressed = gui_bundle.input.just_pressed().get_action(Action::ToggleBlockBar);

    if open.0 {
        // closing the inventory / block bar
        if just_pressed {
            open.0 = false;
            open_time.0 = None;

            // TODO: this has some weird behavior
            if prev_block_bar_state.0 { // one ! for toggle, another ! since it tries to toggle block bar
                block_bar_display.toggle();
            }

            gui_bundle.set_capture(true, false);
        }
    } else {
        if just_pressed {
            open_time.0 = Some(Instant::now());
        }

        if matches!(open_time.0, Some(t) if t.elapsed() > Duration::from_secs_f32(0.25)) {
            open.0 = true;
            open_time.0 = None;

            prev_block_bar_state.0 = matches!(block_bar_display.as_ref(), BlockBarDisplay::Expanded { .. });

            if !prev_block_bar_state.0 {
                block_bar_display.toggle();
            }

            gui_bundle.set_capture(false, false);
        } else if just_rel {
            block_bar_display.toggle();
        }
    }
}
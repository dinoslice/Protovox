mod render;
pub mod hand;

use std::time::{Duration, Instant};
use egui::Align2;
use shipyard::{IntoIter, Unique, UniqueOrDefaultViewMut, UniqueView, UniqueViewMut, View, ViewMut};
use egui_systems::CurrentEguiFrame;
use engine::block_bar_focus::BlockBarFocus;
use engine::components::LocalPlayer;
use engine::input::action_map::Action;
use engine::input::InputManager;
use engine::inventory::Inventory;
use crate::block_bar::BlockBarDisplay;
use crate::egui_views::EguiTextureAtlasViews;
use crate::gui_bundle::GuiBundle;
use crate::inventory::hand::Hand;
use crate::inventory::render::InventoryGui;

#[derive(Unique, Default)]
pub struct InventoryOpenTime(pub Option<Instant>);

#[derive(Unique, Default)]
pub struct InventoryOpen(pub bool);

#[derive(Unique, Default)]
pub struct PrevBlockBarState(pub bool);

pub fn inventory(
    egui_frame: UniqueView<CurrentEguiFrame>,
    local_player: View<LocalPlayer>,
    mut inventory: ViewMut<Inventory>,
    mut block_bar_focus: UniqueViewMut<BlockBarFocus>,
    texture_atlas_views: UniqueView<EguiTextureAtlasViews>,
    input_manager: UniqueView<InputManager>,
    open: UniqueView<InventoryOpen>,
    mut hand: UniqueViewMut<Hand>,
) {
    let (inventory, ..) = (&mut inventory, &local_player).iter()
        .next()
        .expect("LocalPlayer should exist");

    if !open.0 {
        return;
    }

    egui::Area::new("inventory".into())
        .anchor(Align2::RIGHT_CENTER, [-100.0, 0.0])
        .show(egui_frame.ctx(), |ui| {
            ui.add(InventoryGui {
                inventory,
                texture_atlas_views: &texture_atlas_views,
                block_bar_focus_input: Some((&mut block_bar_focus, &input_manager)),
                hand: &mut hand,
                columns: 6,
            })
        });
}

pub fn toggle_inv_block_bar(
    mut open_time: UniqueOrDefaultViewMut<InventoryOpenTime>,
    mut open: UniqueViewMut<InventoryOpen>,
    mut block_bar_display: UniqueViewMut<BlockBarDisplay>,
    mut prev_block_bar_state: UniqueOrDefaultViewMut<PrevBlockBarState>,
    mut gui_bundle: GuiBundle,
) {
    let just_rel = gui_bundle.input.just_released().get_action(Action::ToggleBlockBar);
    
    if !gui_bundle.input.pressed().get_action(Action::ToggleBlockBar) {
        if !open.0 && just_rel && gui_bundle.capture_state.is_captured() {
            block_bar_display.toggle();
        }
        
        return;
    }

    let just_pressed = gui_bundle.input.just_pressed().get_action(Action::ToggleBlockBar);

    if open.0 {
        // closing the inventory
        if just_pressed {
            open.0 = false;
            open_time.0 = None;

            // TODO: this has some weird behavior
            if prev_block_bar_state.0 { // one ! for toggle, another ! since it tries to toggle block bar
                block_bar_display.toggle();
            }

            gui_bundle.set_capture(true, false);
        }
    } else if gui_bundle.capture_state.is_captured() {
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
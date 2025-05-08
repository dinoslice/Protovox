mod render;
pub mod hand;

use std::time::{Duration, Instant};
use egui::{Align2, Area};
use shipyard::{AllStoragesView, IntoIter, Unique, UniqueView, UniqueViewMut, View, ViewMut};
use egui_systems::CurrentEguiFrame;
use engine::application::pause::ToggleGuiPressed;
use engine::block_bar_focus::BlockBarFocus;
use engine::chunks::chunk_manager::ChunkManager;
use engine::components::LocalPlayer;
use engine::input::action_map::Action;
use engine::input::InputManager;
use engine::interact::CurrentlyFocusedBlock;
use engine::inventory::PlayerInventory;
use engine::rendering::gui_bundle::GuiBundle;
use game::block::Block;
use game::inventory::Inventory;
use crate::block_bar::BlockBarDisplay;
use crate::egui_views::EguiTextureAtlasViews;
use crate::inventory::hand::InventoryHand;
use crate::inventory::render::InventoryGui;

#[derive(Unique)]
pub struct InventoryOpenTime(pub Option<Instant>);

#[derive(Unique)]
pub struct InventoryOpen(pub bool);

#[derive(Unique)]
pub struct PrevBlockBarState(pub bool);

#[derive(Unique)]
pub struct ReturnHandEvent;

pub fn initialize(storages: AllStoragesView) {
    storages.add_unique(InventoryOpen(false));
    storages.add_unique(InventoryHand(None));
    storages.add_unique(InventoryOpenTime(None));
    storages.add_unique(PrevBlockBarState(false));
}

pub fn inventory(
    egui_frame: UniqueView<CurrentEguiFrame>,
    local_player: View<LocalPlayer>,
    mut inventory: ViewMut<PlayerInventory>,
    mut block_bar_focus: UniqueViewMut<BlockBarFocus>,
    texture_atlas_views: UniqueView<EguiTextureAtlasViews>,
    input_manager: UniqueView<InputManager>,
    open: UniqueView<InventoryOpen>,
    mut hand: UniqueViewMut<InventoryHand>,
    mut world: UniqueViewMut<ChunkManager>,
    focused_inv: UniqueView<CurrentlyFocusedBlock>,
) {
    let (inventory, ..) = (&mut inventory, &local_player).iter()
        .next()
        .expect("LocalPlayer should exist");

    if !open.0 {
        return;
    }

    Area::new("inventory".into())
        .anchor(Align2::RIGHT_CENTER, [-100.0, 0.0])
        .show(egui_frame.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.add(InventoryGui {
                    inventory,
                    texture_atlas_views: &texture_atlas_views,
                    block_bar_focus_input: Some((&mut block_bar_focus, &input_manager)),
                    hand: &mut hand,
                    columns: 6,
                    id: "player_inventory",
                });

                if let Some(location) = &focused_inv.as_ref().0 {
                    if let Some(Block::Crate { inventory }) = world.get_block_mut(location) {
                        ui.add_space(10.0);

                        ui.add(InventoryGui {
                            inventory,
                            texture_atlas_views: &texture_atlas_views,
                            block_bar_focus_input: None,
                            hand: &mut hand,
                            columns: 6,
                            id: "crate_ui",
                        });
                    }
                }
            });
        });
}

pub fn toggle_inv_block_bar(
    mut open_time: UniqueViewMut<InventoryOpenTime>,
    mut open: UniqueViewMut<InventoryOpen>,
    mut block_bar_display: UniqueViewMut<BlockBarDisplay>,
    mut prev_block_bar_state: UniqueViewMut<PrevBlockBarState>,
    mut gui_bundle: GuiBundle,
    storages: AllStoragesView,
) {
    fn close(
        mut open_time: UniqueViewMut<InventoryOpenTime>,
        mut open: UniqueViewMut<InventoryOpen>,
        mut block_bar_display: UniqueViewMut<BlockBarDisplay>,
        mut prev_block_bar_state: UniqueViewMut<PrevBlockBarState>,
        mut gui_bundle: GuiBundle,
        storages: AllStoragesView,
    ) {
        open.0 = false;

        storages.add_unique(ReturnHandEvent);

        open_time.0 = None;

        // TODO: this has some weird behavior
        if prev_block_bar_state.0 { // one ! for toggle, another ! since it tries to toggle block bar
            block_bar_display.toggle();
        }

        gui_bundle.set_capture(true, false);
    }

    if open.0 && storages.remove_unique::<ToggleGuiPressed>().is_ok() {
        close(open_time, open, block_bar_display, prev_block_bar_state, gui_bundle, storages);
        return;
    }

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
            close(open_time, open, block_bar_display, prev_block_bar_state, gui_bundle, storages);
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

pub fn return_hand(v_local_player: View<LocalPlayer>, mut vm_inventory: ViewMut<PlayerInventory>, mut hand: UniqueViewMut<InventoryHand>, storages: AllStoragesView) {
    let Ok(ReturnHandEvent) = storages.remove_unique() else {
        return;
    };

    let (inventory, ..) = (&mut vm_inventory, &v_local_player).iter()
        .next()
        .expect("LocalPlayer should exist");
    
    if let Some(it) = hand.0.take() {
        if let Some(residual) = inventory.try_insert(it) {
            tracing::warn!("TODO: couldn't insert remaining {residual:?} into player inventory from hand upon closing inventory");
        }
    }
}
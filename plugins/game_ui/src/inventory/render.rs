use std::mem;
use std::num::NonZeroU8;
use egui::{Align2, Color32, FontId, Frame, Grid, PointerButton, Response, Sense, Stroke, Ui, Vec2, Widget};
use engine::block_bar_focus::BlockBarFocus;
use engine::input::action_map::Action;
use engine::input::InputManager;
use engine::inventory::Inventory;
use game::item::ItemStack;
use crate::egui_views::EguiTextureAtlasViews;
use crate::inventory::hand::InventoryHand;
use crate::item_stack::ItemStackRender;

pub struct InventoryGui<'a> {
    pub inventory: &'a mut Inventory,
    pub texture_atlas_views: &'a EguiTextureAtlasViews,
    pub block_bar_focus_input: Option<(&'a mut BlockBarFocus, &'a InputManager)>,
    pub hand: &'a mut InventoryHand,
    pub columns: usize,
}

impl Widget for InventoryGui<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            inventory,
            texture_atlas_views,
            mut block_bar_focus_input,
            hand,
            columns,
        } = self;
        
        Frame::none()
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                let mut block_bar_focus_selected = Vec::with_capacity(
                    block_bar_focus_input.as_ref().map_or(0, |(bbf, _)| bbf.focus.len())
                );

                Grid::new("inventory_grid")
                    .show(ui, |ui| {
                        for (row_idx, row) in inventory.as_mut_slice().chunks_mut(columns).enumerate() {
                            for (col_idx, slot) in row.iter_mut().enumerate() {
                                let i = row_idx * columns + col_idx;

                                let response = Frame::none()
                                    .stroke(Stroke::new(2.0, Color32::GRAY))
                                    .fill(Color32::from_rgba_unmultiplied(128, 128, 128, 175))
                                    .show(ui, |ui| {
                                        ui.style_mut()
                                            .visuals
                                            .override_text_color = Some(Color32::from_rgb(230, 230, 230));

                                        ui.set_height(40.0);
                                        ui.set_width(40.0);

                                        ui.centered_and_justified(|ui| {
                                            let (rect, response) = ui.allocate_exact_size(Vec2::splat(35.0), Sense::click());

                                            if let Some(it) = slot {
                                                ItemStackRender { it, atlas: texture_atlas_views, rect }.ui(ui);
                                            }

                                            if response.clicked_by(PointerButton::Primary) {
                                                Self::interact_hand_inventory_slot(&mut hand.0, slot, PointerButton::Primary);
                                            } else if response.clicked_by(PointerButton::Secondary) {
                                                Self::interact_hand_inventory_slot(&mut hand.0, slot, PointerButton::Secondary);
                                            }
                                        });
                                    });
                                
                                if let Some((block_bar_focus, input_manager)) = block_bar_focus_input.as_mut() {
                                    if let Some(i) = block_bar_focus
                                        .focus
                                        .iter()
                                        .position(|&slot| slot == Some(i))
                                    {
                                        block_bar_focus_selected.push((i, response.response.rect));
                                    }

                                    if response.response.hovered() {
                                        for (bar_slot, &action) in Action::BLOCK_BAR.iter().enumerate() {
                                            if input_manager.just_pressed().get_action(action) {
                                                let slot = block_bar_focus.focus
                                                    .get_mut(bar_slot)
                                                    .expect("should be in range");

                                                if *slot == Some(i) {
                                                    *slot = None;
                                                } else {
                                                    *slot = Some(i);
                                                }

                                                block_bar_focus
                                                    .focus
                                                    .iter_mut()
                                                    .enumerate()
                                                    .filter(|(j, &mut focus)| *j != bar_slot && focus == Some(i))
                                                    .for_each(|(_, slot)| *slot = None);
                                            }
                                        }
                                    }
                                }

                            }

                            ui.end_row();
                        }
                    });

                let font_id = FontId::proportional(16.0);

                for (i, rect) in block_bar_focus_selected {
                    let i = i + 1;

                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(2.0, Color32::LIGHT_RED),
                    );

                    let text_pos = rect.left_top() + Vec2::splat(2.0);

                    ui.painter().text(
                        text_pos + Vec2::splat(0.75),
                        Align2::LEFT_TOP,
                        i,
                        font_id.clone(),
                        Color32::BLACK,
                    );

                    ui.painter().text(
                        text_pos,
                        Align2::LEFT_TOP,
                        i,
                        font_id.clone(),
                        Color32::LIGHT_RED,
                    );
                }
            }).response
    }
}

impl InventoryGui<'_> {
    fn interact_hand_inventory_slot(hand: &mut Option<ItemStack>, slot: &mut Option<ItemStack>, button: PointerButton) {
        match (&hand, &slot, button) {
            (None, Some(_), PointerButton::Secondary) => {
                let s = slot.take().expect("should've matched on Some");
                
                let (hand_it, slot_it) = s.split_half();
                
                *hand = Some(hand_it);
                *slot = slot_it;
            }
            (Some(hand_it), slot_it, PointerButton::Secondary)
                if slot_it.as_ref().is_none_or(|slot_it| slot_it.item == hand_it.item) =>
            {
                let h = hand.take().expect("should've matched on Some");
                
                let (slot_it, mut hand_it) = h.split(NonZeroU8::new(1).expect("not zero"));
                
                match slot {
                    Some(slot) => {
                        if let Some(residual) = slot.try_combine(slot_it) {
                            if let Some(hand_it) = &mut hand_it {
                                let res = hand_it.try_combine(residual);
                                
                                assert!(res.is_none(), "should be eq and have space, since res originally came from hand");
                            } else {
                                hand_it = Some(residual);
                            }
                        }
                    }
                    None => {
                        *slot = Some(slot_it);
                    }
                }
    
                *hand = hand_it;
            }
            (Some(hand_it), Some(slot_it), PointerButton::Primary) if slot_it.item == hand_it.item => {
                let hand_it = hand.take().expect("should've matched on Some");
                let slot = slot.as_mut().expect("should've matched on Some");
                
                *hand = slot.try_combine(hand_it);
            }
            (_, _, PointerButton::Primary) | (_, _, PointerButton::Secondary) => mem::swap(hand, slot),
            /* do nothing on middle or extra buttons */
            _ => tracing::warn!("InventoryGui::interact_hand_inventory_slot should only be called with PointerButton::{{Primary, Secondary}}"),
        }
    }
}


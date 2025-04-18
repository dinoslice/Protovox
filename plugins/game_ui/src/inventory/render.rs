use std::mem;
use egui::{Align2, Color32, Grid, Image, Sense, Vec2};
use egui::load::SizedTexture;
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use egui_systems::CurrentEguiFrame;
use engine::block_bar_focus::BlockBarFocus;
use engine::components::LocalPlayer;
use engine::input::action_map::Action;
use engine::input::InputManager;
use engine::inventory::Inventory;
use crate::egui_views::EguiTextureAtlasViews;
use crate::inventory::hand::Hand;
use crate::inventory::InventoryOpen;
use crate::item_stack::ItemStackRender;

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

    const COLUMNS: usize = 3;

    egui::Area::new("inventory".into())
        .anchor(Align2::RIGHT_CENTER, [-100.0, 0.0])
        .show(egui_frame.ctx(), |ui| {
            egui::Frame::none()
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;

                    let mut selected = Vec::with_capacity(block_bar_focus.focus.len());

                    Grid::new("inventory_grid")
                        .show(ui, |ui| {
                            for (row_idx, row) in inventory.as_mut_slice().chunks_mut(COLUMNS).enumerate() {
                                for (col_idx, slot) in row.iter_mut().enumerate() {
                                    let i = row_idx * COLUMNS + col_idx;

                                    let response = egui::Frame::none()
                                        .stroke(egui::Stroke::new(2.0, egui::Color32::GRAY))
                                        .fill(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 175))
                                        .show(ui, |ui| {
                                            ui.style_mut()
                                                .visuals
                                                .override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));

                                            ui.set_height(40.0);
                                            ui.set_width(40.0);

                                            ui.centered_and_justified(|ui| {
                                                let (rect, response) = ui.allocate_exact_size(Vec2::splat(35.0), Sense::click());
                                                
                                                if let Some(it) = slot {
                                                    ItemStackRender { it, atlas: &texture_atlas_views, rect }.ui(ui);
                                                }
                                                
                                                if response.clicked() {
                                                    mem::swap(slot, &mut hand.0);
                                                }
                                            });
                                        });

                                    if let Some(i) = block_bar_focus
                                        .focus
                                        .iter()
                                        .position(|&slot| slot == Some(i))
                                    {
                                        selected.push((i, response.response.rect));
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

                                ui.end_row();
                            }
                        });

                    let painter = ui.painter();
                    let font_id = egui::FontId::proportional(16.0);

                    for (i, rect) in selected {
                        let i = i + 1;

                        painter.rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::LIGHT_RED),
                        );

                        let text_pos = rect.left_top() + Vec2::splat(2.0);

                        painter.text(
                            text_pos + Vec2::splat(0.75),
                            Align2::LEFT_TOP,
                            i,
                            font_id.clone(),
                            egui::Color32::BLACK,
                        );

                        painter.text(
                            text_pos,
                            Align2::LEFT_TOP,
                            i,
                            font_id.clone(),
                            egui::Color32::LIGHT_RED,
                        );
                    }
                })
        });
}
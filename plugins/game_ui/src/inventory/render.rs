use egui::{Align2, Grid, Vec2};
use egui::load::SizedTexture;
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use egui_systems::CurrentEguiFrame;
use engine::block_bar_focus::BlockBarFocus;
use engine::components::LocalPlayer;
use engine::input::action_map::Action;
use engine::input::InputManager;
use engine::inventory::Inventory;
use crate::egui_views::EguiTextureAtlasViews;
use crate::inventory::InventoryOpen;

pub fn inventory(egui_frame: UniqueView<CurrentEguiFrame>, local_player: View<LocalPlayer>, inventory: View<Inventory>, mut block_bar_focus: UniqueViewMut<BlockBarFocus>, texture_atlas_views: UniqueView<EguiTextureAtlasViews>, input_manager: UniqueView<InputManager>, open: UniqueView<InventoryOpen>) {
    let (inventory, ..) = (&inventory, &local_player).iter()
        .next()
        .expect("LocalPlayer should exist");

    if !open.0 {
        return;
    }

    const COLUMNS: usize = 3;

    egui::Area::new("inventory".into())
        .anchor(Align2::RIGHT_CENTER, [-100.0, 0.0])
        .movable(true)
        .show(egui_frame.ctx(), |ui| {
            egui::Frame::none()
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;

                    let mut selected = Vec::with_capacity(block_bar_focus.focus.len());

                    Grid::new("inventory_grid")
                        .show(ui, |ui| {
                            for (row_idx, row) in inventory.as_slice().chunks(COLUMNS).enumerate() {
                                for (col_idx, slot) in row.iter().enumerate() {
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
                                                if let Some(it) = slot {
                                                    let texture = texture_atlas_views
                                                        .get_from_texture_id(it.ty.texture_id())
                                                        .expect("should have a texture");

                                                    let size = Vec2::splat(35.0);

                                                    let image_response = ui.image(SizedTexture { id: texture, size });

                                                    let painter = ui.painter();
                                                    let rect = image_response.rect;

                                                    let text = it.count.to_string();
                                                    let text_pos = rect.right_bottom() - Vec2::splat(10.0);

                                                    let font_id = egui::FontId::proportional(16.0);

                                                    // shadow
                                                    painter.text(
                                                        text_pos + Vec2::splat(1.2),
                                                        Align2::RIGHT_BOTTOM,
                                                        &text,
                                                        font_id.clone(),
                                                        egui::Color32::BLACK,
                                                    );

                                                    painter.text(
                                                        text_pos,
                                                        Align2::RIGHT_BOTTOM,
                                                        text,
                                                        font_id,
                                                        egui::Color32::WHITE,
                                                    );
                                                } else {
                                                    ui.label(format!("{i}"));
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
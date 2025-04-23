use egui::{Align2, Area, Color32, Frame, Sense, Stroke, Vec2};
use shipyard::{IntoIter, UniqueView, View};
use egui_systems::CurrentEguiFrame;
use engine::block_bar_focus::BlockBarFocus;
use engine::components::LocalPlayer;
use engine::inventory::PlayerInventory;
use game::inventory::Inventory;
use crate::block_bar::BlockBarDisplay;
use crate::egui_views::EguiTextureAtlasViews;
use crate::item_stack::ItemStackRender;

pub fn block_bar(egui_frame: UniqueView<CurrentEguiFrame>, local_player: View<LocalPlayer>, inventory: View<PlayerInventory>, inv_display: UniqueView<BlockBarDisplay>, block_bar_focus: UniqueView<BlockBarFocus>, texture_atlas_views: UniqueView<EguiTextureAtlasViews>) {
    let (inventory, ..) = (&inventory, &local_player).iter()
        .next()
        .expect("LocalPlayer should exist");

    const OFFSET: f32 = 10.0;

    Area::new("hotbar".into())
        .anchor(Align2::RIGHT_BOTTOM, [-OFFSET, -OFFSET])
        .show(egui_frame.ctx(), |ui| {
            Frame::none()
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.spacing_mut()
                            .item_spacing = Vec2::ZERO;

                        for (i, selected) in inv_display.visible()  {
                            let frame = if selected {
                                Frame::none()
                                    .stroke(Stroke::new(2.0, Color32::GRAY))
                                    .fill(Color32::from_rgba_unmultiplied(100, 100, 100, 175))
                            } else {
                                Frame::none()
                                    .stroke(Stroke::new(2.0, Color32::GRAY))
                                    .fill(Color32::from_rgba_unmultiplied(128, 128, 128, 175))
                            };

                            let _ = frame.show(ui, |ui| {
                                ui.style_mut()
                                    .visuals
                                    .override_text_color = Some(Color32::from_rgb(230, 230, 230));

                                if selected {
                                    ui.set_height(50.0);
                                    ui.set_width(50.0);
                                } else {
                                    ui.set_height(40.0);
                                    ui.set_width(40.0);
                                }

                                ui.centered_and_justified(|ui| {
                                    if let Some(it) = block_bar_focus
                                        .focus
                                        .get(i as usize)
                                        .expect("inv_display indices are within focus")
                                        .map(|inv_index|
                                            inventory
                                                .as_slice()
                                                .get(inv_index)
                                                .expect("focus must be in range of inventory")
                                                .as_ref()
                                        )
                                        .flatten()
                                    {
                                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(35.0), Sense::click());

                                        ItemStackRender { it, atlas: &texture_atlas_views, rect }.ui(ui);
                                    }
                                });
                            }).response;
                        }
                    })
                });
        });
}
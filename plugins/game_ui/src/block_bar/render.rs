use egui::Align2;
use egui::load::SizedTexture;
use shipyard::{IntoIter, UniqueView, View};
use egui_systems::CurrentEguiFrame;
use engine::components::LocalPlayer;
use engine::inventory::Inventory;
use crate::block_bar::BlockBarDisplay;
use crate::egui_views::EguiTextureAtlasViews;

pub fn block_bar(egui_frame: UniqueView<CurrentEguiFrame>, local_player: View<LocalPlayer>, inventory: View<Inventory>, inv_display: UniqueView<BlockBarDisplay>, texture_atlas_views: UniqueView<EguiTextureAtlasViews>) {
    let (inventory, ..) = (&inventory, &local_player).iter()
        .next()
        .expect("LocalPlayer should exist");

    let inv = inventory.items().collect::<Vec<_>>();

    const OFFSET: f32 = 10.0;

    egui::Area::new("hotbar".into())
        .anchor(Align2::RIGHT_BOTTOM, [-OFFSET, -OFFSET])
        .show(egui_frame.ctx(), |ui| {
            egui::Frame::none()
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.spacing_mut()
                            .item_spacing = egui::vec2(0.0, 0.0);

                        for (i, selected) in inv_display.visible()  {
                            let frame = if selected {
                                egui::Frame::none()
                                    .stroke(egui::Stroke::new(2.0, egui::Color32::GRAY))
                                    .fill(egui::Color32::from_rgba_unmultiplied(100, 100, 100, 175))
                            } else {
                                egui::Frame::none()
                                    .stroke(egui::Stroke::new(2.0, egui::Color32::GRAY))
                                    .fill(egui::Color32::from_rgba_unmultiplied(128, 128, 128, 175))
                            };

                            frame.show(ui, |ui| {
                                ui.style_mut()
                                    .visuals
                                    .override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));

                                if selected {
                                    ui.set_height(50.0);
                                    ui.set_width(50.0);
                                } else {
                                    ui.set_height(40.0);
                                    ui.set_width(40.0);
                                }

                                ui.centered_and_justified(|ui| {
                                    if let Some(it) = inv.get(i as usize) {
                                        let texture = texture_atlas_views
                                            .get_from_texture_id(it.ty.texture_id())
                                            .expect("should have a texture");

                                        let size = egui::Vec2::splat(35.0);

                                        ui.image(SizedTexture { id: texture, size });
                                    }
                                });
                            }).response;
                        }
                    })
                });
        });
}
use egui::{Align2, Area, Image, Order, Vec2};
use egui::load::SizedTexture;
use shipyard::{Unique, UniqueView};
use egui_systems::CurrentEguiFrame;
use game::item::ItemStack;
use crate::egui_views::EguiTextureAtlasViews;

#[derive(Unique, Default)]
pub struct Hand(pub Option<ItemStack>);

pub fn render_hand(egui_frame: UniqueView<CurrentEguiFrame>, hand: UniqueView<Hand>, texture_atlas_views: UniqueView<EguiTextureAtlasViews>) {
    if let Some(cursor_pos) = egui_frame.ctx().pointer_latest_pos() {
        let size = Vec2::splat(35.0);

        Area::new("hand".into())
            .fixed_pos(cursor_pos - size / 2.0)
            .order(Order::Foreground) // make sure it's above other UI
            .interactable(false)
            .show(egui_frame.ctx(), |ui| {
                if let Some(it) = &hand.0 {
                    let texture = texture_atlas_views
                        .get_from_texture_id(it.ty.texture_id())
                        .expect("should have a texture");

                    let rect = egui::Rect::from_min_size(cursor_pos - size / 2.0, size);

                    Image::new(SizedTexture { id: texture, size })
                        .paint_at(ui, rect);

                    let painter = ui.painter();
                    
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
                }
            });
    }
}
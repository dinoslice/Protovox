use crate::egui_views::EguiTextureAtlasViews;
use crate::item_stack::ItemStackRender;
use egui::{Area, Order, Rect, Vec2};
use egui_systems::CurrentEguiFrame;
use game::item::ItemStack;
use shipyard::{Unique, UniqueView};

#[derive(Unique, Default)]
pub struct InventoryHand(pub Option<ItemStack>);

pub fn render_hand(egui_frame: UniqueView<CurrentEguiFrame>, hand: UniqueView<InventoryHand>, texture_atlas_views: UniqueView<EguiTextureAtlasViews>) {
    if let (Some(cursor_pos), Some(it)) = (egui_frame.ctx().pointer_latest_pos(), &hand.0) {
        let size = Vec2::splat(35.0);

        Area::new("hand".into())
            .fixed_pos(cursor_pos - size / 2.0)
            .order(Order::Foreground) // make sure it's above other UI
            .interactable(false)
            .show(egui_frame.ctx(), |ui| {
                let rect = Rect::from_min_size(cursor_pos - size / 2.0, size);

                ItemStackRender { it, atlas: &texture_atlas_views, rect }.ui(ui);
            });
    }
}
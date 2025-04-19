use egui::{Align2, Color32, FontId, Image, Rect, Ui, Vec2};
use egui::load::SizedTexture;
use game::item::ItemStack;
use crate::egui_views::EguiTextureAtlasViews;

pub struct ItemStackRender<'a> {
    pub it: &'a ItemStack,
    pub atlas: &'a EguiTextureAtlasViews,
    pub rect: Rect,
}

impl ItemStackRender<'_> {
    pub fn ui(self, ui: &mut Ui) {
        let texture = self.atlas
            .get_from_texture_id(self.it.item.ty.texture_id())
            .expect("should have a texture");

        let size = self.rect.size();

        assert_eq!(size.x, size.y, "must be a square");

        Image::new(SizedTexture { id: texture, size })
            .paint_at(ui, self.rect);

        let painter = ui.painter();

        let text = self.it.count.to_string();
        let text_pos = self.rect.right_bottom() - Vec2::splat(size.x * 0.15);

        let font_id = FontId::proportional(size.x / 2.0);

        // shadow
        painter.text(
            text_pos + Vec2::splat(size.x * 0.035),
            Align2::RIGHT_BOTTOM,
            &text,
            font_id.clone(),
            Color32::BLACK,
        );

        painter.text(
            text_pos,
            Align2::RIGHT_BOTTOM,
            text,
            font_id,
            Color32::WHITE,
        );
    }
}
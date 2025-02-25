use egui::epaint;
use shipyard::{AllStoragesView, Unique, UniqueView, UniqueViewMut};
use egui_systems::EguiRenderer;
use engine::rendering::graphics_context::GraphicsContext;
use engine::rendering::texture_atlas::TextureAtlas;
use game::texture_ids::TextureId;

#[derive(Unique)]
pub struct EguiTextureAtlasViews(Box<[epaint::TextureId]>);

impl EguiTextureAtlasViews {
    fn from_texture_atlas(texture_atlas: &TextureAtlas, g_ctx: &GraphicsContext, egui_renderer: &mut EguiRenderer) -> Self {
        let mut texture_ids = Vec::with_capacity(texture_atlas.num_textures);

        for i in 0..texture_atlas.num_textures {
            let view = texture_atlas.texture_atlas.texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: i as _,
                .. Default::default()
            });

            texture_ids.push(egui_renderer.register_native_texture(&g_ctx.device, &view, wgpu::FilterMode::Nearest));
        }

        Self(texture_ids.into())
    }

    pub fn get_from_texture_id(&self, id: TextureId) -> Option<epaint::TextureId> {
        self.0.get(id as usize).copied()
    }
}

pub fn initialize_texture_atlas_views(g_ctx: UniqueView<GraphicsContext>, texture_atlas: UniqueView<TextureAtlas>, mut egui_renderer: UniqueViewMut<EguiRenderer>, all_storages: AllStoragesView) {
    all_storages.add_unique(EguiTextureAtlasViews::from_texture_atlas(&texture_atlas, &g_ctx, &mut egui_renderer))
}
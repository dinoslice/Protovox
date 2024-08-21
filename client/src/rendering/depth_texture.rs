use shipyard::{AllStoragesView, Unique, UniqueView};
use crate::rendering::texture::Texture;
use crate::rendering::graphics_context::GraphicsContext;

#[derive(Unique)]
pub struct DepthTexture(pub Texture);

pub fn initialize_depth_texture(g_ctx: UniqueView<GraphicsContext>, storages: AllStoragesView) {
    let depth_texture = Texture::create_depth_texture(&g_ctx.device, &g_ctx.config, "depth texture");
    storages.add_unique(DepthTexture(depth_texture));
}
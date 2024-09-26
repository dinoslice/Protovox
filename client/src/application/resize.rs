use shipyard::UniqueViewMut;
use crate::camera::Camera;
use crate::rendering::depth_texture::DepthTexture;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::texture::Texture;

pub fn resize(new_size: winit::dpi::PhysicalSize<u32>, mut g_ctx: UniqueViewMut<GraphicsContext>, mut camera: UniqueViewMut<Camera>, mut depth_texture: UniqueViewMut<DepthTexture>) {
    if new_size.width > 0 && new_size.height > 0 {
        g_ctx.resize(new_size);
        camera.perspective.set_aspect(g_ctx.aspect());

        let new_depth_texture = Texture::create_depth_texture(&g_ctx.device, &g_ctx.config, "depth texture");
        *depth_texture = DepthTexture(new_depth_texture);
    } else {
        tracing::warn!("Ignoring resize with non-positive width or height");
    }
}

pub fn reconfigure(g_ctx: UniqueViewMut<GraphicsContext>, camera: UniqueViewMut<Camera>, depth_texture: UniqueViewMut<DepthTexture>) {
    let size = g_ctx.size;
    resize(size, g_ctx, camera, depth_texture);
}
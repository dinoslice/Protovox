use shipyard::{IntoIter, UniqueViewMut, ViewMut};
use engine::camera::Camera;
use engine::rendering::depth_texture::DepthTexture;
use engine::rendering::graphics_context::GraphicsContext;
use engine::rendering::texture::Texture;

pub fn resize(new_size: winit::dpi::PhysicalSize<u32>, mut g_ctx: UniqueViewMut<GraphicsContext>, mut cameras: ViewMut<Camera>, mut depth_texture: UniqueViewMut<DepthTexture>) {
    if new_size.width > 0 && new_size.height > 0 {
        g_ctx.resize(new_size);

        let aspect = g_ctx.aspect();

        (&mut cameras)
            .iter()
            .for_each(|c| c.perspective.set_aspect(aspect));

        let new_depth_texture = Texture::create_depth_texture(&g_ctx.device, &g_ctx.config, "depth texture");
        *depth_texture = DepthTexture(new_depth_texture);
    } else {
        tracing::warn!("Ignoring resize with non-positive width or height ({new_size:?})");
    }
}

pub fn reconfigure(g_ctx: UniqueViewMut<GraphicsContext>, camera: ViewMut<Camera>, depth_texture: UniqueViewMut<DepthTexture>) {
    resize(g_ctx.size, g_ctx, camera, depth_texture);
}
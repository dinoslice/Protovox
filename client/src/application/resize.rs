use shipyard::UniqueViewMut;
use crate::camera::Camera;
use crate::rendering::graphics_context::GraphicsContext;

pub fn resize(new_size: winit::dpi::PhysicalSize<u32>, mut g_ctx: UniqueViewMut<GraphicsContext>, mut camera: UniqueViewMut<Camera>) {
    if new_size.width > 0 && new_size.height > 0 {
        g_ctx.resize(new_size);
        camera.perspective.set_aspect(g_ctx.aspect());
    } else {
        tracing::warn!("Ignoring resize with non-positive width or height");
    }
}

pub fn reconfigure(g_ctx: UniqueViewMut<GraphicsContext>, camera: UniqueViewMut<Camera>) {
    let size = g_ctx.size;
    resize(size, g_ctx, camera);
}
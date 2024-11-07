use egui_wgpu::ScreenDescriptor;
use shipyard::{UniqueView, UniqueViewMut};  
use crate::rendering::egui::EguiRenderer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render::RenderContext;

pub fn render_egui(
    mut ctx: UniqueViewMut<RenderContext>,
    g_ctx: UniqueView<GraphicsContext>,
    mut egui_renderer: UniqueViewMut<EguiRenderer>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();
    
    let screen_descriptor = ScreenDescriptor {
        size_in_pixels: [g_ctx.config.width, g_ctx.config.height],
        pixels_per_point: g_ctx.window.scale_factor() as _,
    };
    
    egui_renderer.draw(
        &g_ctx.device,
        &g_ctx.queue,
        encoder,
        &g_ctx.window,
        tex_view,
        screen_descriptor,
        |ctx| {
            egui::Window::new("egui window!")
                .resizable(true)
                .default_open(false)
                .show(ctx, |ui| {
                    ui.label("it works!")
                });
        },
    );
}
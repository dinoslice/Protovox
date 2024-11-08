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
    
    egui_renderer.draw(&g_ctx, encoder, tex_view, |ctx| {
        egui::Window::new("egui window!")
            .resizable(true)
            .default_open(false)
            .show(ctx, |ui| {
                ui.label("it works!")
            });
    });
}
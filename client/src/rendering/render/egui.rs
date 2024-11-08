use shipyard::{IntoIter, UniqueView, UniqueViewMut, View};
use crate::components::{Entity, LocalPlayer, Transform};
use crate::networking::server_handler::ServerHandler;
use crate::rendering::egui::EguiRenderer;
use crate::rendering::graphics_context::GraphicsContext;
use crate::rendering::render::RenderContext;

pub fn render_egui(
    mut ctx: UniqueViewMut<RenderContext>,
    g_ctx: UniqueView<GraphicsContext>,
    mut egui_renderer: UniqueViewMut<EguiRenderer>,

    // for player debug info
    v_local_player: View<LocalPlayer>,
    v_entity: View<Entity>,
    v_transform: View<Transform>,

    opt_server_handler: Option<UniqueView<ServerHandler>>,
) {
    let RenderContext { tex_view, encoder, .. } = ctx.as_mut();

    let pos_fmt = |v: &glm::Vec3| format!("Position: [{:.2}, {:.2}, {:.2}]", v.x, v.y, v.z);
    
    let local_pos = (&v_local_player, &v_transform)
        .iter()
        .next()
        .expect("LocalPlayer didn't have transform")
        .1
        .position;

    let mut other_pos = (!&v_local_player, &v_entity, &v_transform).iter()
        .map(|e| &e.2.position)
        .peekable();

    egui_renderer.draw(&g_ctx, encoder, tex_view, |ctx| {
        egui::Window::new("Entities")
            .default_open(true)
            .show(ctx, |ui| {
                ui.heading("LocalPlayer");
                ui.label(pos_fmt(&local_pos));
                
                if other_pos.peek().is_some() {
                    ui.heading("Entities");
                    
                    for pos in other_pos {
                        ui.label(pos_fmt(pos));
                    }
                }
            });

        if let Some(server_handler) = opt_server_handler {
            egui::Window::new("ServerHandler")
                .default_open(true)
                .show(ctx, |ui| {
                    ui.label(format!("Address: {}", server_handler.local_addr));
                });
        }
    });
}